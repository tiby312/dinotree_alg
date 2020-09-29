//! Module contains query related structs.

mod inner_prelude {
    pub use crate::inner_prelude::*;
    //pub(crate) use crate::query::tools;
    pub use alloc::vec::Vec;
    pub(crate) use axgeom;
    pub use axgeom::*;
    pub use compt;
    pub use compt::*;
    pub use core::marker::PhantomData;
    pub use itertools::Itertools;
}

pub use graphics::DividerDrawer;
pub use k_nearest::KnearestResult;
pub use raycast::RayCastResult;
pub use rect::{MultiRectMut, RectIntersectErr};

//#[cfg(feature = "nbody")]
pub use crate::query::nbody::NodeMassTrait;

///aabb broadphase collision detection
mod colfind;
pub use colfind::NotSortedQueryBuilder;


///Provides functionality to draw the dividers of a dinotree.
mod graphics;

///Contains all k_nearest code.
mod k_nearest;

///Contains all raycast code.
mod raycast;

///Allows user to intersect the tree with a seperate group of bots.
mod intersect_with;

///[EXPERIMENTAL] Contains all nbody code.
//#[cfg(feature = "nbody")]
mod nbody;

///Contains rect code.
mod rect;

///Contains misc tools
mod tools;




use self::inner_prelude::*;

