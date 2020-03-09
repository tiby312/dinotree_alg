use crate::inner_prelude::*;

///Helper module for creating Vecs of different types of BBoxes.
pub mod bbox_helper {
    use crate::inner_prelude::*;

    ///Convenience function to create a `&mut (Rect<N>,T)` from a `(Rect<N>,T)` bounding box.
    pub fn create_bbox_indirect<'a, N: Num, T>(
        bots: &'a mut [BBox<N, T>],
    ) -> Vec<BBoxIndirect<'a, BBox<N, T>>> {
        bots.iter_mut().map(|a| BBoxIndirect::new(a)).collect()
    }

    ///Convenience function to create a `(Rect<N>,&mut T)` from a `T` and a Rect<N> generating function.
    pub fn create_bbox_mut<'a, N: Num, T>(
        bots: &'a mut [T],
        mut aabb_create: impl FnMut(&T) -> Rect<N>,
    ) -> Vec<BBoxMut<'a, N, T>> {
        bots.iter_mut()
            .map(move |k| BBoxMut::new(aabb_create(k), k))
            .collect()
    }

    ///Helper struct to construct a DinoTree of `(Rect<N>,T)` from a dinotree of `(Rect<N>,&mut T)`
    pub struct IntoDirectHelper<N, T>(Vec<BBox<N, T>>);

    ///Convenience function to create a list of `(Rect<N>,T)` from a `(Rect<N>,&mut T)`. `T` must implement Copy.
    pub fn generate_direct<A: Axis, N: Num, T: Copy>(
        tree: &DinoTree<A, NodeMut<BBoxMut<N, T>>>,
    ) -> IntoDirectHelper<N, T> {
        IntoDirectHelper(
            tree.inner
                .get_nodes()
                .iter()
                .flat_map(|a| a.range.iter())
                .map(|a| BBox::new(a.rect, *a.inner))
                .collect(),
        )
    }

    ///Take a DinoTree of `(Rect<N>,&mut T)` and creates a new one of type `(Rect<N>,T)`
    pub fn into_direct<'a, A: Axis, N: Num, T>(
        tree: &DinoTree<A, NodeMut<BBoxMut<N, T>>>,
        bots: &'a mut IntoDirectHelper<N, T>,
    ) -> DinoTree<A, NodeMut<'a, BBox<N, T>>> {
        let mut bots = &mut bots.0 as &'a mut [_];
        let nodes: Vec<_> = tree
            .inner
            .get_nodes()
            .iter()
            .map(|node| {
                let mut k: &mut [_] = &mut [];
                core::mem::swap(&mut bots, &mut k);
                let (first, mut rest) = k.split_at_mut(node.range.len());
                core::mem::swap(&mut bots, &mut rest);
                NodeMut {
                    range: first,
                    cont: node.cont,
                    div: node.div,
                }
            })
            .collect();

        DinoTree {
            inner: compt::dfs_order::CompleteTreeContainer::from_preorder(nodes).unwrap(),
            axis: tree.axis,
            bot_ptr:bots as *const _
        }
    }
}

///A version of dinotree that is not lifetimed and uses unsafe{} to own the elements
///that are in its tree (as a self-referential struct). Composed of `(Rect<N>,*mut T)`.
pub mod dinotree_owned;

pub mod analyze;

pub use rigid::CollisionList;
mod rigid;

pub(crate) use self::notsorted::NotSorted;
mod notsorted {
    use super::*;

    ///A version of dinotree where the elements are not sorted along each axis, like a KD Tree.
    /// For comparison, a normal kd-tree is provided by `NotSorted`. In this tree, the elements are not sorted
    /// along an axis at each level. Construction of `NotSorted` is faster than `DinoTree` since it does not have to
    /// sort bots that belong to each node along an axis. But most query algorithms can usually take advantage of this
    /// extra property.
    pub struct NotSorted<A: Axis, N: Node>(pub(crate) DinoTree<A, N>);

    impl<'a, T: Aabb + Send + Sync> NotSorted<DefaultA, NodeMut<'a, T>> {
        pub fn new_par(bots: &'a mut [T]) -> NotSorted<DefaultA, NodeMut<'a, T>> {
            DinoTreeBuilder::new(bots).build_not_sorted_par()
        }
    }
    impl<'a, T: Aabb> NotSorted<DefaultA, NodeMut<'a, T>> {
        pub fn new(bots: &'a mut [T]) -> NotSorted<DefaultA, NodeMut<'a, T>> {
            DinoTreeBuilder::new(bots).build_not_sorted_seq()
        }
    }

