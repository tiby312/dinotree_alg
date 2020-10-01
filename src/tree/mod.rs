use crate::inner_prelude::*;

#[cfg(test)]
mod tests;

///A version of dinotree that is not lifetimed and uses unsafe{} to own the elements
///that are in its tree (as a self-referential struct). Composed of `(Rect<N>,*mut T)`.
pub mod owned;


pub mod assert;

pub mod analyze;

///Contains code to write generic code that can be run in parallel, or sequentially. The api is exposed
///in case users find it useful when writing parallel query code to operate on the tree.
pub mod par;

mod notsorted;
pub(crate) use self::notsorted::NotSorted;


use crate::query::*;


pub struct DinoTreeIndPtr<'a,A:Axis,N:Num,T>{
    inner:owned::DinoTreeOwned<A,BBox<N,*mut T>>,
    orig:*mut [T],
    _p:PhantomData<&'a mut T>
}

impl<'a,N:Num,T> DinoTreeIndPtr<'a,DefaultA,N,T>{
    pub fn new(arr:&'a mut [T],func:impl FnMut(&mut T)->Rect<N>)->DinoTreeIndPtr<'a,DefaultA,N,T>{
        DinoTreeIndPtr::with_axis(default_axis(),arr,func)
    }
}

impl<'a,A:Axis,N:Num,T> DinoTreeIndPtr<'a,A,N,T>{
    pub fn with_axis(axis:A,arr:&'a mut [T],mut func:impl FnMut(&mut T)->Rect<N>)->DinoTreeIndPtr<'a,A,N,T>{
        let orig=arr as *mut _;
        let bbox = arr
        .iter_mut()
        .map(|b| BBox::new(func(b), b as *mut _))
        .collect();

        let inner=owned::DinoTreeOwned::with_axis(axis,bbox);

        DinoTreeIndPtr{
            inner,
            orig,
            _p:PhantomData
        }
    }
    pub fn get_elements(&self)->&[T]{
        unsafe{&*self.orig}
    }
    pub fn get_elements_mut(&mut self)->PMut<'a,[T]>{
        PMut::new(unsafe{&mut *self.orig})
    }
}
impl<'a,A:Axis,N:Num+'a,T> core::ops::Deref for DinoTreeIndPtr<'a,A,N,T>{
    type Target=DinoTree<A,NodeMut<'a,BBox<N,&'a mut T>>>;
    fn deref(&self)->&Self::Target{
        //TODO do these in one place
        unsafe{&*(self.inner.as_tree() as *const _ as *const _)}
    }
}
pub struct DinoTreePtr<'a,A:Axis,T:Aabb>{
    inner:DinoTree<A,owned::NodePtr<T>>,
    orig:*mut [T],
    _p:PhantomData<&'a mut T>
}

impl<'a,A:Axis,T:Aabb> core::ops::Deref for DinoTreePtr<'a,A,T>{
    type Target=DinoTree<A,NodeMut<'a,T>>;
    fn deref(&self)->&Self::Target{
        //TODO do these in one place
        unsafe{&*(&self.inner as *const _ as *const _)}
    }
}
impl<'a,A:Axis,T:Aabb> core::ops::DerefMut for DinoTreePtr<'a,A,T>{
    fn deref_mut(&mut self)->&mut Self::Target{
        //TODO do these in one place
        unsafe{&mut *(&mut self.inner as *mut _ as *mut _)}
    }
}

impl<'a,T:Aabb> DinoTreePtr<'a,DefaultA,T>{
    pub fn new(arr:&'a mut [T])->DinoTreePtr<'a,DefaultA,T>{
        DinoTreePtr::with_axis(default_axis(),arr)
    }
}

impl<'a,A:Axis,T:Aabb> DinoTreePtr<'a,A,T>{
    pub fn with_axis(a:A,arr:&'a mut [T])->DinoTreePtr<'a,A,T>{
        let inner=owned::make_owned(a,arr);
        let orig=arr as *mut _;
        DinoTreePtr{
            inner,
            orig,
            _p:PhantomData
        }        
    }
    pub fn get_elements(&self)->&[T]{
        unsafe{&*self.orig}
    }
    pub fn get_elements_mut(&mut self)->PMut<'a,[T]>{
        PMut::new(unsafe{&mut *self.orig})
    }
}


///The data structure this crate revoles around.
pub struct DinoTree<A: Axis, N:Node> {
    axis: A,
    inner: compt::dfs_order::CompleteTreeContainer<N, compt::dfs_order::PreOrder>
}

///The type of the axis of the first node in the dinotree.
///If it is the y axis, then the first divider will be a horizontal line,
///since it is partioning space based off of objects y value.
pub type DefaultA = YAXIS;
///Constructor of the default axis type. Needed since you cannot construct from type alias's.
pub const fn default_axis() -> YAXIS {
    YAXIS
}

impl<'a, T: Aabb> DinoTree<DefaultA,NodeMut<'a, T>> {
    /// # Examples
    ///
    ///```
    ///let mut bots = [axgeom::rect(0,10,0,10)];
    ///let tree = dinotree_alg::DinoTree::new(&mut bots);
    ///
    ///```
    pub fn new(bots: &'a mut [T]) -> DinoTree< DefaultA, NodeMut<'a, T>> {
        DinoTreeBuilder::new(bots).build_seq()
    }
}

impl<'a, T: Aabb + Send + Sync> DinoTree<DefaultA, NodeMut<'a, T>> {
    /// # Examples
    ///
    ///```
    ///let mut bots = [axgeom::rect(0,10,0,10)];
    ///let tree = dinotree_alg::DinoTree::new_par(&mut bots);
    ///
    ///```
    pub fn new_par(bots: &'a mut [T]) -> DinoTree<DefaultA, NodeMut<'a, T>> {
        DinoTreeBuilder::new(bots).build_par()
    }
}

impl<'a, A: Axis, T: Aabb> DinoTree<A, NodeMut<'a, T>> {
    /// # Examples
    ///
    ///```
    ///let mut bots = [axgeom::rect(0,10,0,10)];
    ///let tree = dinotree_alg::DinoTree::with_axis(axgeom::XAXIS,&mut bots);
    ///
    ///```
    pub fn with_axis(axis: A, bots: &'a mut [T]) -> DinoTree<A, NodeMut<'a, T>> {
        DinoTreeBuilder::with_axis(axis, bots).build_seq()
    }

}



impl<'a, A: Axis, T: Aabb + Send + Sync> DinoTree< A, NodeMut<'a, T>> {
    /// # Examples
    ///
    ///```
    ///let mut bots = [axgeom::rect(0,10,0,10)];
    ///let tree = dinotree_alg::DinoTree::with_axis(axgeom::XAXIS,&mut bots);
    ///
    ///```
    pub fn with_axis_par(axis: A, bots: &'a mut [T]) -> DinoTree< A, NodeMut<'a,T>> {
        DinoTreeBuilder::with_axis(axis, bots).build_par()
    }
}

///TODO use this insead
impl<'a,A:Axis,T:Aabb> Queries2<'a> for DinoTree<A,NodeMut<'a,T>>{
    type A=A;
    type T=T;
    type Num=T::Num;
    
