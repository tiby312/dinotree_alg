//! Provides 2d broadphase collision detection.

mod inner;
mod node_handle;
pub(crate) mod oned;


use crate::inner_prelude::*;
use dinotree::notsorted::*;
use self::node_handle::*;
use self::inner::*;

///Used for the advanced algorithms.
///Trait that user implements to handling aabb collisions.
///The user supplies a struct that implements this trait instead of just a closure
///so that the user may also have the struct implement Splitter.
pub trait ColMulti{
    type T: HasAabbMut;

    fn collide(&mut self,
        a: BBoxRefMut<<Self::T as HasAabb>::Num,<Self::T as HasAabb>::Inner>,
        b: BBoxRefMut<<Self::T as HasAabb>::Num,<Self::T as HasAabb>::Inner>);
}


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

impl<K:NotSortedRefMutTrait> NotSortedQueryBuilder<K> where K::Item:Send+Sync{

    #[inline(always)]
    pub fn query_par(self,func:impl Fn(
                    BBoxRefMut<K::Num,K::Inner>,
            BBoxRefMut<K::Num,K::Inner>)+Copy+Send+Sync){
        let mut tree=self.tree;
        let b=inner::QueryFn::new(func);
        let mut sweeper=HandleNoSorted::new(b);

        let axis=tree.axis();
        let oo=tree.vistr_mut();
        let switch_height=self.switch_height;
        let par=compute_default_level_switch_sequential(switch_height,oo.height());
        ColFindRecurser::new().recurse_par(axis, par, &mut sweeper, oo.with_depth(Depth(0)),&mut SplitterEmpty);
    }

}

impl<K:NotSortedRefMutTrait> NotSortedQueryBuilder<K>{

    #[inline(always)]
    pub fn new(tree:K)->NotSortedQueryBuilder<K>{
        let switch_height=default_level_switch_sequential();
        NotSortedQueryBuilder{switch_height,tree}
    }

    #[inline(always)]
    pub fn query_with_splitter_seq(mut self,func:impl FnMut(
                    BBoxRefMut<K::Num,K::Inner>,
            BBoxRefMut<K::Num,K::Inner>),splitter:&mut impl Splitter){
        let b=inner::QueryFnMut::new(func);        
        let mut sweeper=HandleNoSorted::new(b);

        let axis=self.tree.axis();
        let vistr_mut=self.tree.vistr_mut();    
        let dt = vistr_mut.with_depth(Depth(0));
        ColFindRecurser::new().recurse_seq(axis, &mut sweeper, dt,splitter);
    }    

    #[inline(always)]
    pub fn query_seq(mut self,func:impl FnMut(
        BBoxRefMut<K::Num,K::Inner>,
        BBoxRefMut<K::Num,K::Inner>)){
        let b=inner::QueryFnMut::new(func);
        let mut sweeper=HandleNoSorted::new(b);

        let axis=self.tree.axis();
        let vistr_mut=self.tree.vistr_mut();    
        let dt = vistr_mut.with_depth(Depth(0));
        ColFindRecurser::new().recurse_seq(axis, &mut sweeper, dt,&mut SplitterEmpty);
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

impl<K:DinoTreeRefMutTrait> QueryBuilder<K> where K::Item: Send+Sync{

    ///Perform the query in parallel
    #[inline(always)]
    pub fn query_par(mut self,func:impl Fn(
            BBoxRefMut<K::Num,K::Inner>,
            BBoxRefMut<K::Num,K::Inner>
        )+Clone+Send+Sync){
        let b=inner::QueryFn::new(func);
        let mut sweeper=HandleSorted::new(b);

        let switch_height=self.switch_height;
        let axis=self.tree.axis();
        let oo=self.tree.vistr_mut();
        let par=compute_default_level_switch_sequential(switch_height,oo.height());
        ColFindRecurser::new().recurse_par(axis, par, &mut sweeper, oo.with_depth(Depth(0)),&mut SplitterEmpty);

    }


    ///The user has more control using this version of the query.
    ///The splitter will split and add at every level.
    ///The clos will split and add only at levels that are handled in parallel.
    ///This can be useful if the use wants to create a list of colliding pair indicies, but still wants paralleism.
    #[inline(always)]
    pub fn query_splitter_par<C:ColMulti<T=K::Item>+Splitter+Send+Sync>(mut self,clos:C){
        let axis=self.tree.axis();
        let vistr_mut=self.tree.vistr_mut();

        let par=compute_default_level_switch_sequential(self.switch_height,vistr_mut.height());


        let dt = vistr_mut.with_depth(Depth(0));
        let mut sweeper=HandleSorted::new(clos);
        ColFindRecurser::new().recurse_par(axis, par,&mut sweeper, dt,&mut SplitterEmpty);
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
    pub fn query_seq(mut self,func:impl FnMut(
        BBoxRefMut<K::Num,K::Inner>,
        BBoxRefMut<K::Num,K::Inner>
        )){
        let b=inner::QueryFnMut::new(func);
        let mut sweeper=HandleSorted::new(b);
        let mut splitter=SplitterEmpty;

        let axis=self.tree.axis();
        let vistr_mut=self.tree.vistr_mut();    
        let dt = vistr_mut.with_depth(Depth(0));
        ColFindRecurser::new().recurse_seq(axis, &mut sweeper, dt,&mut splitter);
    }

    ///Perform the query sequentially with a splitter.
    #[inline(always)]
    pub fn query_with_splitter_seq(mut self,func:impl FnMut(
        BBoxRefMut<K::Num,K::Inner>,
        BBoxRefMut<K::Num,K::Inner>),splitter:&mut impl Splitter){

        let b=inner::QueryFnMut::new(func);
        
        let mut sweeper=HandleSorted::new(b);
        let axis=self.tree.axis();
        let vistr_mut=self.tree.vistr_mut();    
        let dt = vistr_mut.with_depth(Depth(0));
        ColFindRecurser::new().recurse_seq(axis, &mut sweeper, dt,splitter);
    }     
}

