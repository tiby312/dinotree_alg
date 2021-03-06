//! Provides 2d broadphase collision detection.

mod inner;
mod node_handle;
pub(crate) mod oned;

use self::inner::*;
use self::node_handle::*;
use crate::query::inner_prelude::*;

///Used for the advanced algorithms.
///Trait that user implements to handling aabb collisions.
///The user supplies a struct that implements this trait instead of just a closure
///so that the user may also have the struct implement Splitter.
pub trait ColMulti {
    type T: Aabb;

    fn collide(&mut self, a: PMut<Self::T>, b: PMut<Self::T>);
}

///Naive algorithm.
pub fn query_naive_mut<T: Aabb>(bots: PMut<[T]>, mut func: impl FnMut(PMut<T>, PMut<T>)) {
    tools::for_every_pair(bots, move |a, b| {
        if a.get().intersects_rect(b.get()) {
            func(a, b);
        }
    });
}

///Sweep and prune algorithm.
pub fn query_sweep_mut<T: Aabb>(
    axis: impl Axis,
    bots: &mut [T],
    func: impl FnMut(PMut<T>, PMut<T>),
) {
    ///Sorts the bots.
    #[inline(always)]
    fn sweeper_update<I: Aabb, A: Axis>(axis: A, collision_botids: &mut [I]) {
        let sclosure = move |a: &I, b: &I| -> core::cmp::Ordering {
            let (p1, p2) = (a.get().get_range(axis).start, b.get().get_range(axis).start);
            if p1 > p2 {
                return core::cmp::Ordering::Greater;
            }
            core::cmp::Ordering::Less
        };

        collision_botids.sort_unstable_by(sclosure);
    }

    sweeper_update(axis, bots);

    struct Bl<T: Aabb, F: FnMut(PMut<T>, PMut<T>)> {
        func: F,
        _p: PhantomData<T>,
    }

    impl<T: Aabb, F: FnMut(PMut<T>, PMut<T>)> ColMulti for Bl<T, F> {
        type T = T;
        #[inline(always)]
        fn collide(&mut self, a: PMut<T>, b: PMut<T>) {
            (self.func)(a, b);
        }
    }

    let mut s = oned::Sweeper::new();
    let bots = PMut::new(bots);
    s.find_2d(
        axis,
        bots,
        &mut Bl {
            func,
            _p: PhantomData,
        },
    );
}

///Builder for a query on a NotSorted Dinotree.
pub struct NotSortedQueryBuilder<'a, 'b: 'a, A: Axis, T: Aabb> {
    switch_height: usize,
    tree: &'a mut NotSorted<'b, A, T>,
}

impl<'a, 'b: 'a, A: Axis, T: Aabb + Send + Sync> NotSortedQueryBuilder<'a, 'b, A, T> {
    #[inline(always)]
    pub fn query_par(self, func: impl Fn(PMut<T>, PMut<T>) + Copy + Send + Sync) {
        let b = inner::QueryFn::new(func);
        let mut sweeper = HandleNoSorted::new(b);
        let height = self.tree.get_height();
        let axis = self.tree.axis();
        let oo = self.tree.vistr_mut();
        let switch_height = self.switch_height;

        let par = par::compute_level_switch_sequential(switch_height, height);

        ColFindRecurser::new().recurse_par(axis, par, &mut sweeper, oo, &mut SplitterEmpty);
    }
}

impl<'a, 'b: 'a, A: Axis, T: Aabb> NotSortedQueryBuilder<'a, 'b, A, T> {
    #[inline(always)]
    pub fn new(tree: &'a mut NotSorted<'b, A, T>) -> NotSortedQueryBuilder<'a, 'b, A, T> {
        let switch_height = par::SWITCH_SEQUENTIAL_DEFAULT;
        NotSortedQueryBuilder {
            switch_height,
            tree,
        }
    }

    #[inline(always)]
    pub fn query_with_splitter_seq(
        self,
        func: impl FnMut(PMut<T>, PMut<T>),
        splitter: &mut impl Splitter,
    ) {
        let b = inner::QueryFnMut::new(func);
        let mut sweeper = HandleNoSorted::new(b);

        let axis = self.tree.axis();
        let vistr_mut = self.tree.vistr_mut();
        let dt = vistr_mut;
        ColFindRecurser::new().recurse_seq(axis, &mut sweeper, dt, splitter);
    }

    #[inline(always)]
    pub fn query_seq(self, func: impl FnMut(PMut<T>, PMut<T>)) {
        let b = inner::QueryFnMut::new(func);
        let mut sweeper = HandleNoSorted::new(b);

        let axis = self.tree.axis();
        let vistr_mut = self.tree.vistr_mut();
        let dt = vistr_mut;
        ColFindRecurser::new().recurse_seq(axis, &mut sweeper, dt, &mut SplitterEmpty);
    }
}

