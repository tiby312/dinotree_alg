use super::*;

///A version of dinotree where the elements are not sorted along each axis, like a KD Tree.
/// For comparison, a normal kd-tree is provided by `NotSorted`. In this tree, the elements are not sorted
/// along an axis at each level. Construction of `NotSorted` is faster than `DinoTree` since it does not have to
/// sort bots that belong to each node along an axis. But most query algorithms can usually take advantage of this
/// extra property.
pub struct NotSorted<'a, A: Axis, T: Aabb>(pub(crate) DinoTree<'a, A, T>);

impl<'a, T: Aabb + Send + Sync> NotSorted<'a, DefaultA, T> {
    pub fn new_par(bots: &'a mut [T]) -> NotSorted<'a, DefaultA, T> {
        DinoTreeBuilder::new(bots).build_not_sorted_par()
    }
}
impl<'a, T: Aabb> NotSorted<'a, DefaultA, T> {
    pub fn new(bots: &'a mut [T]) -> NotSorted<'a, DefaultA, T> {
        DinoTreeBuilder::new(bots).build_not_sorted_seq()
    }
}

impl<'a, A: Axis, T: Aabb + Send + Sync> NotSorted<'a, A, T> {
    pub fn with_axis_par(axis: A, bots: &'a mut [T]) -> NotSorted<'a, A, T> {
        DinoTreeBuilder::with_axis(axis, bots).build_not_sorted_par()
    }
}
impl<'a, A: Axis, T: Aabb> NotSorted<'a, A, T> {
    pub fn with_axis(axis: A, bots: &'a mut [T]) -> NotSorted<'a, A, T> {
        DinoTreeBuilder::with_axis(axis, bots).build_not_sorted_seq()
    }
}

impl<'a,A:Axis,T:Aabb> NotSortedQueries for DinoTree<'a,A,T>{
    type A=A;
    type N=NodeMut<'a,T>;
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

/*
impl<'a, A: Axis, T: Aabb + HasInner + Send + Sync> NotSorted<'a, A, T> {
    pub fn find_intersections_mut_par(
        &mut self,
        func: impl Fn(&mut T::Inner, &mut T::Inner) + Send + Sync + Copy,
    ) {
        query::NotSortedQueryBuilder::new(self.axis(),self.vistr_mut())
            .query_par(move |mut a, mut b| func(a.inner_mut(), b.inner_mut()));
    }
}
*/
impl<'a, A: Axis, T: Aabb> NotSorted<'a, A, T> {
    #[inline(always)]
    pub fn get_height(&self) -> usize {
        self.0.get_height()
    }
}
/*
impl<'a, A: Axis, T: Aabb + HasInner> NotSorted<'a, A, T> {
    pub fn find_intersections_mut(
        &mut self,
        mut func: impl FnMut(&mut T::Inner, &mut T::Inner),
    ) {
        query::NotSortedQueryBuilder::new(self.axis(),self.vistr_mut())
            .query_seq(move |mut a, mut b| func(a.inner_mut(), b.inner_mut()));
    }
}
*/