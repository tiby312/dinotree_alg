use crate::inner_prelude::*;

#[cfg(test)]
mod tests;

///A version of dinotree that is not lifetimed and uses unsafe{} to own the elements
///that are in its tree (as a self-referential struct). Composed of `(Rect<N>,*mut T)`.
pub mod owned;

///A verion of dinotree where the user can collect and store queries to use later.
pub mod collectable;

pub mod analyze;

///Contains code to write generic code that can be run in parallel, or sequentially. The api is exposed
///in case users find it useful when writing parallel query code to operate on the tree.
pub mod par;

pub(crate) use self::notsorted::NotSorted;
mod notsorted {
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

    impl<'a, A: Axis, T: Aabb + HasInner + Send + Sync> NotSorted<'a, A, T> {
        pub fn find_intersections_mut_par(
            &mut self,
            func: impl Fn(&mut T::Inner, &mut T::Inner) + Send + Sync + Copy,
        ) {
            colfind::NotSortedQueryBuilder::new(self)
                .query_par(move |mut a, mut b| func(a.inner_mut(), b.inner_mut()));
        }
    }

    impl<'a, A: Axis, T: Aabb> NotSorted<'a, A, T> {
        #[inline(always)]
        pub fn axis(&self) -> A {
            self.0.axis()
        }

        #[inline(always)]
        pub fn get_height(&self) -> usize {
            self.0.get_height()
        }

        #[inline(always)]
        pub fn vistr(&self) -> Vistr<NodeMut<'a, T>> {
            self.0.inner.vistr()
        }

        #[inline(always)]
        pub fn vistr_mut(&mut self) -> VistrMut<NodeMut<'a, T>> {
            
            self.0.inner.vistr_mut()
            
        }
    }
    impl<'a, A: Axis, T: Aabb + HasInner> NotSorted<'a, A, T> {
        pub fn find_intersections_mut(
            &mut self,
            mut func: impl FnMut(&mut T::Inner, &mut T::Inner),
        ) {
            colfind::NotSortedQueryBuilder::new(self)
                .query_seq(move |mut a, mut b| func(a.inner_mut(), b.inner_mut()));
        }
    }
}

use crate::query::*;

///The data structure this crate revoles around.
pub struct DinoTree<'a, A: Axis, T: Aabb> {
    axis: A,
    inner: compt::dfs_order::CompleteTreeContainer<NodeMut<'a, T>, compt::dfs_order::PreOrder>,
    bots: PMutPtr<[T]>,
}

///The type of the axis of the first node in the dinotree.
///If it is the y axis, then the first divider will be a horizontal line,
///since it is partioning space based off of objects y value.
pub type DefaultA = YAXIS;
///Constructor of the default axis type. Needed since you cannot construct from type alias's.
pub const fn default_axis() -> YAXIS {
    YAXIS
}

impl<'a, T: Aabb> DinoTree<'a, DefaultA, T> {
    /// # Examples
    ///
    ///```
    ///let mut bots = [axgeom::rect(0,10,0,10)];
    ///let tree = dinotree_alg::DinoTree::new(&mut bots);
    ///
    ///```
    pub fn new(bots: &'a mut [T]) -> DinoTree<'a, DefaultA, T> {
        DinoTreeBuilder::new(bots).build_seq()
    }
}

impl<'a, T: Aabb + Send + Sync> DinoTree<'a, DefaultA, T> {
    /// # Examples
    ///
    ///```
    ///let mut bots = [axgeom::rect(0,10,0,10)];
    ///let tree = dinotree_alg::DinoTree::new_par(&mut bots);
    ///
    ///```
    pub fn new_par(bots: &'a mut [T]) -> DinoTree<'a, DefaultA, T> {
        DinoTreeBuilder::new(bots).build_par()
    }
}

impl<'a, A: Axis, T: Aabb> DinoTree<'a, A, T> {
    /// # Examples
    ///
    ///```
    ///let mut bots = [axgeom::rect(0,10,0,10)];
    ///let tree = dinotree_alg::DinoTree::with_axis(axgeom::XAXIS,&mut bots);
    ///
    ///```
    pub fn with_axis(axis: A, bots: &'a mut [T]) -> DinoTree<'a, A, T> {
        DinoTreeBuilder::with_axis(axis, bots).build_seq()
    }
}

impl<'a, A: Axis, T: Aabb + Send + Sync> DinoTree<'a, A, T> {
    /// # Examples
    ///
    ///```
    ///let mut bots = [axgeom::rect(0,10,0,10)];
    ///let tree = dinotree_alg::DinoTree::with_axis(axgeom::XAXIS,&mut bots);
    ///
    ///```
    pub fn with_axis_par(axis: A, bots: &'a mut [T]) -> DinoTree<'a, A, T> {
        DinoTreeBuilder::with_axis(axis, bots).build_par()
    }
}

impl<'a, A: Axis, T: Aabb + HasInner + Send + Sync> DinoTree<'a, A, T> {}

impl<'a, A: Axis, T: Aabb + HasInner + Send + Sync> DinoTree<'a, A, T> {
    /// Find all intersections in parallel
    ///
    /// # Examples
    ///
    ///```
    ///use dinotree_alg::*;
    ///let mut bots = [bbox(axgeom::rect(0,10,0,10),0u8),bbox(axgeom::rect(5,15,5,15),0u8)];
    ///let mut tree = DinoTree::new(&mut bots);
    ///tree.find_intersections_mut_par(|a,b|{
    ///    *a+=1;
    ///    *b+=1;
    ///});
    ///
    ///assert_eq!(bots[0].inner,1);
    ///assert_eq!(bots[1].inner,1);
    ///```
    pub fn find_intersections_mut_par(
        &mut self,
        func: impl Fn(&mut T::Inner, &mut T::Inner) + Send + Sync + Copy,
    ) {
        query::colfind::QueryBuilder::new(self)
            .query_par(move |mut a, mut b| func(a.inner_mut(), b.inner_mut()));
    }