///Builder for a query on a DinoTree.
pub struct QueryBuilder<'a, 'b: 'a, A: Axis, T: Aabb> {
    switch_height: usize,
    tree: &'a mut DinoTree<'b, A, T>,
}

impl<'a, 'b: 'a, A: Axis, T: Aabb + Send + Sync> QueryBuilder<'a, 'b, A, T> {
    ///Perform the query in parallel
    #[inline(always)]
    pub fn query_par(self, func: impl Fn(PMut<T>, PMut<T>) + Clone + Send + Sync) {
        let b = inner::QueryFn::new(func);
        let mut sweeper = HandleSorted::new(b);

        let height = self.tree.get_height();
        let switch_height = self.switch_height;
        let axis = self.tree.axis();
        let oo = self.tree.vistr_mut();
        let par = par::compute_level_switch_sequential(switch_height, height);
        ColFindRecurser::new().recurse_par(axis, par, &mut sweeper, oo, &mut SplitterEmpty);
    }

    ///The user has more control using this version of the query.
    ///The splitter will split and add at every level.
    ///The clos will split and add only at levels that are handled in parallel.
    ///This can be useful if the use wants to create a list of colliding pair indicies, but still wants paralleism.
    #[inline(always)]
    pub fn query_splitter_par<C: ColMulti<T = T> + Splitter + Send + Sync>(self, clos: C) -> C {
        let axis = self.tree.axis();
        let height = self.tree.get_height();
        let vistr_mut = self.tree.vistr_mut();

        let par = par::compute_level_switch_sequential(self.switch_height, height);

        let dt = vistr_mut;
        let mut sweeper = HandleSorted::new(clos);
        ColFindRecurser::new().recurse_par(axis, par, &mut sweeper, dt, &mut SplitterEmpty);

        sweeper.func
    }
}

impl<'a, 'b: 'a, A: Axis, T: Aabb> QueryBuilder<'a, 'b, A, T> {
    ///Create the builder.
    #[inline(always)]
    pub fn new(tree: &'a mut DinoTree<'b, A, T>) -> QueryBuilder<'a, 'b, A, T> {
        let switch_height = par::SWITCH_SEQUENTIAL_DEFAULT;
        QueryBuilder {
            switch_height,
            tree,
        }
    }

    ///Choose a custom height at which to switch from parallel to sequential.
    ///If you end up building sequentially, this option is ignored.
    #[inline(always)]
    pub fn with_switch_height(mut self, height: usize) -> Self {
        self.switch_height = height;
        self
    }

    ///Perform the query sequentially.
    #[inline(always)]
    pub fn query_seq(self, func: impl FnMut(PMut<T>, PMut<T>)) {
        let b = inner::QueryFnMut::new(func);
        let mut sweeper = HandleSorted::new(b);
        let mut splitter = SplitterEmpty;

        let axis = self.tree.axis();
        let vistr_mut = self.tree.vistr_mut();
        let dt = vistr_mut;
        ColFindRecurser::new().recurse_seq(axis, &mut sweeper, dt, &mut splitter);
    }

    ///Perform the query sequentially with a splitter.
    #[inline(always)]
    pub fn query_with_splitter_seq(
        self,
        func: impl FnMut(PMut<T>, PMut<T>),
        splitter: &mut impl Splitter,
    ) {
        let b = inner::QueryFnMut::new(func);

        let mut sweeper = HandleSorted::new(b);
        let axis = self.tree.axis();
        let vistr_mut = self.tree.vistr_mut();
        let dt = vistr_mut;
        ColFindRecurser::new().recurse_seq(axis, &mut sweeper, dt, splitter);
    }
}
