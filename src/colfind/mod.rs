//! Provides 2d broadphase collision detection.

use crate::inner_prelude::*;

///Used for the advanced algorithms.
///Trait that user implements to handling aabb collisions.
///The user supplies a struct that implements this trait instead of just a closure
///so that the user may also have the struct implement Splitter.
pub trait ColMulti{
    type T: HasAabb;
    fn collide(&mut self, a: &mut Self::T, b: &mut Self::T);
}




mod inner;
mod node_handle;
pub(crate) mod oned;

use self::node_handle::*;
use self::inner::*;

///Naive algorithm.
/// # Examples
///
/// ```
/// use dinotree_sample::SampleBuilder;
/// use dinotree_alg::colfind::query_naive_mut;
///
/// let mut builder=SampleBuilder::new();
/// let mut bots = dinotree::advanced::into_bbox_vec(builder.build().take(1000),|a|builder.create_aabb(a));
/// query_naive_mut(&mut bots,|a,b|a.inner.collide(&mut b.inner));
/// ```
pub fn query_naive_mut<T:HasAabb>(bots:&mut [T],mut func:impl FnMut(&mut T,&mut T)){
    tools::for_every_pair(bots,|a,b|{
        if a.get().get_intersect_rect(b.get()).is_some(){
            func(a,b);
        }
    });
}


///Sweep and prune algorithm.
///Naive algorithm.
/// # Examples
///
/// ```
/// use axgeom;
/// use dinotree_sample::SampleBuilder;
/// use dinotree_alg::colfind::query_sweep_mut;
///
/// let mut builder=SampleBuilder::new();
/// let mut bots = dinotree::advanced::into_bbox_vec(builder.build().take(1000),|a|builder.create_aabb(a));
/// query_sweep_mut(axgeom::XAXISS,&mut bots,|a,b|a.inner.collide(&mut b.inner));
/// ```
pub fn query_sweep_mut<T:HasAabb>(axis:impl AxisTrait,bots:&mut [T],func:impl FnMut(&mut T,&mut T)){  
    ///Sorts the bots.
    #[inline(always)]
    fn sweeper_update<I:HasAabb,A:AxisTrait>(axis:A,collision_botids: &mut [I]) {

        let sclosure = |a: &I, b: &I| -> core::cmp::Ordering {
            let (p1,p2)=(a.get().get_range(axis).left,b.get().get_range(axis).left);
            if p1 > p2 {
                return core::cmp::Ordering::Greater;
            }
            core::cmp::Ordering::Less
        };

        collision_botids.sort_unstable_by(sclosure);
    }

    sweeper_update(axis,bots);


    struct Bl<T:HasAabb,F: FnMut(&mut T,&mut T)> {
        func: F,
        _p:PhantomData<T>
    }

    impl<T:HasAabb,F: FnMut(&mut T,&mut T)> ColMulti for Bl<T,F> {
        type T = T;
        #[inline(always)]
        fn collide(&mut self, a: &mut Self::T, b: &mut Self::T) {    
            (self.func)(a, b);
        }
       
    }

    let mut s=oned::Sweeper::new();
    s.find_2d(axis,bots,&mut Bl{func,_p:PhantomData});
}



///Builder for a query on a NotSorted Dinotree.
pub struct NotSortedQueryBuilder<K:NotSortedRefMutTrait>{
    switch_height:usize,
    tree:K
}
impl<K:NotSortedRefMutTrait> NotSortedQueryBuilder<K> where K::Item:Send{

    #[inline(always)]
    pub fn new(tree:K)->NotSortedQueryBuilder<K>{
        let switch_height=default_level_switch_sequential();
        NotSortedQueryBuilder{switch_height,tree}
    }

    #[inline(always)]
    pub fn query_par(self,func:impl Fn(&mut K::Item,&mut K::Item)+Copy+Send){
        let mut tree=self.tree;
        let b=inner::QueryFn::new(func);
        let mut sweeper=HandleNoSorted::new(b);

        let axis=tree.axis();
        let oo=tree.vistr_mut();
        let switch_height=self.switch_height;
        let par=dinotree::advanced::compute_default_level_switch_sequential(switch_height,oo.height());
        ColFindRecurser::new().recurse(axis, par, &mut sweeper, oo.with_depth(Depth(0)),&mut SplitterEmpty);
    }

    #[inline(always)]
    pub fn query_with_splitter_seq(self,func:impl FnMut(&mut K::Item,&mut K::Item),splitter:&mut impl Splitter){
        let b=inner::QueryFnMut::new(func);        
        let mut sweeper=HandleNoSorted::new(b);
        inner_query_seq_adv_mut_not_sorted(self.tree,splitter,&mut sweeper);
    }    