    /// Allows the user to potentially collect some aspect of every intersection in parallel.
    ///
    /// # Examples
    ///
    ///```
    ///use dinotree_alg::*;
    ///let mut bots = [bbox(axgeom::rect(0,10,0,10),0u8),bbox(axgeom::rect(5,15,5,15),1u8)];
    ///let mut tree = DinoTree::new(&mut bots);
    ///let intersections=tree.find_intersections_par_ext(
    ///     |_|Vec::new(),              //Start a new thread
    ///     |a,mut b|a.append(&mut b),  //Combine two threads
    ///     |v,a,b|v.push((*a,*b)),     //What to do for each intersection for a thread.
    ///     Vec::new()                  //Starting thread
    ///);
    ///
    ///assert_eq!(intersections.len(),1);
    ///```
    pub fn find_intersections_par_ext<B: Send + Sync>(
        &mut self,
        split: impl Fn(&mut B) -> B + Send + Sync + Copy,
        fold: impl Fn(&mut B, B) + Send + Sync + Copy,
        collision: impl Fn(&mut B, &mut T::Inner, &mut T::Inner) + Send + Sync + Copy,
        acc: B,
    ) -> B {
        struct Foo<T, A, B, C, D> {
            _p: PhantomData<T>,
            acc: A,
            split: B,
            fold: C,
            collision: D,
        }

        impl<T: Aabb + HasInner, A, B, C, D: Fn(&mut A, &mut T::Inner, &mut T::Inner)> ColMulti
            for Foo<T, A, B, C, D>
        {
            type T = T;
            fn collide(&mut self, mut a: PMut<Self::T>, mut b: PMut<Self::T>) {
                (self.collision)(&mut self.acc, a.inner_mut(), b.inner_mut())
            }
        }
        impl<T, A, B: Fn(&mut A) -> A + Copy, C: Fn(&mut A, A) + Copy, D: Copy> Splitter
            for Foo<T, A, B, C, D>
        {
            fn div(&mut self) -> Self {
                let acc = (self.split)(&mut self.acc);
                Foo {
                    _p: PhantomData,
                    acc,
                    split: self.split,
                    fold: self.fold,
                    collision: self.collision,
                }
            }

            fn add(&mut self, b: Self) {
                (self.fold)(&mut self.acc, b.acc)
            }

            fn node_start(&mut self) {}

            fn node_end(&mut self) {}
        }

        let foo = Foo {
            _p: PhantomData,
            acc,
            split,
            fold,
            collision,
        };

        let foo = query::colfind::QueryBuilder::new(self).query_splitter_par(foo);
        foo.acc
    }
}

impl<A: Axis, T: Aabb + HasInner + Send + Sync> DinoTree<'_, A, T> {
    #[cfg(feature = "nbody")]
    pub fn nbody_mut<X: query::nbody::NodeMassTrait<Num = T::Num, Item = T> + Send + Sync>(
        &mut self,
        ncontext: &X,
        rect: Rect<T::Num>,
    ) where
        X::No: Send,
    {
        query::nbody::nbody(self, ncontext, rect)
    }
}

impl<'a, A: Axis, T: Aabb + HasInner + Send + Sync> DinoTree<'a, A, T> {
    #[cfg(feature = "nbody")]
    pub fn nbody_mut_par<X: query::nbody::NodeMassTrait<Num = T::Num, Item = T> + Sync + Send>(
        &mut self,
        ncontext: &X,
        rect: Rect<T::Num>,
    ) where
        X::No: Send,
    {
        query::nbody::nbody_par(self, ncontext, rect)
    }
}

impl<'a, A: Axis, T: Aabb + HasInner> DinoTree<'a, A, T> {
    /// # Examples
    ///
    ///```
    ///use dinotree_alg::*;
    ///use axgeom::*;
    ///
    ///let border = rect(0,100,0,100);
    ///
    ///let mut bots = [bbox(rect(0,10,0,10),vec2(5,5)),
    ///                bbox(rect(2,5,2,5),vec2(4,4)),
    ///                bbox(rect(4,10,4,10),vec2(5,5))];
    ///
    ///let mut bots_copy=bots.clone();
    ///let mut tree = DinoTree::new(&mut bots);
    ///let ray=ray(vec2(5,-5),vec2(0,1));
    ///let mut counter =0;
    ///let res = tree.raycast_mut(
    ///     ray,&mut counter,
    ///     |c,ray,r|{*c+=1;ray.cast_to_rect(r)},
    ///     |c,ray,t|{*c+=1;ray.inner_as::<f32>().cast_to_circle(t.inner.inner_as(),5.).map(|a|a as i32)},   //Do more fine-grained checking here.
    ///     border);
    ///
    ///let (bots,dis)=res.unwrap();
    ///assert_eq!(dis,4);
    ///assert_eq!(bots.len(),1);
    ///assert_eq!(bots[0],&vec2(4,4));
    ///assert_eq!(counter,3);
    ///```
    #[must_use]
    pub fn raycast_mut<Acc>(
        &mut self,
        ray: axgeom::Ray<T::Num>,
        start: &mut Acc,
        broad: impl FnMut(&mut Acc, &Ray<T::Num>, &Rect<T::Num>) -> CastResult<T::Num>,
        fine: impl FnMut(&mut Acc, &Ray<T::Num>, &T) -> CastResult<T::Num>,
        border: Rect<T::Num>,
    ) -> raycast::RayCastResult<T::Inner, T::Num> {
        let mut rtrait = raycast::RayCastClosure {
            a: start,
            broad,
            fine,
            _p: PhantomData,
        };
        raycast::raycast_mut(self, border, ray, &mut rtrait)
    }