    impl<'a, A: Axis, T: Aabb + Send + Sync> NotSorted<A, NodeMut<'a, T>> {
        pub fn with_axis_par(axis: A, bots: &'a mut [T]) -> NotSorted<A, NodeMut<'a, T>> {
            DinoTreeBuilder::with_axis(axis, bots).build_not_sorted_par()
        }
    }
    impl<'a, A: Axis, T: Aabb> NotSorted<A, NodeMut<'a, T>> {
        pub fn with_axis(axis: A, bots: &'a mut [T]) -> NotSorted<A, NodeMut<'a, T>> {
            DinoTreeBuilder::with_axis(axis, bots).build_not_sorted_seq()
        }
    }

    impl<A: Axis, N: Node + Send + Sync> NotSorted<A, N>
    where
        N::T: Send + Sync,
    {
        pub fn find_collisions_mut_par(
            &mut self,
            func: impl Fn(PMut<N::T>, PMut<N::T>) + Send + Sync,
        ) {
            colfind::NotSortedQueryBuilder::new(self).query_par(|a, b| func(a, b));
        }
    }
    impl<A: Axis, N: Node> NotSorted<A, N> {
        pub fn find_collisions_mut(&mut self, mut func: impl FnMut(PMut<N::T>, PMut<N::T>)) {
            colfind::NotSortedQueryBuilder::new(self).query_seq(|a, b| func(a, b));
        }

        #[inline(always)]
        pub fn axis(&self) -> A {
            self.0.axis()
        }

        #[inline(always)]
        pub fn get_height(&self) -> usize {
            self.0.get_height()
        }

        #[inline(always)]
        pub fn vistr(&self) -> Vistr<N> {
            self.0.inner.vistr()
        }

