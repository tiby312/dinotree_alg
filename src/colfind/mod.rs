
//! ## Overview
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
    fn sweeper_update<I:HasAabb,A:AxisTrait>(axis:A,collision_botids: &mut [I]) {

        let sclosure = |a: &I, b: &I| -> std::cmp::Ordering {
            let (p1,p2)=(a.get().get_range(axis).left,b.get().get_range(axis).left);
            if p1 > p2 {
                return std::cmp::Ordering::Greater;
            }
            std::cmp::Ordering::Less
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
        fn collide(&mut self, a: &mut Self::T, b: &mut Self::T) {    
            (self.func)(a, b);
        }
       
    }

    let mut s=oned::Sweeper::new();
    s.find_2d(axis,bots,&mut Bl{func,_p:PhantomData});
}



///Builder for a query on a NotSorted Dinotree.
pub struct NotSortedQueryBuilder<'a,A:AxisTrait,T:HasAabb>{
    switch_height:usize,
    tree:&'a mut NotSorted<A,T>
}
impl<'a,A:AxisTrait,T:HasAabb+Send> NotSortedQueryBuilder<'a,A,T>{


    pub fn new(tree:&'a mut NotSorted<A,T>)->NotSortedQueryBuilder<'a,A,T>{
        let switch_height=default_level_switch_sequential();
        NotSortedQueryBuilder{switch_height,tree}
    }
    pub fn query_par(self,func:impl Fn(&mut T,&mut T)+Copy+Send){
        let mut tree=self.tree.0.as_ref_mut();
        let b=inner::QueryFn::new(func);
        let mut sweeper=HandleNoSorted::new(b);

        let axis=tree.axis();
        let oo=tree.vistr_mut();
        let switch_height=self.switch_height;
        let par=dinotree::advanced::compute_default_level_switch_sequential(switch_height,oo.height());
        ColFindRecurser::new().recurse(axis, par, &mut sweeper, oo.with_depth(Depth(0)),&mut SplitterEmpty);
    }

    pub fn query_with_splitter_seq(self,func:impl FnMut(&mut T,&mut T),splitter:&mut impl Splitter){
        let b=inner::QueryFnMut::new(func);        
        let mut sweeper=HandleNoSorted::new(b);
        inner_query_seq_adv_mut(self.tree.0.as_ref_mut(),splitter,&mut sweeper);
    }    

    pub fn query_seq(self,func:impl FnMut(&mut T,&mut T)){
        let b=inner::QueryFnMut::new(func);
        let mut sweeper=HandleNoSorted::new(b);
        inner_query_seq_adv_mut(self.tree.0.as_ref_mut(),&mut SplitterEmpty,&mut sweeper);
    }
}

///Builder for a query on a DinoTree.
/// # Examples
///
/// ```
/// use axgeom;
/// use dinotree_sample::SampleBuilder;
/// use dinotree::DinoTreeBuilder;
/// use dinotree_alg::colfind::QueryBuilder;
///
/// let builder = SampleBuilder::new();
/// let mut bots:Vec<_>= builder.build().take(1000).collect();
/// let mut tree=DinoTreeBuilder::new(axgeom::XAXISS,&mut bots,|a|builder.create_aabb(a)).build_seq();
/// QueryBuilder::new(tree.as_ref_mut()).query_seq(|a,b|a.inner.collide(&mut b.inner));
/// ```
pub struct QueryBuilder<'a,A:AxisTrait,T:HasAabb>{
    switch_height:usize,
    tree:DinoTreeRefMut<'a,A,T>
}


impl<'a,A:AxisTrait,T:HasAabb+Send> QueryBuilder<'a,A,T>{

    ///Perform the query in parallel
    pub fn query_par(mut self,func:impl Fn(&mut T,&mut T)+Clone+Send){
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
    pub fn query_splitter<C:ColMulti<T=T>+Splitter+Send>(mut self,clos:C){
        let axis=self.tree.axis();
        let vistr_mut=self.tree.vistr_mut();

        let par=dinotree::advanced::compute_default_level_switch_sequential(self.switch_height,vistr_mut.height());


        let dt = vistr_mut.with_depth(Depth(0));
        let mut sweeper=HandleSorted::new(clos);
        ColFindRecurser::new().recurse(axis, par, &mut sweeper, dt,&mut SplitterEmpty);
    }
}
impl<'a,A:AxisTrait,T:HasAabb> QueryBuilder<'a,A,T>{