    /// # Examples
    ///
    ///```
    ///use dinotree_alg::*;
    ///use axgeom::*;
    ///let border = rect(0,100,0,100);
    ///
    ///let mut bots = [bbox(rect(0,10,0,10),vec2(0,0)),
    ///                bbox(rect(2,5,2,5),vec2(0,5)),
    ///                bbox(rect(4,10,4,10),vec2(3,3))];
    ///
    ///let mut bots_copy=bots.clone();
    ///let mut tree = DinoTree::new(&mut bots);
    ///
    ///let mut counter = 0;
    ///let res = tree.k_nearest_mut(
    ///     vec2(0,0),
    ///     2,
    ///     &mut counter,
    ///     |c,p,r|{*c+=1;r.distance_squared_to_point(p).unwrap_or(0)},
    ///     |c,p,t|{*c+=1;t.inner.distance_squared_to_point(p)},    //Do more fine-grained checking here.
    ///     border);
    ///
    ///assert_eq!(res.len(),2);
    ///assert_eq!(*res[0].bot,bots_copy[0].inner);
    ///assert_eq!(*res[1].bot,bots_copy[2].inner);
    ///assert_eq!(counter,3);
    ///```
    #[must_use]
    pub fn k_nearest_mut<Acc>(
        &mut self,
        point: Vec2<T::Num>,
        num: usize,
        start: &mut Acc,
        broad: impl FnMut(&mut Acc, Vec2<T::Num>, &Rect<T::Num>) -> T::Num,
        fine: impl FnMut(&mut Acc, Vec2<T::Num>, &T) -> T::Num,
        border: Rect<T::Num>,
    ) -> Vec<k_nearest::KnearestResult<T::Inner, T::Num>> {
        let mut foo = k_nearest::KnearestClosure {
            acc: start,
            broad,
            fine,
            _p: PhantomData,
        };
        k_nearest::k_nearest_mut(self, point, num, &mut foo, border)
    }

    /// # Examples
    ///
    ///```
    ///use dinotree_alg::*;
    ///let mut bots1 = [bbox(axgeom::rect(0,10,0,10),0u8)];
    ///let mut bots2 = [bbox(axgeom::rect(5,15,5,15),0u8)];
    ///let mut tree = DinoTree::new(&mut bots1);
    ///
    ///tree.intersect_with_mut(&mut bots2,|a,b|{
    ///    *a+=1;
    ///    *b+=2;    
    ///});
    ///
    ///assert_eq!(bots1[0].inner,1);
    ///assert_eq!(bots2[0].inner,2);
    ///```
    pub fn intersect_with_mut<X: Aabb<Num = T::Num> + HasInner>(
        &mut self,
        other: &mut [X],
        func: impl Fn(&mut T::Inner, &mut X::Inner),
    ) {
        intersect_with::intersect_with_mut(self, other, move |a, b| {
            (func)(a.into_inner(), b.into_inner())
        })
    }

    /// # Examples
    ///
    ///```
    ///use dinotree_alg::*;
    ///let mut bots = [bbox(axgeom::rect(0,10,0,10),0u8)];
    ///let mut tree = DinoTree::new(&mut bots);
    ///tree.for_all_not_in_rect_mut(&axgeom::rect(10,20,10,20),|a|{
    ///    *a+=1;    
    ///});
    ///
    ///assert_eq!(bots[0].inner,1);
    ///
    ///```
    pub fn for_all_not_in_rect_mut<'b>(
        &'b mut self,
        rect: &Rect<T::Num>,
        mut func: impl FnMut(&'b mut T::Inner),
    ) {
        rect::for_all_not_in_rect_mut(self, rect, move |a| (func)(a.into_inner()));
    }

    /// # Examples
    ///
    ///```
    ///use dinotree_alg::*;
    ///let mut bots = [bbox(axgeom::rect(0,10,0,10),0u8)];
    ///let mut tree = DinoTree::new(&mut bots);
    ///tree.for_all_intersect_rect_mut(&axgeom::rect(9,20,9,20),|a|{
    ///    *a+=1;    
    ///});
    ///
    ///assert_eq!(bots[0].inner,1);
    ///
    ///```
    pub fn for_all_intersect_rect_mut<'b>(
        &'b mut self,
        rect: &Rect<T::Num>,
        mut func: impl FnMut(&'b mut T::Inner),
    ) {
        rect::for_all_intersect_rect_mut(self, rect, move |a| (func)(a.into_inner()));
    }

    /// # Examples
    ///
    ///```
    ///use dinotree_alg::*;
    ///let mut bots = [bbox(axgeom::rect(0,10,0,10),0u8)];
    ///let mut tree = DinoTree::new(&mut bots);
    ///tree.for_all_in_rect_mut(&axgeom::rect(0,10,0,10),|a|{
    ///    *a+=1;    
    ///});
    ///
    ///assert_eq!(bots[0].inner,1);
    ///
    ///```
    pub fn for_all_in_rect_mut<'b>(
        &'b mut self,
        rect: &Rect<T::Num>,
        mut func: impl FnMut(&'b mut T::Inner),
    ) {
        rect::for_all_in_rect_mut(self, rect, move |a| (func)(a.into_inner()));
    }

    /// Find all aabb intersections
    /// # Examples
    ///
    ///```
    ///use dinotree_alg::*;
    ///let mut bots = [bbox(axgeom::rect(0,10,0,10),0u8),bbox(axgeom::rect(5,15,5,15),0u8)];
    ///let mut tree = DinoTree::new(&mut bots);
    ///tree.find_intersections_mut(|a,b|{
    ///    *a+=1;
    ///    *b+=1;
    ///});
    ///
    ///assert_eq!(bots[0].inner,1);
    ///assert_eq!(bots[1].inner,1);
    ///```
    pub fn find_intersections_mut(&mut self, mut func: impl FnMut(&mut T::Inner, &mut T::Inner)) {
        colfind::QueryBuilder::new(self)
            .query_seq(move |a, b| func(a.into_inner(), b.into_inner()));
    }