    #[inline(always)]
    pub fn query_seq(self,func:impl FnMut(&mut K::Item,&mut K::Item)){
        let b=inner::QueryFnMut::new(func);
        let mut sweeper=HandleNoSorted::new(b);
        inner_query_seq_adv_mut_not_sorted(self.tree,&mut SplitterEmpty,&mut sweeper);
    }
}


///Builder for a query on a DinoTree.
/// # Examples
///
/// ```
/// use axgeom;
/// use dinotree_sample::SampleBuilder;
/// use dinotree::copy::DinoTreeBuilder;
/// use dinotree_alg::colfind::QueryBuilder;
///
/// let builder = SampleBuilder::new();
/// let mut bots:Vec<_>= builder.build().take(1000).collect();
/// let mut tree=DinoTreeBuilder::new(axgeom::XAXISS,&mut bots,|a|builder.create_aabb(a)).build_seq();
/// QueryBuilder::new(&mut tree).query_seq(|a,b|a.inner.collide(&mut b.inner));
/// ```
pub struct QueryBuilder<K:DinoTreeRefMutTrait>{
    switch_height:usize,
    tree:K
}


impl<K:DinoTreeRefMutTrait> QueryBuilder<K> where K::Item: Send{

    ///Perform the query in parallel
    #[inline(always)]
    pub fn query_par(mut self,func:impl Fn(&mut K::Item,&mut K::Item)+Clone+Send){
        let b=inner::QueryFn::new(func);
        let mut sweeper=HandleSorted::new(b);

        let switch_height=self.switch_height;
        let axis=self.tree.axis();
        let oo=self.tree.vistr_mut();
        let par=dinotree::advanced::compute_default_level_switch_sequential(switch_height,oo.height());
        ColFindRecurser::new().recurse(axis, par, &mut sweeper, oo.with_depth(Depth(0)),&mut SplitterEmpty);

    }

    ///The user has more control using this version of the query.
    ///The splitter will split and add at every level.
    ///The clos will split and add only at levels that are handled in parallel.
    ///This can be useful if the use wants to create a list of colliding pair indicies, but still wants paralleism.
    #[inline(always)]
    pub fn query_splitter<C:ColMulti<T=K::Item>+Splitter+Send>(mut self,clos:C){
        let axis=self.tree.axis();
        let vistr_mut=self.tree.vistr_mut();

        let par=dinotree::advanced::compute_default_level_switch_sequential(self.switch_height,vistr_mut.height());


        let dt = vistr_mut.with_depth(Depth(0));
        let mut sweeper=HandleSorted::new(clos);
        ColFindRecurser::new().recurse(axis, par, &mut sweeper, dt,&mut SplitterEmpty);
    }
}
impl<K:DinoTreeRefMutTrait> QueryBuilder<K>{

    ///Create the builder.
    #[inline(always)]
    pub fn new(tree:K)->QueryBuilder<K>{
        let switch_height=dinotree::advanced::default_level_switch_sequential();
        QueryBuilder{switch_height,tree}
    }

    ///Choose a custom height at which to switch from parallel to sequential.
    ///If you end up building sequentially, this option is ignored.
    #[inline(always)]
    pub fn with_switch_height(mut self,height:usize)->Self{
        self.switch_height=height;
        self
    }
    
    ///Perform the query sequentially.
    #[inline(always)]
    pub fn query_seq(self,func:impl FnMut(&mut K::Item,&mut K::Item)){
        let b=inner::QueryFnMut::new(func);
        let mut sweeper=HandleSorted::new(b);
        let mut splitter=SplitterEmpty;
        inner_query_seq_adv_mut(self.tree,&mut splitter,&mut sweeper);
    }

    ///Perform the query sequentially with a splitter.
    #[inline(always)]
    pub fn query_with_splitter_seq(self,func:impl FnMut(&mut K::Item,&mut K::Item),splitter:&mut impl Splitter){

        let b=inner::QueryFnMut::new(func);
        
        let mut sweeper=HandleSorted::new(b);


        inner_query_seq_adv_mut(self.tree,splitter,&mut sweeper);
    }     
}



///See query_adv_mut
fn inner_query_seq_adv_mut<
    V:DinoTreeRefMutTrait,
    K:Splitter,
    S: NodeHandler<T=V::Item>+Splitter>(   
    mut tree:V,
    splitter:&mut K,
    sweeper:&mut S
){    
    //let splitter:&mut wrap::SplitterWrapper<K>=unsafe{std::mem::transmute(splitter)};
    let splitter:&mut wrap::SplitterWrapper<K>=unsafe{&mut *(splitter as *mut K as *mut wrap::SplitterWrapper<K>)};

    let axis=tree.axis();

    let dt=tree.vistr_mut();    
    let vistr_mut:VistrMut<wrap::Wrap<V::Item>>=unsafe{core::mem::transmute(dt)};
    
    //let sweeper:&mut wrap::NodeHandlerWrapper<S>=unsafe{std::mem::transmute(sweeper)};//wrap::NodeHandlerWrapper(sweeper);
    let sweeper:&mut wrap::NodeHandlerWrapper<S>=unsafe{&mut *(sweeper as *mut S as *mut wrap::NodeHandlerWrapper<S>)};

    let dt = vistr_mut.with_depth(Depth(0));
    
    
    ColFindRecurser::new().recurse(axis, par::Sequential, sweeper, dt,splitter);
    
}