    ///Create the builder.
    pub fn new(tree:DinoTreeRefMut<'a,A,T>)->QueryBuilder<'a,A,T>{
        let switch_height=dinotree::advanced::default_level_switch_sequential();
        QueryBuilder{switch_height,tree}
    }

    ///Choose a custom height at which to switch from parallel to sequential.
    ///If you end up building sequentially, this option is ignored.
    pub fn with_switch_height(mut self,height:usize)->Self{
        self.switch_height=height;
        self
    }
    
    ///Perform the query sequentially.
    pub fn query_seq(self,func:impl FnMut(&mut T,&mut T)){
        let b=inner::QueryFnMut::new(func);
        let mut sweeper=HandleSorted::new(b);
        let mut splitter=SplitterEmpty;
        inner_query_seq_adv_mut(self.tree,&mut splitter,&mut sweeper);
    }

    ///Perform the query sequentially with a splitter.
    pub fn query_with_splitter_seq(self,func:impl FnMut(&mut T,&mut T),splitter:&mut impl Splitter){

        let b=inner::QueryFnMut::new(func);
        
        let mut sweeper=HandleSorted::new(b);


        inner_query_seq_adv_mut(self.tree,splitter,&mut sweeper);
    }     
}



///See query_adv_mut
fn inner_query_seq_adv_mut<
    A: AxisTrait,
    T: HasAabb,
    K:Splitter,
    S: NodeHandler<T=T>+Splitter>(   
    mut tree:DinoTreeRefMut<A,T>,
    splitter:&mut K,
    sweeper:&mut S
){
  

    mod wrap{
        //Use this to get rid of Send trait constraint.
        #[repr(transparent)]
        pub struct Wrap<T>(T);
        unsafe impl<T> Send for Wrap<T>{}
        unsafe impl<T> Sync for Wrap<T>{}
        unsafe impl<T:HasAabb> HasAabb for Wrap<T>{
            type Num=T::Num;
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
            fn div(&mut self)->Self{
                SplitterWrapper(self.0.div())
            }
            fn add(&mut self,a:Self){
                self.0.add(a.0)
            }
            fn node_start(&mut self){self.0.node_start()}
            fn node_end(&mut self){self.0.node_end()}
        }        
        unsafe impl<T> Send for SplitterWrapper<T>{}
        unsafe impl<T> Sync for SplitterWrapper<T>{}


        #[repr(transparent)]
        pub struct NodeHandlerWrapper<T>(pub T);
        

        impl<T:NodeHandler> NodeHandler for NodeHandlerWrapper<T>{
            type T=Wrap<T::T>;
            fn handle_node(&mut self,axis:impl AxisTrait,bots:&mut [Self::T])
            {
                //let bots:&mut [T::T]=unsafe{std::mem::transmute(bots)};
                let bots:&mut [T::T]=unsafe{&mut *(bots as *mut [Wrap<T::T>] as *mut [T::T])};
                self.0.handle_node(axis,bots);
            }
            fn handle_children<A:AxisTrait,B:AxisTrait>(&mut self,
                anchor:&mut DestructuredNode<Self::T,A>,
                current:&mut DestructuredNodeLeaf<Self::T,B>){

                let anchor:&mut DestructuredNode<T::T,A>=unsafe{&mut *((anchor as *mut DestructuredNode<Self::T,A>) as *mut DestructuredNode<T::T,A>)};
                let current:&mut DestructuredNodeLeaf<T::T,B>=unsafe{&mut *((current as *mut DestructuredNodeLeaf<Self::T,B>) as *mut DestructuredNodeLeaf<T::T,B>)};

                self.0.handle_children(anchor,current);
            }
        }
        impl<T:NodeHandler+Splitter> Splitter for NodeHandlerWrapper<T>{
            fn div(&mut self)->Self{
                NodeHandlerWrapper(self.0.div())
            }
            fn add(&mut self,a:Self){
                self.0.add(a.0)
            }
            fn node_start(&mut self){
                self.0.node_start();
            }
            fn node_end(&mut self){
                self.0.node_end();
            }
        }
        unsafe impl<T> Send for NodeHandlerWrapper<T>{}
        unsafe impl<T> Sync for NodeHandlerWrapper<T>{}
    }

    
    //let splitter:&mut wrap::SplitterWrapper<K>=unsafe{std::mem::transmute(splitter)};
    let splitter:&mut wrap::SplitterWrapper<K>=unsafe{&mut *(splitter as *mut K as *mut wrap::SplitterWrapper<K>)};

    let axis=tree.axis();

    let dt=tree.vistr_mut();    
    let vistr_mut:VistrMut<wrap::Wrap<T>>=unsafe{std::mem::transmute(dt)};
    
    //let sweeper:&mut wrap::NodeHandlerWrapper<S>=unsafe{std::mem::transmute(sweeper)};//wrap::NodeHandlerWrapper(sweeper);
    let sweeper:&mut wrap::NodeHandlerWrapper<S>=unsafe{&mut *(sweeper as *mut S as *mut wrap::NodeHandlerWrapper<S>)};

    let dt = vistr_mut.with_depth(Depth(0));
    
    
    ColFindRecurser::new().recurse(axis, par::Sequential, sweeper, dt,splitter);
    
}