    /// Find all aabb intersections and return a PMut<T> of it. Unlike the regular `find_intersections_mut`, this allows the
    /// user to access a read only reference of the AABB.
    ///
    /// # Examples
    ///
    ///```
    ///use dinotree_alg::*;
    ///let mut bots = [bbox(axgeom::rect(0,10,0,10),0u8),bbox(axgeom::rect(5,15,5,15),0u8)];
    ///let mut tree = DinoTree::new(&mut bots);
    ///tree.find_intersections_pmut(|mut a,mut b|{
    ///    *a.inner_mut()+=1;
    ///    *b.inner_mut()+=1;
    ///});
    ///
    ///assert_eq!(bots[0].inner,1);
    ///assert_eq!(bots[1].inner,1);
    ///```
    pub fn find_intersections_pmut(&mut self, mut func: impl FnMut(PMut<T>, PMut<T>)) {
        colfind::QueryBuilder::new(self).query_seq(move |a, b| func(a, b));
    }
}

impl<'a, A: Axis, T: Aabb> DinoTree<'a, A, T> {
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

    /// # Examples
    ///
    ///```
    ///use dinotree_alg::*;
    ///let mut bots = [axgeom::rect(0,10,0,10)];
    ///let mut tree = DinoTree::new(&mut bots);
    ///
    ///use axgeom::Axis;
    ///assert!(tree.axis().is_equal_to(default_axis()));
    ///```
    #[must_use]
    #[inline(always)]
    pub fn axis(&self) -> A {
        self.axis
    }

    /// # Examples
    ///
    ///```
    ///use dinotree_alg::*;
    ///let mut bots = [bbox(axgeom::rect(0,10,0,10),0)];
    ///let mut tree = DinoTree::new(&mut bots);
    ///
    ///use compt::Visitor;
    ///for mut b in tree.vistr_mut().dfs_preorder_iter().flat_map(|n|n.get_mut().bots.iter_mut()){
    ///    *b.inner_mut()+=1;    
    ///}
    ///assert_eq!(bots[0].inner,1);
    ///```
    #[must_use]
    pub fn vistr_mut(&mut self) -> VistrMut<NodeMut<'a, T>> {
        self.inner.vistr_mut()
    }

    /// # Examples
    ///
    ///```
    ///use dinotree_alg::*;
    ///let mut bots = [axgeom::rect(0,10,0,10)];
    ///let mut tree = DinoTree::new(&mut bots);
    ///
    ///use compt::Visitor;
    ///let mut test = Vec::new();
    ///for b in tree.vistr().dfs_preorder_iter().flat_map(|n|n.get().bots.iter()){
    ///    test.push(b);
    ///}
    ///assert_eq!(test[0],&axgeom::rect(0,10,0,10));
    ///```
    #[must_use]
    pub fn vistr(&self) -> Vistr<NodeMut<'a, T>> {
        self.inner.vistr()
    }

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

impl<'a, A: Axis, T: Aabb> DinoTree<'a, A, T> {
    /// # Examples
    ///
    /// ```
    /// use dinotree_alg::*;
    /// use axgeom::*;
    ///
    /// struct Drawer;
    /// impl dinotree_alg::query::DividerDrawer for Drawer{
    ///     type N=i32;
    ///     fn draw_divider<A:Axis>(
    ///             &mut self,
    ///             axis:A,
    ///             div:Self::N,
    ///             cont:[Self::N;2],
    ///             length:[Self::N;2],
    ///             depth:usize)
    ///     {
    ///         if axis.is_xaxis(){
    ///             //draw vertical line
    ///         }else{
    ///             //draw horizontal line
    ///         }
    ///     }
    /// }
    ///
    /// let border=rect(0,100,0,100);
    /// let mut bots =[rect(0,10,0,10)];
    /// let tree=DinoTree::new(&mut bots);
    /// tree.draw(&mut Drawer,&border);
    /// ```
    ///
    pub fn draw(&self, drawer: &mut impl graphics::DividerDrawer<N = T::Num>, rect: &Rect<T::Num>) {
        graphics::draw(self, drawer, rect)
    }

    /// # Examples
    ///
    ///```
    ///use dinotree_alg::*;
    ///let mut bots1 = [bbox(axgeom::rect(0,10,0,10),0u8)];
    ///let mut tree = DinoTree::new(&mut bots1);
    ///let mut multi = tree.multi_rect();
    ///
    ///multi.for_all_in_rect_mut(axgeom::rect(0,10,0,10),|a|{}).unwrap();
    ///let res = multi.for_all_in_rect_mut(axgeom::rect(5,15,5,15),|a|{});
    ///assert_eq!(res,Err(dinotree_alg::query::RectIntersectErr));
    ///```
    #[must_use]
    pub fn multi_rect<'b>(&'b mut self) -> rect::MultiRectMut<'b, 'a, A, T> {
        rect::MultiRectMut::new(self)
    }
    /// # Examples
    ///
    ///```
    ///use dinotree_alg::*;
    ///let mut bots = [axgeom::rect(0,10,0,10),axgeom::rect(20,30,20,30)];
    ///let mut tree = DinoTree::new(&mut bots);
    ///let mut test = Vec::new();
    ///tree.for_all_intersect_rect(&axgeom::rect(9,20,9,20),|a|{
    ///    test.push(a);
    ///});
    ///
    ///assert_eq!(test[0],&axgeom::rect(0,10,0,10));
    ///
    ///```
    pub fn for_all_intersect_rect<'b>(&'b self, rect: &Rect<T::Num>, func: impl FnMut(&'b T)) {
        rect::for_all_intersect_rect(self, rect, func);
    }