        #[inline(always)]
        pub fn vistr_mut(&mut self) -> VistrMut<N> {
            VistrMut {
                inner: self.0.inner.vistr_mut(),
            }
        }
    }
}

use crate::query::*;

///The data structure this crate revoles around.
pub struct DinoTree<A: Axis, N: Node> {
    axis: A,
    inner: compt::dfs_order::CompleteTreeContainer<N, compt::dfs_order::PreOrder>,
    bot_ptr:*const [N::T] //just used for reference. TODO check that tree still implements Send+Sync
}

///The type of the axis of the first node in the dinotree.
///If it is the y axis, then the first divider will be a horizontal line,
///since it is partioning space based off of objects y value.
pub type DefaultA = YAXIS;
///Constructor of the default axis type. Needed since you cannot construct from type alias's.
pub const fn default_axis() -> YAXIS {
    YAXIS
}

impl<'a, T: Aabb> DinoTree<DefaultA, NodeMut<'a, T>> {
    /// # Examples
    ///
    ///```
    ///let mut bots = [axgeom::rect(0,10,0,10)];
    ///let tree = dinotree_alg::DinoTree::new(&mut bots);
    ///
    ///```
    pub fn new(bots: &'a mut [T]) -> DinoTree<DefaultA, NodeMut<'a, T>> {
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

impl<'a, A: Axis, T: Aabb + Send + Sync> DinoTree<A, NodeMut<'a, T>> {
    /// # Examples
    ///
    ///```
    ///let mut bots = [axgeom::rect(0,10,0,10)];
    ///let tree = dinotree_alg::DinoTree::with_axis(axgeom::XAXIS,&mut bots);
    ///
    ///```
    pub fn with_axis_par(axis: A, bots: &'a mut [T]) -> DinoTree<A, NodeMut<'a, T>> {
        DinoTreeBuilder::with_axis(axis, bots).build_par()
    }
}




impl<A: Axis, N: Node + Send + Sync> DinoTree<A, N>
where
    N::T:HasInner + Send + Sync
{
    //TODO documennt
    ///Sometimes you want want to iterate over all the collisions multiple times.
    ///this function lets you do this safely. it is implemented on top of
    ///find__collisions_mut_par_ext
    pub fn collect_collisions_list_par<K:Send+Sync>(
            &mut self,collision:impl Fn(&mut <N::T as HasInner>::Inner,&mut <N::T as HasInner>::Inner)->Option<K> + Send +Sync
    )->CollisionList<N::T,K>
    {
        rigid::create_collision_list(self,collision)
    }
    /// # Examples
    ///
    ///```
    ///use dinotree_alg::prelude::*;
    ///let mut bots = [bbox(axgeom::rect(0,10,0,10),0u8),bbox(axgeom::rect(5,15,5,15),0u8)];
    ///let mut tree = DinoTree::new(&mut bots);
    ///tree.find_collisions_mut_par(|mut a,mut b|{
    ///    *a.inner_mut()+=1;
    ///    *b.inner_mut()+=1;
    ///});
    ///
    ///assert_eq!(bots[0].inner,1);
    ///assert_eq!(bots[1].inner,1);
    ///```
    pub fn find_collisions_mut_par(&mut self, func: impl Fn( &mut <N::T as HasInner>::Inner,&mut <N::T as HasInner>::Inner) + Send + Sync) {
        query::colfind::QueryBuilder::new(self).query_par(|mut a,mut b| func(a.inner_mut(), b.inner_mut()));
    }





    ///TODO document
    pub fn find_collisions_par_ext<B:Send+Sync>(
        &mut self,
        split: impl Fn(&mut B)->B + Send + Sync+Copy,
        fold: impl Fn(&mut B,B) + Send + Sync+Copy,
        collision:impl Fn(&mut B,&mut <N::T as HasInner>::Inner,&mut <N::T as HasInner>::Inner) + Send + Sync + Copy,
        acc: B)->B{

        struct Foo<
            T,
            A,
            B,
            C,
            D>{
            _p:PhantomData<T>,
            acc:A,
            split:B,
            fold:C,
            collision:D,
        }

        

        impl<T:Aabb+HasInner,A,B,C,D:Fn(&mut A,&mut T::Inner,&mut T::Inner)> ColMulti for Foo<T,A,B,C,D>{
            type T=T;
            fn collide(&mut self, mut a: PMut<Self::T>, mut b: PMut<Self::T>){
                (self.collision)(&mut self.acc,a.inner_mut(),b.inner_mut())
            }
        }
        impl<T,A,B:Fn(&mut A)->A+Copy,C:Fn(&mut A,A)+Copy,D:Copy> Splitter for Foo<T,A,B,C,D>{
            fn div(&mut self) -> Self{
                let acc=(self.split)(&mut self.acc);
                Foo{
                    _p:PhantomData,
                    acc,
                    split:self.split,
                    fold:self.fold,
                    collision:self.collision
                }
            }

            fn add(&mut self, b: Self){
                (self.fold)(&mut self.acc,b.acc)
            }

            fn node_start(&mut self){}

            fn node_end(&mut self){}
        }

        let foo=Foo{
            _p:PhantomData,
            acc,
            split,
            fold,
            collision
        };

        let foo=query::colfind::QueryBuilder::new(self).query_splitter_par(foo);
        foo.acc
    }


}


impl<A: Axis, N: Node + Send + Sync> DinoTree<A, N>
where
    N::T: Send + Sync,
{
   

    #[cfg(feature = "nbody")]
    pub fn nbody_mut_par<X: query::nbody::NodeMassTrait<Num = N::Num, Item = N::T> + Sync + Send>(
        &mut self,
        ncontext: &X,
        rect: Rect<N::Num>,
    ) where
        X::No: Send,
        N::T: Send + Copy,
        N::T:HasInner
    {
        query::nbody::nbody_par(self, ncontext, rect)
    }

    //TODO remove send/sync trait bounds
    #[cfg(feature = "nbody")]
    pub fn nbody_mut<X: query::nbody::NodeMassTrait<Num = N::Num, Item = N::T> + Sync + Send>(
        &mut self,
        ncontext: &X,
        rect: Rect<N::Num>,
    ) where
        X::No: Send,
        N::T: Send + Sync,
        N::T:HasInner
    {
        query::nbody::nbody(self, ncontext, rect)
    }
}



impl<A: Axis, N: Node> DinoTree<A, N> where N::T:HasInner{

    /// # Examples
    ///
    ///```
    ///use dinotree_alg::prelude::*;
    ///use axgeom::*;
    ///
    ///struct Foo;
    ///impl RayCast for Foo{
    ///    type T=BBox<i32,Vec2<i32>>;
    ///    type N=i32;
    ///    fn compute_distance_to_rect(&self, ray: &Ray<Self::N>, a: &Rect<Self::N>) -> CastResult<Self::N>{
    ///        ray.cast_to_rect(a)
    ///    }
    ///
    ///    fn compute_distance_to_bot(&self, ray: &Ray<Self::N>, a: &Self::T) -> CastResult<Self::N> {
    ///         //Do more fine-grained collision checking.
    ///         //Here we know the two aabbs intersect, but do an additional check
    ///         //to see if they intersect as circles.
    ///         ray.inner_as::<f32>().cast_to_circle(a.inner.inner_as(),5.).map(|a|a as i32)
    ///    }
    ///}
    ///
    ///let border = rect(0,100,0,100);
    ///
    ///let mut bots = [bbox(rect(0,10,0,10),vec2(5,5)),
    ///                bbox(rect(2,5,2,5),vec2(4,4)),
    ///                bbox(rect(4,10,4,10),vec2(5,5))];
    ///
    ///
    ///let mut bots_copy=bots.clone();
    ///let mut tree = DinoTree::new(&mut bots);
    ///let ray=ray(vec2(5,-5),vec2(0,1));
    ///let res = tree.raycast_fine_mut(ray,&mut Foo,border);
    ///
    ///let (bots,dis)=res.unwrap();
    ///assert_eq!(dis,4);
    ///assert_eq!(bots.len(),1);
    ///assert_eq!(bots[0].get(),&rect(2,5,2,5));
    ///```
    pub fn raycast_fine_mut(
        &mut self,
        ray: axgeom::Ray<N::Num>,
        rtrait: &mut impl raycast::RayCast<N = N::Num, T = N::T>,
        border: Rect<N::Num>,
    ) -> raycast::RayCastResult<<N::T as HasInner>::Inner,N::Num> {
        raycast::raycast_mut(self, border, ray, rtrait)
    }

    /// # Examples
    ///
    ///```
    ///use dinotree_alg::prelude::*;
    ///use axgeom::*;
    ///let border = rect(0,100,0,100);
    ///
    ///let mut bots = [rect(0,10,0,10),
    ///                rect(2,5,2,5),
    ///                rect(4,10,4,10)];
    ///
    ///let mut bots_copy=bots.clone();
    ///let mut tree = DinoTree::new(&mut bots);
    ///let ray=ray(vec2(5,-5),vec2(0,1));
    ///let res = tree.raycast_mut(ray,|ray,r|ray.cast_to_rect(r),border);
    ///
    ///let (bots,dis)=res.unwrap();
    ///assert_eq!(dis,5);
    ///assert_eq!(bots.len(),1);
    ///assert_eq!(bots[0].get(),&rect(0,10,0,10));
    ///```
    pub fn raycast_mut(
        &mut self,
        ray: axgeom::Ray<N::Num>,
        rtrait: impl Fn(&Ray<N::Num>, &Rect<N::Num>) -> axgeom::CastResult<N::Num>,
        border: Rect<N::Num>,
    ) -> raycast::RayCastResult<<N::T as HasInner>::Inner,N::Num> {
        let mut rtrait = raycast::RayCastFineWrapper {
            inner: rtrait,
            _p: PhantomData,
        };

        raycast::raycast_mut(self, border, ray, &mut rtrait)
    }
    /// # Examples
    ///
    ///```
    ///use dinotree_alg::prelude::*;
    ///use axgeom::*;
    ///let border = rect(0,100,0,100);
    ///
    ///let mut bots = [rect(0,10,0,10),
    ///                rect(2,5,2,5),
    ///                rect(4,10,4,10)];
    ///
    ///let mut bots_copy=bots.clone();
    ///let mut tree = DinoTree::new(&mut bots);
    ///
    ///let res = tree.k_nearest_mut(vec2(0,0),2,|a,b|b.distance_squared_to_point(a).unwrap_or(0),border);
    ///
    ///assert_eq!(res.len(),2);
    ///assert_eq!(res[0].bot.get(),&bots_copy[0]);
    ///assert_eq!(res[1].bot.get(),&bots_copy[1]);
    ///```
    pub fn k_nearest_mut(
        &mut self,
        point: Vec2<N::Num>,
        num: usize,
        distance: impl Fn(Vec2<N::Num>, &Rect<N::Num>) -> N::Num,
        border: Rect<N::Num>,
    ) -> Vec<k_nearest::KnearestResult<<N::T as HasInner>::Inner,N::Num>> {
        let mut knear = k_nearest::KnearestWrapper {
            inner: distance,
            _p: PhantomData,
        };
        k_nearest::k_nearest_mut(self, point, num, &mut knear, border)
    }

    /// # Examples
    ///
    ///```
    ///use dinotree_alg::prelude::*;
    ///use axgeom::*;
    ///let border = rect(0,100,0,100);
    ///
    ///struct Foo;
    ///impl Knearest for Foo{
    ///     type T=BBox<i32,Vec2<i32>>;
    ///     type N=i32;
    ///     fn distance_to_rect(&self, point: Vec2<Self::N>, rect: &Rect<Self::N>) -> Self::N{
    ///         rect.distance_squared_to_point(point).unwrap_or(0)
    ///     }
    ///
    ///     fn distance_to_bot(&self, point: Vec2<Self::N>, bot: &Self::T) -> Self::N{
    ///         //Do more fine-grained checking here.
    ///         //At this point we know the aabbs intersect.
    ///         //Do additional checking to see if they intersect as circles
    ///         bot.inner.distance_squared_to_point(point)
    ///     }
    ///}
    ///let mut bots = [bbox(rect(0,10,0,10),vec2(0,0)),
    ///                bbox(rect(2,5,2,5),vec2(0,5)),
    ///                bbox(rect(4,10,4,10),vec2(3,3))];
    ///
    ///let mut bots_copy=bots.clone();
    ///let mut tree = DinoTree::new(&mut bots);
    ///
    ///let res = tree.k_nearest_fine_mut(vec2(0,0),2,&mut Foo,border);
    ///
    ///assert_eq!(res.len(),2);
    ///assert_eq!(res[0].bot.get(),bots_copy[0].get());
    ///assert_eq!(res[1].bot.get(),bots_copy[2].get());
    ///```
    pub fn k_nearest_fine_mut(
        &mut self,
        point: Vec2<N::Num>,
        num: usize,
        knear: &mut impl k_nearest::Knearest<N = N::Num, T = N::T>,
        border: Rect<N::Num>,
    ) -> Vec<k_nearest::KnearestResult<<N::T as HasInner>::Inner,N::Num>> {
        k_nearest::k_nearest_mut(self, point, num, knear, border)
    }

    /// # Examples
    ///
    ///```
    ///use dinotree_alg::prelude::*;
    ///let mut bots1 = [bbox(axgeom::rect(0,10,0,10),0u8)];
    ///let mut bots2 = [bbox(axgeom::rect(5,15,5,15),0u8)];
    ///let mut tree = DinoTree::new(&mut bots1);
    ///
    ///tree.intersect_with_mut(&mut bots2,|mut a,mut b|{
    ///    *a.inner_mut()+=1;
    ///    *b.inner_mut()+=2;    
    ///});
    ///
    ///assert_eq!(bots1[0].inner,1);
    ///assert_eq!(bots2[0].inner,2);
    ///```
    pub fn intersect_with_mut<X: Aabb<Num = N::Num>+HasInner>(
        &mut self,
        other: &mut [X],
        func: impl Fn(&mut <N::T as HasInner>::Inner, &mut X::Inner),
    ) {
        intersect_with::intersect_with_mut(self, other, |a,b|(func)(a.into_inner(),b.into_inner()))
    }


    /// # Examples
    ///
    ///```
    ///use dinotree_alg::prelude::*;
    ///let mut bots = [bbox(axgeom::rect(0,10,0,10),0u8)];
    ///let mut tree = DinoTree::new(&mut bots);
    ///tree.for_all_not_in_rect_mut(&axgeom::rect(10,20,10,20),|mut a|{
    ///    *a.inner_mut()+=1;    
    ///});
    ///
    ///assert_eq!(bots[0].inner,1);
    ///
    ///```
    pub fn for_all_not_in_rect_mut(&mut self, rect: &Rect<N::Num>, mut func: impl FnMut(&mut <N::T as HasInner>::Inner)) {
        rect::for_all_not_in_rect_mut(self, rect, |a|(func)(a.into_inner()));
    }

    /// # Examples
    ///
    ///```
    ///use dinotree_alg::prelude::*;
    ///let mut bots = [bbox(axgeom::rect(0,10,0,10),0u8)];
    ///let mut tree = DinoTree::new(&mut bots);
    ///tree.for_all_intersect_rect_mut(&axgeom::rect(9,20,9,20),|mut a|{
    ///    *a.inner_mut()+=1;    
    ///});
    ///
    ///assert_eq!(bots[0].inner,1);
    ///
    ///```
    pub fn for_all_intersect_rect_mut(
        &mut self,
        rect: &Rect<N::Num>,
        mut func: impl FnMut(&mut <N::T as HasInner>::Inner),
    ) {
        rect::for_all_intersect_rect_mut(self, rect, |a|(func)(a.into_inner()));
    }

    /// # Examples
    ///
    ///```
    ///use dinotree_alg::prelude::*;
    ///let mut bots = [bbox(axgeom::rect(0,10,0,10),0u8)];
    ///let mut tree = DinoTree::new(&mut bots);
    ///tree.for_all_in_rect_mut(&axgeom::rect(0,10,0,10),|mut a|{
    ///    *a.inner_mut()+=1;    
    ///});
    ///
    ///assert_eq!(bots[0].inner,1);
    ///
    ///```
    pub fn for_all_in_rect_mut(&mut self, rect: &Rect<N::Num>, mut func: impl FnMut(&mut <N::T as HasInner>::Inner)) {
        rect::for_all_in_rect_mut(self, rect, |a|(func)(a.into_inner()));
    }

    /// # Examples
    ///
    ///```
    ///use dinotree_alg::prelude::*;
    ///let mut bots = [bbox(axgeom::rect(0,10,0,10),0u8),bbox(axgeom::rect(5,15,5,15),0u8)];
    ///let mut tree = DinoTree::new(&mut bots);
    ///tree.find_collisions_mut(|mut a,mut b|{
    ///    *a.inner_mut()+=1;
    ///    *b.inner_mut()+=1;
    ///});
    ///
    ///assert_eq!(bots[0].inner,1);
    ///assert_eq!(bots[1].inner,1);
    ///```
    pub fn find_collisions_mut(&mut self, mut func: impl FnMut(&mut <N::T as HasInner>::Inner, &mut <N::T as HasInner>::Inner)) {
        colfind::QueryBuilder::new(self).query_seq(|a, b| func(a.into_inner(), b.into_inner()));
    }




    pub fn collect_all<D>(&mut self,mut func:impl FnMut(&Rect<N::Num>,&mut <N::T as HasInner>::Inner)->Option<D>)->SingleCollisionList<N::T,D>{
        let mut res=Vec::new();
        for node in self.inner.get_nodes_mut().iter_mut(){
            for b in node.get_mut().bots.iter_mut(){
                let (x,y)=b.unpack();
                if let Some(d)=func(x,y){
                    res.push(SingleCol{inner:y as *mut _,mag:d});
                }
            }
        }
        SingleCollisionList{a:res,bot_ptr:self.bot_ptr}
    }


}

pub struct SingleCollisionList<T:HasInner,D>{
    bot_ptr:*const [T],
    a:Vec<SingleCol<T,D>>
}

unsafe impl<T:HasInner,D> Send for SingleCol<T,D>{}
unsafe impl<T:HasInner,D> Sync for SingleCol<T,D>{}
struct SingleCol<T:HasInner,D>{
    inner:*mut T::Inner,
    mag:D
}

impl<T:Aabb+HasInner+Send+Sync,D:Send+Sync> SingleCollisionList<T,D>{

     pub fn for_every_par(&mut self,arr:&mut [T],func:impl Fn(&mut T::Inner,&mut D)+Send+Sync+Copy){
        use rayon::prelude::*;
        assert_eq!(self.bot_ptr,arr as *const _ );
        self.a.par_iter_mut().for_each(|a|func(unsafe{&mut *a.inner},&mut a.mag));
    }
}
impl<T:Aabb+HasInner,D> SingleCollisionList<T,D>{
    pub fn for_every(&mut self,arr:&mut [T],mut func:impl FnMut(&mut T::Inner,&mut D)){
        assert_eq!(self.bot_ptr,arr as *const _ );
        for a in self.a.iter_mut(){
            func(unsafe{&mut *a.inner},&mut a.mag)
        }
    }

}


impl<A: Axis, N: Node> DinoTree<A, N> {
    pub fn get_bots(&self)->&[N::T]{
        unsafe{&*self.bot_ptr}
    }

    pub fn get_bots_mut(&mut self)->&mut [N::T]{
        unsafe{&mut *(self.bot_ptr as *mut _)}
    }
    /// # Examples
    ///
    /// ```
    /// use dinotree_alg::prelude::*;
    /// use axgeom::*;
    ///
    /// struct Drawer;
    /// impl DividerDrawer for Drawer{
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
    pub fn draw(&self, drawer: &mut impl graphics::DividerDrawer<N = N::Num>, rect: &Rect<N::Num>) {
        graphics::draw(self, drawer, rect)
    }

   

    /// # Examples
    ///
    ///```
    ///use dinotree_alg::prelude::*;
    ///let mut bots1 = [bbox(axgeom::rect(0,10,0,10),0u8)];
    ///let mut tree = DinoTree::new(&mut bots1);
    ///let mut multi = tree.multi_rect();
    ///
    ///multi.for_all_in_rect_mut(axgeom::rect(0,10,0,10),|a|{}).unwrap();
    ///let res = multi.for_all_in_rect_mut(axgeom::rect(5,15,5,15),|a|{});
    ///assert_eq!(res,Err(RectIntersectErr));
    ///```
    #[must_use]
    pub fn multi_rect(&mut self) -> rect::MultiRectMut<A, N> {
        rect::MultiRectMut::new(self)
    }
    /// # Examples
    ///
    ///```
    ///use dinotree_alg::prelude::*;
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
    pub fn for_all_intersect_rect<'a>(&'a self, rect: &Rect<N::Num>, func: impl FnMut(&'a N::T)) {
        rect::for_all_intersect_rect(self, rect, func);
    }



    /// # Examples
    ///
    ///```
    ///use dinotree_alg::prelude::*;
    ///let mut bots = [axgeom::rect(0,10,0,10),axgeom::rect(20,30,20,30)];
    ///let mut tree = DinoTree::new(&mut bots);
    ///let mut test = Vec::new();
    ///tree.for_all_in_rect(&axgeom::rect(0,20,0,20),|a|{
    ///    test.push(a);
    ///});
    ///
    ///assert_eq!(test[0],&axgeom::rect(0,10,0,10));
    ///
    pub fn for_all_in_rect<'a>(&'a self, rect: &Rect<N::Num>, func: impl FnMut(&'a N::T)) {
        rect::for_all_in_rect(self, rect, func);
    }

    /// # Examples
    ///
    ///```
    ///use dinotree_alg::prelude::*;
    ///let mut bots = [axgeom::rect(0,10,0,10)];
    ///let mut tree = DinoTree::new(&mut bots);
    ///
    ///use axgeom::Axis;
    ///assert!(tree.axis().is_equal_to(default_axis()));
    ///```
    pub fn axis(&self) -> A {
        self.axis
    }

    /// # Examples
    ///
    ///```
    ///use dinotree_alg::prelude::*;
    ///let mut bots = [bbox(axgeom::rect(0,10,0,10),0)];
    ///let mut tree = DinoTree::new(&mut bots);
    ///
    ///use compt::Visitor;
    ///for mut b in tree.vistr_mut().dfs_preorder_iter().flat_map(|n|n.get_mut().bots.iter_mut()){
    ///    *b.inner_mut()+=1;    
    ///}
    ///assert_eq!(bots[0].inner,1);
    ///```
    pub(crate) fn vistr_mut(&mut self) -> VistrMut<N> {
        VistrMut {
            inner: self.inner.vistr_mut(),
        }
    }

    /// # Examples
    ///
    ///```
    ///use dinotree_alg::prelude::*;
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
    pub fn vistr(&self) -> Vistr<N> {
        self.inner.vistr()
    }

    /// # Examples
    ///
    ///```
    ///use dinotree_alg::prelude::*;
    ///let mut bots = vec![axgeom::rect(0,10,0,10);400];
    ///let mut tree = DinoTree::new(&mut bots);
    ///
    ///assert_eq!(tree.get_height(),analyze::compute_tree_height_heuristic(400,analyze::DEFAULT_NUMBER_ELEM_PER_NODE));
    ///```
    ///
    pub fn get_height(&self) -> usize {
        self.inner.get_height()
    }

    /// # Examples
    ///
    ///```
    ///use dinotree_alg::prelude::*;
    ///let mut bots = vec![axgeom::rect(0,10,0,10);400];
    ///let mut tree = DinoTree::new(&mut bots);
    ///
    ///assert_eq!(tree.num_nodes(),analyze::nodes_left(0,tree.get_height() ));
    ///
    ///```
    pub fn num_nodes(&self) -> usize {
        self.inner.get_nodes().len()
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
        pub fn build_not_sorted_par(&mut self) -> NotSorted<A, NodeMut<'a, T>> {
            let mut bots: &mut [T] = &mut [];
            core::mem::swap(&mut bots, &mut self.bots);
            let bot_ptr=bots as *const _;
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
                bot_ptr
            })
        }

        ///Build in parallel
        pub fn build_par(&mut self) -> DinoTree<A, NodeMut<'a, T>> {
            let mut bots: &mut [T] = &mut [];
            core::mem::swap(&mut bots, &mut self.bots);
            let bot_ptr=bots as *const _;
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
                bot_ptr
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
        pub fn build_not_sorted_seq(&mut self) -> NotSorted<A, NodeMut<'a, T>> {
            let mut bots: &mut [T] = &mut [];
            core::mem::swap(&mut bots, &mut self.bots);
            let bot_ptr=bots as *const _;
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
                bot_ptr
            })
        }