///A collection of query functions 
pub trait Queries{
    type A:Axis;
    type N:Node<T=Self::T,Num=Self::Num>;
    type T:Aabb<Num=Self::Num>;
    type Num:Num;
    
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
    fn vistr_mut(&mut self)->VistrMut<Self::N>;

    
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
    fn vistr(&self)->Vistr<Self::N>;

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
    fn axis(&self)->Self::A;
        

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
    /// tree.draw_divider(&mut Drawer,&border);
    /// ```
    ///
    fn draw_divider(&self,drawer: &mut impl graphics::DividerDrawer<N = Self::Num>, rect: &Rect<Self::Num>){
        graphics::draw(self.axis(),self.vistr(), drawer, rect)
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
    fn find_intersections_mut(
        &mut self,
        mut func: impl FnMut(&mut <Self::T as HasInner>::Inner, &mut <Self::T as HasInner>::Inner),
    ) where Self::T:HasInner{
        query::colfind::QueryBuilder::new(self.axis(),self.vistr_mut())
            .query_seq(move |mut a, mut b| func(a.inner_mut(), b.inner_mut())); 
    }



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
    fn find_intersections_mut_par(
        &mut self,
        mut func: impl Fn(&mut <Self::T as HasInner>::Inner, &mut <Self::T as HasInner>::Inner)+Clone+Send+Sync,
    ) where Self::N:Send+Sync,Self::T:HasInner+Send+Sync{
        query::colfind::QueryBuilder::new(self.axis(),self.vistr_mut())
            .query_par(move |mut a, mut b| func(a.inner_mut(), b.inner_mut())); 
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
    fn find_intersections_pmut(&mut self, mut func: impl FnMut(PMut<Self::T>, PMut<Self::T>)) {
        colfind::QueryBuilder::new(self.axis(),self.vistr_mut()).query_seq(move |a, b| func(a, b));
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
    fn find_intersections_par_ext<B: Send + Sync>(
        &mut self,
        split: impl Fn(&mut B) -> B + Send + Sync + Copy,
        fold: impl Fn(&mut B, B) + Send + Sync + Copy,
        collision: impl Fn(&mut B, &mut <Self::T as HasInner>::Inner, &mut <Self::T as HasInner>::Inner) + Send + Sync + Copy,
        acc: B,
    ) -> B where Self::T:HasInner+Send+Sync,Self::N:Send+Sync{
        struct Foo<T, A, B, C, D> {
            _p: PhantomData<T>,
            acc: A,
            split: B,
            fold: C,
            collision: D,
        }

        impl<T: Aabb + HasInner, A, B, C, D: Fn(&mut A, &mut T::Inner, &mut T::Inner)> colfind::ColMulti
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

        let foo = query::colfind::QueryBuilder::new(self.axis(),self.vistr_mut()).query_splitter_par(foo);
        foo.acc
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
    fn multi_rect(&mut self) -> rect::MultiRectMut<Self::A, Self::N> {
        rect::MultiRectMut::new(self.axis(),self.vistr_mut())
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
    fn for_all_intersect_rect<'b>(&'b self, rect: &Rect<Self::Num>, func: impl FnMut(&'b Self::T)) {
        rect::for_all_intersect_rect(self.axis(),self.vistr(), rect, func);
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
    fn for_all_in_rect<'b>(&'b self, rect: &Rect<Self::Num>, func: impl FnMut(&'b Self::T)) {
        rect::for_all_in_rect(self.axis(),self.vistr(), rect, func);
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
    fn for_all_not_in_rect_mut<'b>(
        &'b mut self,
        rect: &Rect<Self::Num>,
        mut func: impl FnMut(&'b mut <Self::T as HasInner>::Inner),
    ) where Self::T:HasInner{
        rect::for_all_not_in_rect_mut(self.axis(),self.vistr_mut(), rect, move |a| (func)(a.into_inner()));
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
    fn for_all_intersect_rect_mut<'b>(
        &'b mut self,
        rect: &Rect<Self::Num>,
        mut func: impl FnMut(&'b mut <Self::T as HasInner>::Inner),
    ) where Self::T:HasInner{
        rect::for_all_intersect_rect_mut(self.axis(),self.vistr_mut(), rect, move |a| (func)(a.into_inner()));
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
    fn for_all_in_rect_mut<'b>(
        &'b mut self,
        rect: &Rect<Self::Num>,
        mut func: impl FnMut(&'b mut <Self::T as HasInner>::Inner),
    ) where Self::T:HasInner{
        rect::for_all_in_rect_mut(self.axis(),self.vistr_mut(), rect, move |a| (func)(a.into_inner()));
    }


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
    fn raycast_mut<Acc>(
        &mut self,
        ray: axgeom::Ray<Self::Num>,
        start: &mut Acc,
        broad: impl FnMut(&mut Acc, &Ray<Self::Num>, &Rect<Self::Num>) -> CastResult<Self::Num>,
        fine: impl FnMut(&mut Acc, &Ray<Self::Num>, &Self::T) -> CastResult<Self::Num>,
        border: Rect<Self::Num>,
    ) -> raycast::RayCastResult<<Self::T as HasInner>::Inner, Self::Num> where Self::T:HasInner{
        let mut rtrait = raycast::RayCastClosure {
            a: start,
            broad,
            fine,
            _p: PhantomData,
        };
        raycast::raycast_mut(self.axis(),self.vistr_mut(), border, ray, &mut rtrait)
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
    fn k_nearest_mut<Acc>(
        &mut self,
        point: Vec2<Self::Num>,
        num: usize,
        start: &mut Acc,
        broad: impl FnMut(&mut Acc, Vec2<Self::Num>, &Rect<Self::Num>) -> Self::Num,
        fine: impl FnMut(&mut Acc, Vec2<Self::Num>, &Self::T) -> Self::Num,
        border: Rect<Self::Num>,
    ) -> Vec<k_nearest::KnearestResult<<Self::T as HasInner>::Inner, Self::Num>> where Self::T:HasInner {
        let mut foo = k_nearest::KnearestClosure {
            acc: start,
            broad,
            fine,
            _p: PhantomData,
        };
        k_nearest::k_nearest_mut(self.axis(),self.vistr_mut(), point, num, &mut foo, border)
    }


    
    //#[cfg(feature = "nbody")]
    fn nbody_mut<X: query::nbody::NodeMassTrait<Num = Self::Num, Item = Self::T> + Send + Sync>(
        &mut self,
        ncontext: &X,
        rect: Rect<Self::Num>,
    ) where
        X::No: Send,
        Self::N:Send+Sync,
        Self::T:HasInner+Send+Sync
    {
        query::nbody::nbody(self.axis(),self.vistr_mut(), ncontext, rect)
    }


    //#[cfg(feature = "nbody")]
    fn nbody_mut_par<X: query::nbody::NodeMassTrait<Num = Self::Num, Item = Self::T> + Sync + Send>(
        &mut self,
        ncontext: &X,
        rect: Rect<Self::Num>,
    ) where
        X::No: Send,
        Self::N:Send+Sync,
        Self::T:HasInner+Send+Sync
    {
        query::nbody::nbody_par(self.axis(),self.vistr_mut(), ncontext, rect)
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
    fn intersect_with_mut<X: Aabb<Num = Self::Num> + HasInner>(
        &mut self,
        other: &mut [X],
        func: impl Fn(&mut <Self::T as HasInner>::Inner, &mut X::Inner),
    ) where Self::T:HasInner{
        intersect_with::intersect_with_mut(self.axis(),self.vistr_mut(), other, move |a, b| {
            (func)(a.into_inner(), b.into_inner())
        })
    } 
    
}




pub fn find_collisions_sweep_mut<A: Axis, T: Aabb + HasInner>(
    bots: &mut [T],
    axis: A,
    mut func: impl FnMut(&mut T::Inner, &mut T::Inner),
) {
    colfind::query_sweep_mut(axis, bots, |a, b| func(a.into_inner(), b.into_inner()));
}







/// Collection of functions that panics if the dinotree result differs from the naive solution.
/// Should never panic unless invariants of the tree data struct have been violated.    
pub struct Assert;
impl Assert {
    ///Returns false if the tree's invariants are not met.
    #[must_use]
    pub fn tree_invariants<A: Axis, N:Node>(axis:A,vistr:Vistr<N>) -> bool {
        Self::inner(axis, vistr.with_depth(compt::Depth(0))).is_ok()
    }

    fn inner<A: Axis, N: Node>(axis: A, iter: compt::LevelIter<Vistr<N>>) -> Result<(), ()> {
        fn a_bot_has_value<N: Num>(it: impl Iterator<Item = N>, val: N) -> bool {
            for b in it {
                if b == val {
                    return true;
                }
            }
            false
        }

        macro_rules! assert2 {
            ($bla:expr) => {
                if !$bla {
                    return Err(());
                }
            };
        }

        let ((_depth, nn), rest) = iter.next();
        let nn = nn.get();
        let axis_next = axis.next();

        let f = |a: &&N::T, b: &&N::T| -> Option<core::cmp::Ordering> {
            let j = a
                .get()
                .get_range(axis_next)
                .start
                .cmp(&b.get().get_range(axis_next).start);
            Some(j)
        };

        {
            use is_sorted::IsSorted;
            assert2!(IsSorted::is_sorted_by(&mut nn.bots.iter(), f));
        }

        if let Some([start, end]) = rest {
            match nn.div {
                Some(div) => {
                    match nn.cont {
                        Some(cont) => {
                            for bot in nn.bots.iter() {
                                assert2!(bot.get().get_range(axis).contains(*div));
                            }

                            assert2!(a_bot_has_value(
                                nn.bots.iter().map(|b| b.get().get_range(axis).start),
                                *div
                            ));

                            for bot in nn.bots.iter() {
                                assert2!(cont.contains_range(bot.get().get_range(axis)));
                            }

                            assert2!(a_bot_has_value(
                                nn.bots.iter().map(|b| b.get().get_range(axis).start),
                                cont.start
                            ));
                            assert2!(a_bot_has_value(
                                nn.bots.iter().map(|b| b.get().get_range(axis).end),
                                cont.end
                            ));
                        }
                        None => assert2!(nn.bots.is_empty()),
                    }

                    Self::inner(axis_next, start)?;
                    Self::inner(axis_next, end)?;
                }
                None => {
                    for (_depth, n) in start.dfs_preorder_iter().chain(end.dfs_preorder_iter()) {
                        let n = n.get();
                        assert2!(n.bots.is_empty());
                        assert2!(n.cont.is_none());
                        assert2!(n.div.is_none());
                    }
                }
            }
        }
        Ok(())
    }

    pub fn find_intersections_mut<A: Axis, T: Aabb + HasInner>(tree: &mut DinoTreeWrap<A, T>) {
        let mut res_dino = Vec::new();
        tree.get_tree_mut().find_intersections_mut(|a, b| {
            let a = a as *const _ as usize;
            let b = b as *const _ as usize;
            let k = if a < b { (a, b) } else { (b, a) };
            res_dino.push(k);
        });

        let mut res_naive = Vec::new();
        NaiveAlgs::new(tree.get_bots_mut()).find_intersections_mut(|a, b| {
            let a = a as *const _ as usize;
            let b = b as *const _ as usize;
            let k = if a < b { (a, b) } else { (b, a) };
            res_naive.push(k);
        });

        res_naive.sort();
        res_dino.sort();

        assert_eq!(res_naive.len(), res_dino.len());
        assert!(res_naive.iter().eq(res_dino.iter()));
    }

    pub fn k_nearest_mut<Acc, A: Axis, T: Aabb + HasInner>(
        tree: &mut DinoTreeWrap<A, T>,
        point: Vec2<T::Num>,
        num: usize,
        acc: &mut Acc,
        mut broad: impl FnMut(&mut Acc, Vec2<T::Num>, &Rect<T::Num>) -> T::Num,
        mut fine: impl FnMut(&mut Acc, Vec2<T::Num>, &T) -> T::Num,
        rect: Rect<T::Num>,
    ) {
        let bots = tree.get_bots_mut();

        let mut res_naive = NaiveAlgs::new(bots)
            .k_nearest_mut(point, num, acc, &mut broad, &mut fine)
            .drain(..)
            .map(|a| (a.bot as *const _ as usize, a.mag))
            .collect::<Vec<_>>();

        let mut r = tree.get_tree_mut().k_nearest_mut(point, num, acc, broad, fine, rect);
        let mut res_dino: Vec<_> = r
            .drain(..)
            .map(|a| (a.bot as *const _ as usize, a.mag))
            .collect();

        res_naive.sort();
        res_dino.sort();

        assert_eq!(res_naive.len(), res_dino.len());
        assert!(res_naive.iter().eq(res_dino.iter()));
    }

    pub fn raycast_mut<Acc, A: Axis, T: Aabb + HasInner>(
        tree: &mut DinoTreeWrap<A, T>,
        ray: axgeom::Ray<T::Num>,
        start: &mut Acc,
        mut broad: impl FnMut(&mut Acc, &Ray<T::Num>, &Rect<T::Num>) -> CastResult<T::Num>,
        mut fine: impl FnMut(&mut Acc, &Ray<T::Num>, &T) -> CastResult<T::Num>,
        border: Rect<T::Num>,
    ) where
        <T as Aabb>::Num: core::fmt::Debug,
    {
        let bots = tree.get_bots_mut();

        let mut res_naive = Vec::new();
        match NaiveAlgs::new(bots).raycast_mut(ray, start, &mut broad, &mut fine, border) {
            RayCastResult::Hit((bots, mag)) => {
                for a in bots.iter() {
                    let j = (*a) as *const _ as usize;
                    res_naive.push((j, mag))
                }
            }
            RayCastResult::NoHit => {
                //do nothing
            }
        }

        let mut res_dino = Vec::new();
        match tree.get_tree_mut().raycast_mut(ray, start, broad, fine, border) {
            RayCastResult::Hit((bots, mag)) => {
                for a in bots.iter() {
                    let j = (*a) as *const _ as usize;
                    res_dino.push((j, mag))
                }
            }
            RayCastResult::NoHit => {
                //do nothing
            }
        }

        res_naive.sort();
        res_dino.sort();

        //dbg!("{:?}  {:?}",res_naive.len(),res_dino.len());
        assert_eq!(
            res_naive.len(),
            res_dino.len(),
            "len:{:?}",
            (res_naive, res_dino)
        );
        assert!(
            res_naive.iter().eq(res_dino.iter()),
            "nop:{:?}",
            (res_naive, res_dino)
        );
    }

    pub fn for_all_in_rect_mut<A: Axis, T: Aabb + HasInner>(
        tree: &mut DinoTreeWrap<A, T>,
        rect: &axgeom::Rect<T::Num>,
    ) {
        let mut res_dino = Vec::new();
        tree.get_tree_mut().for_all_in_rect_mut(rect, |a| {
            res_dino.push(a as *const _ as usize);
        });

        let mut res_naive = Vec::new();
        NaiveAlgs::new(tree.get_bots_mut()).for_all_in_rect_mut(rect, |a| {
            res_naive.push(a as *const _ as usize);
        });

        res_dino.sort();
        res_naive.sort();

        assert_eq!(res_naive.len(), res_dino.len());
        assert!(res_naive.iter().eq(res_dino.iter()));
    }

    /// Panics if the result differs from the naive solution.
    /// Should never panic unless invariants of the tree data struct have been violated.
    pub fn for_all_not_in_rect_mut<A: Axis, T: Aabb + HasInner>(
        tree: &mut DinoTreeWrap<A, T>,
        rect: &axgeom::Rect<T::Num>,
    ) {
        let mut res_dino = Vec::new();
        tree.get_tree_mut().for_all_not_in_rect_mut(rect, |a| {
            res_dino.push(a as *const _ as usize);
        });

        let mut res_naive = Vec::new();
        NaiveAlgs::new(tree.get_bots_mut()).for_all_not_in_rect_mut(rect, |a| {
            res_naive.push(a as *const _ as usize);
        });

        res_dino.sort();
        res_naive.sort();

        assert_eq!(res_naive.len(), res_dino.len());
        assert!(res_naive.iter().eq(res_dino.iter()));
    }

    pub fn for_all_intersect_rect_mut<A: Axis, T: Aabb + HasInner>(
        tree: &mut DinoTreeWrap<A, T>,
        rect: &axgeom::Rect<T::Num>,
    ) {
        let mut res_dino = Vec::new();
        tree.get_tree_mut().for_all_intersect_rect_mut(rect, |a| {
            res_dino.push(a as *const _ as usize);
        });

        let mut res_naive = Vec::new();
        NaiveAlgs::new(tree.get_bots_mut()).for_all_intersect_rect_mut(rect, |a| {
            res_naive.push(a as *const _ as usize);
        });

        res_dino.sort();
        res_naive.sort();

        assert_eq!(res_naive.len(), res_dino.len());
        assert!(res_naive.iter().eq(res_dino.iter()));
    }
}

///Provides the naive implementation of the dinotree api.
pub struct NaiveAlgs<'a, T> {
    bots: PMut<'a, [T]>,
}

impl<'a, T: Aabb + HasInner> NaiveAlgs<'a, T> {
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
        raycast::raycast_naive_mut(self.bots.as_mut(), ray, &mut rtrait, border)
    }

    #[must_use]
    pub fn k_nearest_mut<Acc>(
        &mut self,
        point: Vec2<T::Num>,
        num: usize,
        start: &mut Acc,
        broad: impl FnMut(&mut Acc, Vec2<T::Num>, &Rect<T::Num>) -> T::Num,
        fine: impl FnMut(&mut Acc, Vec2<T::Num>, &T) -> T::Num,
    ) -> Vec<k_nearest::KnearestResult<T::Inner, T::Num>> {
        let mut knear = k_nearest::KnearestClosure {
            acc: start,
            broad,
            fine,
            _p: PhantomData,
        };
        k_nearest::k_nearest_naive_mut(self.bots.as_mut(), point, num, &mut knear)
    }
}

impl<'a, T: Aabb + HasInner> NaiveAlgs<'a, T> {
    pub fn for_all_in_rect_mut(
        &mut self,
        rect: &Rect<T::Num>,
        mut func: impl FnMut(&mut T::Inner),
    ) {
        rect::naive_for_all_in_rect_mut(self.bots.as_mut(), rect, |a| (func)(a.into_inner()));
    }
    pub fn for_all_not_in_rect_mut(
        &mut self,
        rect: &Rect<T::Num>,
        mut func: impl FnMut(&mut T::Inner),
    ) {
        rect::naive_for_all_not_in_rect_mut(self.bots.as_mut(), rect, |a| (func)(a.into_inner()));
    }

    pub fn for_all_intersect_rect_mut(
        &mut self,
        rect: &Rect<T::Num>,
        mut func: impl FnMut(&mut T::Inner),
    ) {
        rect::naive_for_all_intersect_rect_mut(self.bots.as_mut(), rect, |a| {
            (func)(a.into_inner())
        });
    }

    pub fn find_intersections_mut(&mut self, mut func: impl FnMut(&mut T::Inner, &mut T::Inner)) {
        colfind::query_naive_mut(self.bots.as_mut(), |a, b| {
            func(a.into_inner(), b.into_inner())
        });
    }
}

impl<'a, T: Aabb> NaiveAlgs<'a, T> {
    #[must_use]
    pub fn new(bots: PMut<[T]>) -> NaiveAlgs<T> {
        NaiveAlgs { bots }
    }

    #[cfg(feature = "nbody")]
    pub fn nbody(&mut self, func: impl FnMut(PMut<T>, PMut<T>)) {
        nbody::naive_mut(self.bots.as_mut(), func);
    }
}