    /// # Examples
    ///
    ///```
    ///use dinotree_alg::*;
    ///let mut bots = [axgeom::rect(0,10,0,10),axgeom::rect(20,30,20,30)];
    ///let mut tree = DinoTree::new(&mut bots);
    ///let mut test = Vec::new();
    ///tree.for_all_in_rect(&axgeom::rect(0,20,0,20),|a|{
    ///    test.push(a);
    ///});
    ///
    ///assert_eq!(test[0],&axgeom::rect(0,10,0,10));
    ///
    pub fn for_all_in_rect<'b>(&'b self, rect: &Rect<T::Num>, func: impl FnMut(&'b T)) {
        rect::for_all_in_rect(self, rect, func);
    }
}

use self::builder::DinoTreeBuilder;
mod builder {
    use super::*;

    ///Builder pattern for dinotree.
    pub struct DinoTreeBuilder<'a, A: Axis, T> {
        axis: A,
        bots: &'a mut [T],
        rebal_strat: BinStrat,
        height: usize,
        height_switch_seq: usize,
    }

    impl<'a, A: Axis, T: Aabb + Send + Sync> DinoTreeBuilder<'a, A, T> {
        ///Build not sorted in parallel
        pub fn build_not_sorted_par(&mut self) -> NotSorted<'a, A, T> {
            let b = PMut::new(self.bots).as_ptr();
            let mut bots: &mut [T] = &mut [];
            core::mem::swap(&mut bots, &mut self.bots);
            let dlevel = par::compute_level_switch_sequential(self.height_switch_seq, self.height);
            let inner = create_tree_par(
                self.axis,
                dlevel,
                bots,
                NoSorter,
                &mut SplitterEmpty,
                self.height,
                self.rebal_strat,
            );
            NotSorted(DinoTree {
                axis: self.axis,
                inner,
                bots: b,
            })
        }

        ///Build in parallel
        pub fn build_par(&mut self) -> DinoTree<'a, A, T> {
            let b = PMut::new(self.bots).as_ptr();
            let mut bots: &mut [T] = &mut [];
            core::mem::swap(&mut bots, &mut self.bots);
            let dlevel = par::compute_level_switch_sequential(self.height_switch_seq, self.height);
            let inner = create_tree_par(
                self.axis,
                dlevel,
                bots,
                DefaultSorter,
                &mut SplitterEmpty,
                self.height,
                self.rebal_strat,
            );
            DinoTree {
                axis: self.axis,
                inner,
                bots: b,
            }
        }
    }

    impl<'a, T: Aabb> DinoTreeBuilder<'a, DefaultA, T> {
        ///Create a new builder with a slice of elements that implement `Aabb`.
        pub fn new(bots: &'a mut [T]) -> DinoTreeBuilder<'a, DefaultA, T> {
            Self::with_axis(default_axis(), bots)
        }
    }

    impl<'a, A: Axis, T: Aabb> DinoTreeBuilder<'a, A, T> {
        ///Create a new builder with a slice of elements that implement `Aabb`.
        pub fn with_axis(axis: A, bots: &'a mut [T]) -> DinoTreeBuilder<'a, A, T> {
            let rebal_strat = BinStrat::NotChecked;

            //we want each node to have space for around num_per_node bots.
            //there are 2^h nodes.
            //2^h*200>=num_bots.  Solve for h s.t. h is an integer.

            //Make this number too small, and the tree will have too many levels,
            //and too much time will be spent recursing.
            //Make this number too high, and you will lose the properties of a tree,
            //and you will end up with just sweep and prune.
            //This number was chosen emprically from running the dinotree_alg_data project,
            //on two different machines.
            let height = compute_tree_height_heuristic(bots.len(), DEFAULT_NUMBER_ELEM_PER_NODE);

            let height_switch_seq = par::SWITCH_SEQUENTIAL_DEFAULT;

            DinoTreeBuilder {
                axis,
                bots,
                rebal_strat,
                height,
                height_switch_seq,
            }
        }

        ///Build not sorted sequentially
        pub fn build_not_sorted_seq(&mut self) -> NotSorted<'a, A, T> {
            let b = PMut::new(self.bots).as_ptr();
            let mut bots: &mut [T] = &mut [];
            core::mem::swap(&mut bots, &mut self.bots);
            let inner = create_tree_seq(
                self.axis,
                bots,
                NoSorter,
                &mut SplitterEmpty,
                self.height,
                self.rebal_strat,
            );
            NotSorted(DinoTree {
                axis: self.axis,
                inner,
                bots: b,
            })
        }

        ///Build sequentially
        pub fn build_seq(&mut self) -> DinoTree<'a, A, T> {
            let b = PMut::new(self.bots).as_ptr();
            let mut bots: &mut [T] = &mut [];
            core::mem::swap(&mut bots, &mut self.bots);
            let inner = create_tree_seq(
                self.axis,
                bots,
                DefaultSorter,
                &mut SplitterEmpty,
                self.height,
                self.rebal_strat,
            );
            DinoTree {
                axis: self.axis,
                inner,
                bots: b,
            }
        }

        #[inline(always)]
        pub fn with_bin_strat(&mut self, strat: BinStrat) -> &mut Self {
            self.rebal_strat = strat;
            self
        }

        #[inline(always)]
        pub fn with_height(&mut self, height: usize) -> &mut Self {
            self.height = height;
            self
            //TODO test corner cases of this
        }

        ///Choose the height at which to switch from parallel to sequential.
        ///If you end up building sequentially, this argument is ignored.
        #[inline(always)]
        pub fn with_height_switch_seq(&mut self, height: usize) -> &mut Self {
            self.height_switch_seq = height;
            self
        }

        ///Build with a Splitter.
        pub fn build_with_splitter_seq<S: Splitter>(
            &mut self,
            splitter: &mut S,
        ) -> DinoTree<'a, A, T> {
            let b = PMut::new(self.bots).as_ptr();
            let mut bots: &mut [T] = &mut [];
            core::mem::swap(&mut bots, &mut self.bots);
            let inner = create_tree_seq(
                self.axis,
                bots,
                DefaultSorter,
                splitter,
                self.height,
                self.rebal_strat,
            );
            DinoTree {
                axis: self.axis,
                inner,
                bots: b,
            }
        }
    }
}