    #[inline(always)]
    fn axis(&self)->Self::A{
        self.axis
    }

    #[inline(always)]
    fn vistr_mut(&mut self)->VistrMut<NodeMut<'a,T>>{
        VistrMut{inner:self.inner.vistr_mut()}
    }

    #[inline(always)]
    fn vistr(&self)->Vistr<NodeMut<'a,T>>{
        self.inner.vistr()
    }
}


impl<A:Axis,N:Node> Queries for DinoTree<A,N>{
    type A=A;
    type N=N;
    type T=N::T;
    type Num=N::Num;
    
    #[inline(always)]
    fn axis(&self)->Self::A{
        self.axis
    }

    #[inline(always)]
    fn vistr_mut(&mut self)->VistrMut<N>{
        VistrMut{inner:self.inner.vistr_mut()}
    }

    #[inline(always)]
    fn vistr(&self)->Vistr<N>{
        self.inner.vistr()
    }
}



pub struct IntersectionList<'a, T, D> {
    ///See collect_intersections_list()
    ///The same elements can be part of
    ///multiple intersecting pairs.
    ///So pointer aliasing rules are not
    ///being met if we were to just use this
    ///vec according to its type signature.
    cols: Vec<(*mut T, *mut T, D)>,
    _p:PhantomData<&'a mut T>
}
impl<'a,T,D> IntersectionList<'a,T,D>{
    pub fn for_every_pair_mut<'b, A: Axis, N: Num>(
        &'b mut self,
        mut func: impl FnMut(&mut T, &mut T, &mut D),
    ) {
        for (a, b, d) in self.cols.iter_mut() {
            func(unsafe{&mut **a}, unsafe{&mut **b}, d)
        }
    }
}


impl<'a,'b,A:Axis,N:Num,T> DinoTree<A,NodeMut<'a, BBox<N,&'b mut T>>>{
    

    pub fn collect_intersections_list<'c,D: Send + Sync>(
        &mut self,
        mut func: impl FnMut(&mut T, &mut T) -> Option<D> + Send + Sync,
    ) -> IntersectionList<'b, T, D> {
        let mut cols: Vec<_> = Vec::new();
    
        self.find_intersections_mut(|a, b| {
            if let Some(d) = func(a, b) {
                //We use unsafe to collect mutable references of
                //all colliding pairs.
                //This is safe to do because the user is forced
                //to iterate through all the colliding pairs
                //one at a time.
                let a=*a as *mut T;
                let b=*b as *mut T;
                
                cols.push((a,b,d));
            }
        });

        IntersectionList {
            cols,
            _p:PhantomData
        }
    }
}



impl< A: Axis, N:Node> DinoTree< A, N> {

    /// # Examples
    ///
    ///```
    ///use dinotree_alg::*;
    ///let mut bots = vec![axgeom::rect(0,10,0,10);400];
    ///let mut tree = DinoTree::new(&mut bots);
    ///
    ///assert_eq!(tree.get_height(),analyze::compute_tree_height_heuristic(400,analyze::DEFAULT_NUMBER_ELEM_PER_NODE));
    ///```
    ///
    #[must_use]
    #[inline(always)]
    pub fn get_height(&self) -> usize {
        self.inner.get_height()
    }

    

    /// # Examples
    ///
    ///```
    ///use dinotree_alg::*;
    ///let mut bots = vec![axgeom::rect(0,10,0,10);400];
    ///let mut tree = DinoTree::new(&mut bots);
    ///
    ///assert_eq!(tree.num_nodes(),analyze::nodes_left(0,tree.get_height() ));
    ///
    ///```
    #[must_use]
    #[inline(always)]
    pub fn num_nodes(&self) -> usize {
        self.inner.get_nodes().len()
    }
}


pub use self::builder::DinoTreeBuilder;
mod builder;

pub(crate) use self::node::*;

///Contains node-level building block structs and visitors used for a DinoTree.
pub mod node;