        ///Build sequentially
        pub fn build_seq(&mut self) -> DinoTree<A, NodeMut<'a, T>> {
            let mut bots: &mut [T] = &mut [];
            core::mem::swap(&mut bots, &mut self.bots);
            let bot_ptr=bots as *const _;
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
                bot_ptr
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
        ) -> DinoTree<A, NodeMut<'a, T>> {
            let mut bots: &mut [T] = &mut [];
            core::mem::swap(&mut bots, &mut self.bots);
            let bot_ptr=bots as *const _;
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
                bot_ptr
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

    mod vistr_mut {
        use crate::inner_prelude::*;

        //Cannot use since we need create_wrap_mut()
        //We must create our own new type.
        //pub type VistrMut<'a,N> = compt::MapStruct<compt::dfs_order::VistrMut<'a,N,compt::dfs_order::PreOrder>,Foo<'a,N>>;

        /// Tree Iterator that returns a protected mutable reference to each node.
        #[repr(transparent)]
        pub struct VistrMut<'a, N: Node> {
            pub(crate) inner: compt::dfs_order::VistrMut<'a, N, compt::dfs_order::PreOrder>,
        }

        impl<'a, N: Node> VistrMut<'a, N> {
            ///It is safe to borrow the iterator and then produce mutable references from that
            ///as long as by the time the borrow ends, all the produced references also go away.
            #[inline(always)]
            pub fn create_wrap_mut(&mut self) -> VistrMut<N> {
                VistrMut {
                    inner: self.inner.create_wrap_mut(),
                }
            }
        }