pub(crate) use self::node::*;
///Contains node-level building block structs and visitors used for a DinoTree.
pub mod node {
    use super::*;

    ///When we traverse the tree in read-only mode, we can simply return a reference to each node.
    ///We don't need to protect the user from only mutating parts of the BBox's since they can't
    ///change anything.
    pub type Vistr<'a, N> = compt::dfs_order::Vistr<'a, N, compt::dfs_order::PreOrder>;

    pub type VistrMut<'a,N>=compt::dfs_order::VistrMut<'a,N,compt::dfs_order::PreOrder>;
    /*
    mod vistr_mut {
        use crate::inner_prelude::*;

        //Cannot use since we need create_wrap_mut()
        //We must create our own new type.
        //pub type VistrMut<'a,N> = compt::MapStruct<compt::dfs_order::VistrMut<'a,N,compt::dfs_order::PreOrder>,Foo<'a,N>>;

        /// Tree Iterator that returns a protected mutable reference to each node.
        #[repr(transparent)]
        pub struct VistrMut<'a, N> {
            pub(crate) inner: compt::dfs_order::VistrMut<'a, N, compt::dfs_order::PreOrder>,
        }

        impl<'a, N> VistrMut<'a, N> {
            ///It is safe to borrow the iterator and then produce mutable references from that
            ///as long as by the time the borrow ends, all the produced references also go away.
            #[inline(always)]
            pub fn create_wrap_mut(&mut self) -> VistrMut<N> {
                VistrMut {
                    inner: self.inner.create_wrap_mut(),
                }
            }

            #[inline(always)]
            pub fn as_slice_mut(&mut self) -> PMut<[N]> {
                PMut::new(self.inner.as_slice_mut())
            }
        }

        impl<'a, N> core::ops::Deref for VistrMut<'a, N> {
            type Target = Vistr<'a, N>;

            #[inline(always)]
            fn deref(&self) -> &Vistr<'a, N> {
                unsafe { &*(self as *const VistrMut<_> as *const Vistr<_>) }
            }
        }

        unsafe impl<'a, N> compt::FixedDepthVisitor for VistrMut<'a, N> {}

        impl<'a, N> Visitor for VistrMut<'a, N> {
            type Item = PMut<'a, N>;

            #[inline(always)]
            fn next(self) -> (Self::Item, Option<[Self; 2]>) {
                let (nn, rest) = self.inner.next();

                let k = match rest {
                    Some([left, right]) => {
                        Some([VistrMut { inner: left }, VistrMut { inner: right }])
                    }
                    None => None,
                };
                (PMut::new(nn), k)
            }

            #[inline(always)]
            fn level_remaining_hint(&self) -> (usize, Option<usize>) {
                self.inner.level_remaining_hint()
            }

            #[inline(always)]
            fn dfs_preorder(self, mut func: impl FnMut(Self::Item)) {
                self.inner.dfs_preorder(move |a| func(PMut::new(a)));
            }
        }
    }
    pub use vistr_mut::VistrMut;
    */

    ///Expose a node trait api to hide the lifetime of NodeMut.
    ///This way query algorithms do not need to worry about this lifetime.
    pub trait Node {
        type T: Aabb<Num = Self::Num>;
        type Num: Num;
        fn get(&self) -> NodeRef<Self::T>;
        fn get_mut(&mut self) -> NodeRefMut<Self::T>;
    }

    impl<'a, T: Aabb> Node for NodeMut<'a, T> {
        type T = T;
        type Num = T::Num;
        fn get(&self) -> NodeRef<Self::T> {
            //TODO point as struct impl
            NodeRef {
                bots: self.range.as_ref(),
                cont: &self.cont,
                div: &self.div,
            }
        }
        fn get_mut(&mut self) -> NodeRefMut<Self::T> {
            NodeRefMut {
                bots: self.range.as_mut(),
                cont: &self.cont,
                div: &self.div,
            }
        }
    }

    ///A lifetimed node in a dinotree.
    pub struct NodeMut<'a, T: Aabb> {
        pub(crate) range: PMut<'a, [T]>,

        //range is empty iff cont is none.
        pub(crate) cont: Option<axgeom::Range<T::Num>>,
        //for non leafs:
        //  div is some iff mid is nonempty.
        //  div is none iff mid is empty.
        //for leafs:
        //  div is none
        pub(crate) div: Option<T::Num>,
    }

    impl<'a, T: Aabb> NodeMut<'a, T> {
        pub fn get(&self) -> NodeRef<T> {
            NodeRef {
                bots: self.range.as_ref(),
                cont: &self.cont,
                div: &self.div,
            }
        }
        pub fn get_mut(&mut self) -> NodeRefMut<T> {
            NodeRefMut {
                bots: self.range.as_mut(),
                cont: &self.cont,
                div: &self.div,
            }
        }
    }

    ///Mutable reference to a node in the dinotree.
    pub struct NodeRefMut<'a, T: Aabb> {
        ///The bots that belong to this node.
        pub bots: PMut<'a, [T]>,

        ///Is None iff bots is empty.
        pub cont: &'a Option<axgeom::Range<T::Num>>,

        ///Is None if node is a leaf, or there are no bots in this node or in any decendants.
        pub div: &'a Option<T::Num>,
    }

    ///Reference to a node in the dinotree.
    pub struct NodeRef<'a, T: Aabb> {
        ///The bots that belong to this node.
        pub bots: &'a [T],

        ///Is None iff bots is empty.
        pub cont: &'a Option<axgeom::Range<T::Num>>,

        ///Is None if node is a leaf, or there are no bots in this node or in any decendants.
        pub div: &'a Option<T::Num>,
    }
}