///See query_adv_mut
fn inner_query_seq_adv_mut_not_sorted<
    V:NotSortedRefMutTrait,
    K:Splitter,
    S: NodeHandler<T=V::Item>+Splitter>(   
    mut tree:V,
    splitter:&mut K,
    sweeper:&mut S
){    
    //let splitter:&mut wrap::SplitterWrapper<K>=unsafe{std::mem::transmute(splitter)};
    let splitter:&mut wrap::SplitterWrapper<K>=unsafe{&mut *(splitter as *mut K as *mut wrap::SplitterWrapper<K>)};

    let axis=tree.axis();

    let dt=tree.vistr_mut();    
    let vistr_mut:VistrMut<wrap::Wrap<V::Item>>=unsafe{core::mem::transmute(dt)};
    
    //let sweeper:&mut wrap::NodeHandlerWrapper<S>=unsafe{std::mem::transmute(sweeper)};//wrap::NodeHandlerWrapper(sweeper);
    let sweeper:&mut wrap::NodeHandlerWrapper<S>=unsafe{&mut *(sweeper as *mut S as *mut wrap::NodeHandlerWrapper<S>)};

    let dt = vistr_mut.with_depth(Depth(0));
    
    
    ColFindRecurser::new().recurse(axis, par::Sequential, sweeper, dt,splitter);
    
}



mod wrap{
    //Use this to get rid of Send trait constraint.
    #[repr(transparent)]
    pub struct Wrap<T>(T);
    unsafe impl<T> Send for Wrap<T>{}
    unsafe impl<T> Sync for Wrap<T>{}
    unsafe impl<T:HasAabb> HasAabb for Wrap<T>{
        type Num=T::Num;
        #[inline(always)]
        fn get(&self)->&Rect<Self::Num>{
            self.0.get()
        }
    }

    use super::*;

    #[repr(transparent)]
    pub struct SplitterWrapper<T>(
        pub T,
    );

    impl<T:Splitter> Splitter for SplitterWrapper<T>{
        #[inline(always)]
        fn div(&mut self)->Self{
            SplitterWrapper(self.0.div())
        }
        #[inline(always)]
        fn add(&mut self,a:Self){
            self.0.add(a.0)
        }
        #[inline(always)]
        fn node_start(&mut self){self.0.node_start()}
        #[inline(always)]
        fn node_end(&mut self){self.0.node_end()}
    }        
    unsafe impl<T> Send for SplitterWrapper<T>{}
    unsafe impl<T> Sync for SplitterWrapper<T>{}


    #[repr(transparent)]
    pub struct NodeHandlerWrapper<T>(pub T);
    

    impl<T:NodeHandler> NodeHandler for NodeHandlerWrapper<T>{
        type T=Wrap<T::T>;
        #[inline(always)]
        fn handle_node(&mut self,axis:impl AxisTrait,bots:&mut [Self::T])
        {
            //let bots:&mut [T::T]=unsafe{std::mem::transmute(bots)};
            let bots:&mut [T::T]=unsafe{&mut *(bots as *mut [Wrap<T::T>] as *mut [T::T])};
            self.0.handle_node(axis,bots);
        }
        #[inline(always)]
        fn handle_children<A:AxisTrait,B:AxisTrait>(&mut self,
            anchor:&mut DestructuredNode<Self::T,A>,
            current:&mut DestructuredNodeLeaf<Self::T,B>){

            let anchor:&mut DestructuredNode<T::T,A>=unsafe{&mut *((anchor as *mut DestructuredNode<Self::T,A>) as *mut DestructuredNode<T::T,A>)};
            let current:&mut DestructuredNodeLeaf<T::T,B>=unsafe{&mut *((current as *mut DestructuredNodeLeaf<Self::T,B>) as *mut DestructuredNodeLeaf<T::T,B>)};

            self.0.handle_children(anchor,current);
        }
    }
    impl<T:NodeHandler+Splitter> Splitter for NodeHandlerWrapper<T>{
        #[inline(always)]
        fn div(&mut self)->Self{
            NodeHandlerWrapper(self.0.div())
        }
        #[inline(always)]
        fn add(&mut self,a:Self){
            self.0.add(a.0)
        }
        #[inline(always)]
        fn node_start(&mut self){
            self.0.node_start();
        }
        #[inline(always)]
        fn node_end(&mut self){
            self.0.node_end();
        }
    }
    unsafe impl<T> Send for NodeHandlerWrapper<T>{}
    unsafe impl<T> Sync for NodeHandlerWrapper<T>{}
}