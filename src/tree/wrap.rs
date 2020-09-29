use super::*;
pub struct DinoTreeWrap<'a,A:Axis,T:Aabb>{
    inner:DinoTree<'a,A,T>,
    bots:PMutPtr<[T]>
}
impl<'a, T: Aabb> DinoTreeWrap<'a, DefaultA, T> {
    /// # Examples
    ///
    ///```
    ///let mut bots = [axgeom::rect(0,10,0,10)];
    ///let tree = dinotree_alg::DinoTree::new(&mut bots);
    ///
    ///```
    pub fn new(bots: &'a mut [T]) -> DinoTreeWrap<'a, DefaultA, T> {
        DinoTreeWrap::new_inner(bots,|bots|DinoTree::new(bots))
    }
}
    
impl<'a, T: Aabb + Send + Sync> DinoTreeWrap<'a, DefaultA, T> {
    /// # Examples
    ///
    ///```
    ///let mut bots = [axgeom::rect(0,10,0,10)];
    ///let tree = dinotree_alg::DinoTree::new_par(&mut bots);
    ///
    ///```
    pub fn new_par(bots: &'a mut [T]) -> DinoTreeWrap<'a, DefaultA, T> {
        DinoTreeWrap::new_inner(bots,|bots|DinoTree::new_par(bots))
    }
}

impl<'a, A: Axis, T: Aabb> DinoTreeWrap<'a, A, T> {
    /// # Examples
    ///
    ///```
    ///let mut bots = [axgeom::rect(0,10,0,10)];
    ///let tree = dinotree_alg::DinoTree::with_axis(axgeom::XAXIS,&mut bots);
    ///
    ///```
    pub fn with_axis(axis: A, bots: &'a mut [T]) -> DinoTreeWrap<'a, A, T> {
        DinoTreeWrap::new_inner(bots,|bots|DinoTree::with_axis(axis,bots))
    }
}

impl<'a, A: Axis, T: Aabb + Send + Sync> DinoTreeWrap<'a, A, T> {
    /// # Examples
    ///
    ///```
    ///let mut bots = [axgeom::rect(0,10,0,10)];
    ///let tree = dinotree_alg::DinoTree::with_axis(axgeom::XAXIS,&mut bots);
    ///
    ///```
    pub fn with_axis_par(axis: A, bots: &'a mut [T]) -> DinoTreeWrap<'a, A, T> {
        DinoTreeWrap::new_inner(bots,|bots|DinoTree::with_axis_par(axis,bots))
    }
}

impl<'a,'b,A:Axis,T:Aabb> core::ops::Deref for DinoTreeWrap<'a,A,T> {
    type Target = DinoTree<'a,A,T>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'a,'b,A:Axis,T:Aabb> core::ops::DerefMut for DinoTreeWrap<'a,A,T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}


impl<'a,A:Axis,T:Aabb> DinoTreeWrap<'a,A,T>{
    fn new_inner(arr:&'a mut [T], func:impl FnOnce(&'a mut [T])->DinoTree<'a,A,T>)->DinoTreeWrap<'a,A,T>{
        let bots=PMut::new(arr).as_ptr();
        let inner=func(arr);
        //TODO pick one bot in a node and assert it is within the arr.
        DinoTreeWrap{inner,bots}
    }
    pub fn get_tree(&self)->&DinoTree<'a,A,T>{
        &self.inner
    }
    

    pub fn get_tree_mut(&mut self)->&mut DinoTree<'a,A,T>{
        &mut self.inner
    }
    
    ///Returns the elements in the tree in the order
    ///they are arranged internally in the tree.
    #[must_use]
    #[inline(always)]
    pub fn get_bots(&self) -> &[T] {
        &unsafe { self.bots.as_mut() }
    }

    ///Returns the elements in the tree in the order
    ///they are arranged internally in the tree.
    ///The elements are prevented from being mutated
    ///such that their aabb changes through use of
    ///the PMut pointer type.
    #[must_use]
    #[inline(always)]
    pub fn get_bots_mut(&mut self) -> PMut<[T]> {
        unsafe { self.bots.as_mut() }
    }


}