fn create_tree_seq<'a, A: Axis, T: Aabb, K: Splitter>(
    div_axis: A,
    rest: &'a mut [T],
    sorter: impl Sorter,
    splitter: &mut K,
    height: usize,
    binstrat: BinStrat,
) -> compt::dfs_order::CompleteTreeContainer<NodeMut<'a, T>, compt::dfs_order::PreOrder> {
    let num_bots = rest.len();

    let mut nodes = Vec::with_capacity(tree::nodes_left(0, height));

    let r = Recurser {
        height,
        binstrat,
        sorter,
        _p: PhantomData,
    };
    r.recurse_preorder_seq(div_axis, rest, &mut nodes, splitter, 0);

    let tree = compt::dfs_order::CompleteTreeContainer::from_preorder(nodes).unwrap();

    let k = tree
        .get_nodes()
        .iter()
        .fold(0, move |acc, a| acc + a.range.len());
    debug_assert_eq!(k, num_bots);

    tree
}

fn create_tree_par<
    'a,
    A: Axis,
    JJ: par::Joiner,
    T: Aabb + Send + Sync,
    K: Splitter + Send + Sync,
>(
    div_axis: A,
    dlevel: JJ,
    rest: &'a mut [T],
    sorter: impl Sorter,
    splitter: &mut K,
    height: usize,
    binstrat: BinStrat,
) -> compt::dfs_order::CompleteTreeContainer<NodeMut<'a, T>, compt::dfs_order::PreOrder> {
    let num_bots = rest.len();

    let mut nodes = Vec::with_capacity(tree::nodes_left(0, height));

    let r = Recurser {
        height,
        binstrat,
        sorter,
        _p: PhantomData,
    };
    r.recurse_preorder(div_axis, dlevel, rest, &mut nodes, splitter, 0);

    let tree = compt::dfs_order::CompleteTreeContainer::from_preorder(nodes).unwrap();

    let k = tree
        .get_nodes()
        .iter()
        .fold(0, move |acc, a| acc + a.range.len());
    debug_assert_eq!(k, num_bots);

    tree
}

struct Recurser<'a, T: Aabb, K: Splitter, S: Sorter> {
    height: usize,
    binstrat: BinStrat,
    sorter: S,
    _p: PhantomData<(K, &'a T)>,
}