        impl<'a, N: Node> core::ops::Deref for VistrMut<'a, N> {
            type Target = Vistr<'a, N>;

            #[inline(always)]
            fn deref(&self) -> &Vistr<'a, N> {
                unsafe { &*(self as *const VistrMut<_> as *const Vistr<_>) }
            }
        }

        unsafe impl<'a, N: Node> compt::FixedDepthVisitor for VistrMut<'a, N> {}

        impl<'a, N: Node> Visitor for VistrMut<'a, N> {
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
                self.inner.dfs_preorder(|a| func(PMut::new(a)));
            }
        }
    }
    pub use vistr_mut::VistrMut;

    ///Expose a node trait api so that we can have nodes made up of both
    ///&mut [T] and *mut [T].
    ///We ideally want to use the lifetimed version of `NodeMut`, but
    ///but for `DinoTreeOwned` we must use `NodePtr`.
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
            NodeRef {
                bots: self.range,
                cont: &self.cont,
                div: &self.div,
            }
        }
        fn get_mut(&mut self) -> NodeRefMut<Self::T> {
            NodeRefMut {
                bots: PMut::new(self.range),
                cont: &self.cont,
                div: &self.div,
            }
        }
    }

    ///A lifetimed node in a dinotree.
    pub struct NodeMut<'a, T: Aabb> {
        pub(crate) range: &'a mut [T],

        //range is empty iff cont is none.
        pub(crate) cont: Option<axgeom::Range<T::Num>>,
        //for non leafs:
        //  div is some iff mid is nonempty.
        //  div is none iff mid is empty.
        //for leafs:
        //  div is none
        pub(crate) div: Option<T::Num>,
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
        .fold(0, |acc, a| acc + a.range.len());
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
        .fold(0, |acc, a| acc + a.range.len());
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
            range: rest,
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
                    range: mid,
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
                    range: empty,
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
        .map(|pos| BBox::new(aabb_create_isize(pos, 5), ()))
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
        let closure = |a: &T, b: &T| -> core::cmp::Ordering { oned::compare_bots(div_axis, a, b) };

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
