//! Provides 2d broadphase collision detection.

use crate::inner_prelude::*;

use dinotree::notsorted::*;

///Used for the advanced algorithms.
///Trait that user implements to handling aabb collisions.
///The user supplies a struct that implements this trait instead of just a closure
///so that the user may also have the struct implement Splitter.
pub trait ColMulti{
    type T: HasAabbMut;
    //type Num:NumTrait;
    //type Inner;
    fn collide(&mut self,
        a: BBoxRefMut<<Self::T as HasAabb>::Num,<Self::T as HasAabb>::Inner>,
        b: BBoxRefMut<<Self::T as HasAabb>::Num,<Self::T as HasAabb>::Inner>);
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
pub fn query_naive_mut<T:HasAabbMut>(bots:ElemSliceMut<T>,mut func:impl FnMut(BBoxRefMut<T::Num,T::Inner>,BBoxRefMut<T::Num,T::Inner>)){
    //let bots=ElemSliceMut::new(ElemSlice::from_slice_mut(bots));
    tools::for_every_pair(bots,|a,b|{
        if a.rect.get_intersect_rect(b.rect).is_some(){
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
pub fn query_sweep_mut<T:HasAabbMut>(axis:impl AxisTrait,bots:&mut [T],func:impl FnMut(BBoxRefMut<T::Num,T::Inner>,BBoxRefMut<T::Num,T::Inner>)){  
    ///Sorts the bots.
    #[inline(always)]
    fn sweeper_update<I:HasAabb,A:AxisTrait>(axis:A,collision_botids: &mut [I]) {

        let sclosure = |a: &I, b: &I| -> core::cmp::Ordering {
            let (p1,p2)=(a.get().rect.get_range(axis).left,b.get().rect.get_range(axis).left);
            if p1 > p2 {
                return core::cmp::Ordering::Greater;
            }
            core::cmp::Ordering::Less
        };

        collision_botids.sort_unstable_by(sclosure);
    }

    sweeper_update(axis,bots);


    struct Bl<T:HasAabb,F: FnMut(BBoxRefMut<T::Num,T::Inner>,BBoxRefMut<T::Num,T::Inner>)> {
        func: F,
        _p:PhantomData<T>
    }

    impl<T:HasAabbMut,F: FnMut(BBoxRefMut<T::Num,T::Inner>,BBoxRefMut<T::Num,T::Inner>)> ColMulti for Bl<T,F> {
        type T = T;
        #[inline(always)]
        fn collide(&mut self, a: BBoxRefMut<T::Num,T::Inner>, b: BBoxRefMut<T::Num,T::Inner>) {    
            (self.func)(a, b);
        }
       
    }

    let mut s=oned::Sweeper::new();
    s.find_2d(axis,ElemSliceMut::new(ElemSlice::from_slice_mut(bots)),&mut Bl{func,_p:PhantomData});
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
    pub fn query_par(self,func:impl Fn(
                    BBoxRefMut<K::Num,K::Inner>,
            BBoxRefMut<K::Num,K::Inner>)+Copy+Send){
        let mut tree=self.tree;
        let b=inner::QueryFn::new(func);
        let mut sweeper=HandleNoSorted::new(b);

        let axis=tree.axis();
        let oo=tree.vistr_mut();
        let switch_height=self.switch_height;
        let par=compute_default_level_switch_sequential(switch_height,oo.height());
        ColFindRecurser::new().recurse(axis, par, &mut sweeper, oo.with_depth(Depth(0)),&mut SplitterEmpty);
    }

    #[inline(always)]
    pub fn query_with_splitter_seq(self,func:impl FnMut(
                    BBoxRefMut<K::Num,K::Inner>,
            BBoxRefMut<K::Num,K::Inner>),splitter:&mut impl Splitter){
        let b=inner::QueryFnMut::new(func);        
        let mut sweeper=HandleNoSorted::new(b);
        inner_query_seq_adv_mut_not_sorted(self.tree,splitter,&mut sweeper);
    }    

    #[inline(always)]
    pub fn query_seq(self,func:impl FnMut(
        BBoxRefMut<K::Num,K::Inner>,
        BBoxRefMut<K::Num,K::Inner>)){
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
    pub fn query_par(mut self,func:impl Fn(
            BBoxRefMut<K::Num,K::Inner>,
            BBoxRefMut<K::Num,K::Inner>
        )+Clone+Send){
        let b=inner::QueryFn::new(func);
        let mut sweeper=HandleSorted::new(b);

        let switch_height=self.switch_height;
        let axis=self.tree.axis();
        let oo=self.tree.vistr_mut();
        let par=compute_default_level_switch_sequential(switch_height,oo.height());
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

        let par=compute_default_level_switch_sequential(self.switch_height,vistr_mut.height());


        let dt = vistr_mut.with_depth(Depth(0));
        let mut sweeper=HandleSorted::new(clos);
        ColFindRecurser::new().recurse(axis, par, &mut sweeper, dt,&mut SplitterEmpty);
    }
}

impl<K:DinoTreeRefMutTrait> QueryBuilder<K>{

    ///Create the builder.
    #[inline(always)]
    pub fn new(tree:K)->QueryBuilder<K>{
        let switch_height=default_level_switch_sequential();
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
    pub fn query_seq(self,func:impl FnMut(
        BBoxRefMut<K::Num,K::Inner>,
        BBoxRefMut<K::Num,K::Inner>
        )){
        let b=inner::QueryFnMut::new(func);
        let mut sweeper=HandleSorted::new(b);
        let mut splitter=SplitterEmpty;
        inner_query_seq_adv_mut(self.tree,&mut splitter,&mut sweeper);
    }

    ///Perform the query sequentially with a splitter.
    #[inline(always)]
    pub fn query_with_splitter_seq(self,func:impl FnMut(
        BBoxRefMut<K::Num,K::Inner>,
        BBoxRefMut<K::Num,K::Inner>),splitter:&mut impl Splitter){

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
    impl<T:HasAabb> HasAabb for Wrap<T>{
        type Num=T::Num;
        type Inner=T::Inner;
        #[inline(always)]
        fn get(&self)->BBoxRef<T::Num,T::Inner>{
            self.0.get()
        }
    }
    impl<T:HasAabbMut> HasAabbMut for Wrap<T>{
        fn get_mut(&mut self)->BBoxRefMut<T::Num,T::Inner>{
            self.0.get_mut()
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
        fn handle_node(&mut self,axis:impl AxisTrait,bots: ElemSliceMut<Self::T>)
        {
            //let bots:&mut [T::T]=unsafe{std::mem::transmute(bots)};
            let bots:ElemSliceMut<T::T>=unsafe{core::mem::transmute(bots)};
            //let bots:&mut ElemSlice<T::T>=unsafe{&mut *(bots as *mut ElemSlice<Wrap<T::T>> as *mut ElemSlice<T::T>)};
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