impl<'a, T: Aabb, K: Splitter, S: Sorter> Recurser<'a, T, K, S> {
    fn create_leaf<A: Axis>(&self, axis: A, rest: &'a mut [T]) -> NodeMut<'a, T> {
        self.sorter.sort(axis.next(), rest);

        let cont = create_cont(axis, rest);

        NodeMut {
            range: PMut::new(rest),
            cont,
            div: None,
        }
    }

    fn create_non_leaf<A: Axis>(
        &self,
        axis: A,
        rest: &'a mut [T],
    ) -> (NodeMut<'a, T>, &'a mut [T], &'a mut [T]) {
        match construct_non_leaf(self.binstrat, self.sorter, axis, rest) {
            ConstructResult::NonEmpty {
                cont,
                div,
                mid,
                left,
                right,
            } => (
                NodeMut {
                    range: PMut::new(mid),
                    cont,
                    div: Some(div),
                },
                left,
                right,
            ),
            ConstructResult::Empty(empty) => {
                //let (a,empty) = tools::duplicate_empty_slice(empty);
                //let (b,c) = tools::duplicate_empty_slice(empty);
                let node = NodeMut {
                    range: PMut::new(empty),
                    cont: None,
                    div: None,
                };

                (node, &mut [], &mut [])
            }
        }
    }

    fn recurse_preorder_seq<A: Axis>(
        &self,
        axis: A,
        rest: &'a mut [T],
        nodes: &mut Vec<NodeMut<'a, T>>,
        splitter: &mut K,
        depth: usize,
    ) {
        splitter.node_start();

        if depth < self.height - 1 {
            let (node, left, right) = self.create_non_leaf(axis, rest);
            nodes.push(node);

            let mut splitter2 = splitter.div();

            self.recurse_preorder_seq(axis.next(), left, nodes, splitter, depth + 1);
            self.recurse_preorder_seq(axis.next(), right, nodes, &mut splitter2, depth + 1);

            splitter.add(splitter2);
        } else {
            let node = self.create_leaf(axis, rest);
            nodes.push(node);
            splitter.node_end();
        }
    }
}
impl<'a, T: Aabb + Send + Sync, K: Splitter + Send + Sync, S: Sorter> Recurser<'a, T, K, S> {
    fn recurse_preorder<A: Axis, JJ: par::Joiner>(
        &self,
        axis: A,
        dlevel: JJ,
        rest: &'a mut [T],
        nodes: &mut Vec<NodeMut<'a, T>>,
        splitter: &mut K,
        depth: usize,
    ) {
        splitter.node_start();

        if depth < self.height - 1 {
            let (node, left, right) = self.create_non_leaf(axis, rest);

            nodes.push(node);

            let mut splitter2 = splitter.div();

            let splitter = match dlevel.next() {
                par::ParResult::Parallel([dleft, dright]) => {
                    let splitter2 = &mut splitter2;

                    //dbg!("PAR SPLIT");

                    let ((splitter, nodes), mut nodes2) = rayon::join(
                        move || {
                            self.recurse_preorder(
                                axis.next(),
                                dleft,
                                left,
                                nodes,
                                splitter,
                                depth + 1,
                            );
                            (splitter, nodes)
                        },
                        move || {
                            let mut nodes2: Vec<_> =
                                Vec::with_capacity(nodes_left(depth, self.height));
                            self.recurse_preorder(
                                axis.next(),
                                dright,
                                right,
                                &mut nodes2,
                                splitter2,
                                depth + 1,
                            );
                            nodes2
                        },
                    );

                    nodes.append(&mut nodes2);
                    splitter
                }
                par::ParResult::Sequential(_) => {
                    //dbg!("SEQ SPLIT");

                    self.recurse_preorder_seq(axis.next(), left, nodes, splitter, depth + 1);
                    self.recurse_preorder_seq(axis.next(), right, nodes, &mut splitter2, depth + 1);
                    splitter
                }
            };

            splitter.add(splitter2);
        } else {
            let node = self.create_leaf(axis, rest);
            nodes.push(node);
            splitter.node_end();
        }
    }
}

#[bench]
#[cfg(all(feature = "unstable", test))]
fn bench_cont(b: &mut test::Bencher) {
    let grow = 2.0;
    let s = dists::spiral::Spiral::new([400.0, 400.0], 17.0, grow);

    fn aabb_create_isize(pos: [isize; 2], radius: isize) -> axgeom::Rect<isize> {
        axgeom::Rect::new(
            pos[0] - radius,
            pos[0] + radius,
            pos[1] - radius,
            pos[1] + radius,
        )
    }
    let bots: Vec<_> = s
        .as_isize()
        .take(100_000)
        .map(move |pos| BBox::new(aabb_create_isize(pos, 5), ()))
        .collect();

    b.iter(|| {
        let k = create_cont(axgeom::XAXISS, &bots);
        let _ = test::black_box(k);
    });
}

#[bench]
#[cfg(all(feature = "unstable", test))]
fn bench_cont2(b: &mut test::Bencher) {
    fn create_cont2<A: Axis, T: Aabb>(axis: A, middle: &[T]) -> axgeom::Range<T::Num> {
        let left = middle
            .iter()
            .map(|a| a.get().get_range(axis).left)
            .min()
            .unwrap();
        let right = middle
            .iter()
            .map(|a| a.get().get_range(axis).right)
            .max()
            .unwrap();
        axgeom::Range { left, right }
    }

    let grow = 2.0;
    let s = dists::spiral::Spiral::new([400.0, 400.0], 17.0, grow);

    fn aabb_create_isize(pos: [isize; 2], radius: isize) -> axgeom::Rect<isize> {
        axgeom::Rect::new(
            pos[0] - radius,
            pos[0] + radius,
            pos[1] - radius,
            pos[1] + radius,
        )
    }
    let bots: Vec<_> = s
        .as_isize()
        .take(100_000)
        .map(|pos| BBox::new(aabb_create_isize(pos, 5), ()))
        .collect();

    b.iter(|| {
        let k = create_cont2(axgeom::XAXISS, &bots);
        let _ = test::black_box(k);
    });
}

fn create_cont<A: Axis, T: Aabb>(axis: A, middle: &[T]) -> Option<axgeom::Range<T::Num>> {
    match middle.split_first() {
        Some((first, rest)) => {
            let mut min = first.get().get_range(axis).start;
            let mut max = first.get().get_range(axis).end;

            for a in rest.iter() {
                let start = &a.get().get_range(axis).start;
                let end = &a.get().get_range(axis).end;

                if *start < min {
                    min = *start;
                }

                if *end > max {
                    max = *end;
                }
            }

            Some(axgeom::Range {
                start: min,
                end: max,
            })
        }
        None => None,
    }
}

enum ConstructResult<'a, T: Aabb> {
    NonEmpty {
        div: T::Num,
        cont: Option<axgeom::Range<T::Num>>,
        mid: &'a mut [T],
        right: &'a mut [T],
        left: &'a mut [T],
    },
    Empty(&'a mut [T]),
}

fn construct_non_leaf<T: Aabb>(
    bin_strat: BinStrat,
    sorter: impl Sorter,
    div_axis: impl Axis,
    bots: &mut [T],
) -> ConstructResult<T> {
    let med = if bots.is_empty() {
        return ConstructResult::Empty(bots);
    } else {
        let closure =
            move |a: &T, b: &T| -> core::cmp::Ordering { oned::compare_bots(div_axis, a, b) };

        let k = {
            let mm = bots.len() / 2;
            pdqselect::select_by(bots, mm, closure);
            &bots[mm]
        };

        k.get().get_range(div_axis).start
    };

    //TODO. its possible that middle is empty is the ranges inserted had
    //zero length.
    //It is very important that the median bot end up be binned into the middile bin.
    //We know this must be true because we chose the divider to be the medians left border,
    //and we binned so that all bots who intersect with the divider end up in the middle bin.
    //Very important that if a bots border is exactly on the divider, it is put in the middle.
    //If this were not true, there is no guarentee that the middile bin has bots in it even
    //though we did pick a divider.
    let binned = match bin_strat {
        BinStrat::Checked => oned::bin_middle_left_right(div_axis, &med, bots),
        BinStrat::NotChecked => unsafe {
            oned::bin_middle_left_right_unchecked(div_axis, &med, bots)
        },
    };

    //debug_assert!(!binned.middle.is_empty());
    sorter.sort(div_axis.next(), binned.middle);

    let cont = create_cont(div_axis, binned.middle);

    //We already know that the middile is non zero in length.

    ConstructResult::NonEmpty {
        mid: binned.middle,
        cont,
        div: med,
        left: binned.left,
        right: binned.right,
    }
}
