#![feature(iterator_step_by)]
#![feature(test)]

extern crate axgeom;
extern crate compt;
extern crate ordered_float;
extern crate pdqselect;
extern crate rayon;

#[cfg(test)]
extern crate num;
#[cfg(test)]
extern crate cgmath;
#[cfg(test)]
extern crate collision;
extern crate dinotree_inner;
#[cfg(test)]
extern crate rand;
extern crate smallvec;
#[cfg(test)]
extern crate test;

mod inner_prelude {
    pub use dinotree_inner::prelude::*;
    pub use AABBox;
    pub use axgeom::Axis;
    pub use compt::LevelIter;
    pub use compt::LevelDesc;
    pub use axgeom::Range;
    pub use ::*;
    pub use oned::Sweeper;
    pub use compt::CTreeIterator;
    //pub use par;
    pub use axgeom::AxisTrait;
    pub use std::marker::PhantomData;
    pub use NumTrait;
    pub use ::*;
}

///Provides functionality to draw the dividers of a dinotree.
pub mod graphics;

///Contains convenience structs.
pub mod support;

///Contains query code
mod colfind;

///A collection of 1d functions that operate on lists of 2d objects.
mod oned;

///Contains misc tools
mod tools;

pub use rects::Rects;
mod rects;

//use dinotree_inner::support::DefaultDepthLevel;
pub use dinotree_inner::AABBox;
pub use dinotree_inner::NumTrait;
pub use dinotree_inner::SweepTrait;
use dinotree_inner::par;
use axgeom::Rect;
use axgeom::XAXISS;
use axgeom::YAXISS;
use colfind::ColMulti;
use smallvec::SmallVec;
use dinotree_inner::TreeTimer2;
use dinotree_inner::TreeTimerEmpty;
use dinotree_inner::Bag;

use dinotree_inner::compute_tree_height;

///Represents a destructured SweepTrait into the immutable bounding box reference,
///and the mutable reference to the rest of the object.
pub struct ColSingle<'a, T: SweepTrait + 'a> {
    pub rect: &'a AABBox<T::Num>,
    pub inner: &'a mut T::Inner,
}

use dinotree_inner::DynTree;

pub use ba::DinoTree;
pub(crate) use ba::DynTreeEnum;

mod ba {
    use super::*;
    use DynTree;

    mod closure_struct {
        use super::*;
        use std::marker::PhantomData;
        use ColSingle;
        use ColMulti;

        pub struct ColMultiStruct<
            'a,
            T: SweepTrait<Inner = I>,
            I: Send + Sync,
            F: Fn(ColSingle<T>, ColSingle<T>) + Send + Sync + 'a,
        > {
            a: &'a F,
            p: PhantomData<T>,
        }

        impl<
            'a,
            T: SweepTrait<Inner = I>,
            I: Send + Sync,
            F: Fn(ColSingle<T>, ColSingle<T>) + Send + Sync,
        > ColMultiStruct<'a, T, I, F>
        {
            pub fn new(a: &'a F) -> ColMultiStruct<'a, T, I, F> {
                ColMultiStruct { a, p: PhantomData }
            }
        }

        impl<
            'a,
            T: SweepTrait<Inner = I>,
            I: Send + Sync,
            F: Fn(ColSingle<T>, ColSingle<T>) + Send + Sync,
        > Copy for ColMultiStruct<'a, T, I, F>
        {
        }

        impl<
            'a,
            T: SweepTrait<Inner = I>,
            I: Send + Sync,
            F: Fn(ColSingle<T>, ColSingle<T>) + Send + Sync,
        > Clone for ColMultiStruct<'a, T, I, F>
        {
            fn clone(&self) -> Self {
                *self
            }
        }

        impl<
            'a,
            T: SweepTrait<Inner = I>,
            I: Send + Sync,
            F: Fn(ColSingle<T>, ColSingle<T>) + Send + Sync,
        > ColMulti for ColMultiStruct<'a, T, I, F>
        {
            type T = T;
            fn collide(&self, a: ColSingle<T>, b: ColSingle<T>) {
                (self.a)(a, b);
            }
        }
    }

    pub(crate) enum DynTreeEnum<'a, T: SweepTrait + 'a> {
        Xa(DynTree<'a, XAXISS, T>),
        Ya(DynTree<'a, YAXISS, T>),
    }

    ///This is the struct that this crate revolves around.
    pub struct DinoTree<'a, T: SweepTrait + 'a>(pub(crate) DynTreeEnum<'a, T>);

    impl<'a, T: SweepTrait + 'a> DinoTree<'a, T> {
        ///Create a dinotree.
        ///Specify the starting axis along which the bots will be partitioned.
        ///So if you picked the x axis, the root divider will be a vertical line.
        ///True means xaxis.
        ///The length of the slice must be less than the max value of a u32.
        pub fn new(rest: &'a mut [T], axis: bool) -> DinoTree<'a, T> {
            let height = self::compute_tree_height(rest.len());
            if axis {
                let k = DynTree::<XAXISS, T>::new::<
                    par::Parallel,
                    TreeTimerEmpty,
                >(rest, height);
                DinoTree(DynTreeEnum::Xa(k.0))
            } else {
                let k = DynTree::<YAXISS, T>::new::<
                    par::Parallel,
                    TreeTimerEmpty,
                >(rest, height);
                DinoTree(DynTreeEnum::Ya(k.0))
            }
        }

        ///Create a dinotree that does not use any parallel algorithms.
        pub fn new_seq(rest: &'a mut [T], axis: bool) -> DinoTree<'a, T> {
            let height = self::compute_tree_height(rest.len());
            if axis {
                let k = DynTree::<XAXISS, T>::new::<
                    par::Sequential,
                    TreeTimerEmpty,
                >(rest, height);
                DinoTree(DynTreeEnum::Xa(k.0))
            } else {
                let k = DynTree::<YAXISS, T>::new::<
                    par::Sequential,
                    TreeTimerEmpty,
                >(rest, height);
                DinoTree(DynTreeEnum::Ya(k.0))
            }
        }

        ///Create a dinotree.
        ///Specify the starting axis along which the bots will be partitioned.
        ///So if you picked the x axis, the root divider will be a vertical line.
        pub fn new_debug(rest: &'a mut [T], axis: bool) -> (DinoTree<'a, T>, Bag) {
            let height = self::compute_tree_height(rest.len());
            if axis {
                let k = DynTree::<XAXISS, T>::new::<par::Parallel, TreeTimer2>(
                    rest,
                    height,
                );
                (DinoTree(DynTreeEnum::Xa(k.0)), k.1)
            } else {
                let k = DynTree::<YAXISS, T>::new::<par::Parallel, TreeTimer2>(
                    rest,
                    height,
                );
                (DinoTree(DynTreeEnum::Ya(k.0)), k.1)
            }
        }

        ///Create a dinotree that does not use any parallel algorithms.
        pub fn new_seq_debug(rest: &'a mut [T], axis: bool) -> (DinoTree<'a, T>, Bag) {
            let height = self::compute_tree_height(rest.len());
            if axis {
                let k =
                    DynTree::<XAXISS, T>::new::<par::Sequential, TreeTimer2>(
                        rest,
                        height,
                    );
                (DinoTree(DynTreeEnum::Xa(k.0)), k.1)
            } else {
                let k =
                    DynTree::<YAXISS, T>::new::<par::Sequential, TreeTimer2>(
                        rest,
                        height,
                    );
                (DinoTree(DynTreeEnum::Ya(k.0)), k.1)
            }
        }

        ///Create a rect finding session.
        ///From this returned argument, you can retrive references to all elements
        ///within non-intersecting rectangles.
        pub fn rects<'b>(&'b mut self) -> Rects<'a, 'b, T> {
            Rects::new(self)
        }

        ///Find all intersects between the elements in this dinotree, and the specified elements.
        ///No intersecting pairs within each group are looked for.
        ///Only those between the two groups.
        ///Ideally the group that this tree is built around should be the bigger of the two groups.
        pub fn intersect_with_seq<X: SweepTrait<Num = T::Num>, F: FnMut(ColSingle<T>, ColSingle<X>)>(
            &mut self,
            b: &mut [X],
            mut func: F,
        ) {
            //todo find better algorithn?
            //todo do this before axis specialization?
            //ideally you'd bin the new group using the existing dividers and query that.
            //let func = &func;
            for i in b.iter_mut() {
                let jj = i.get_mut();

                self.for_all_intersect_rect(jj.0, |a: ColSingle<T>| {
                    let blag = ColSingle {
                        rect: jj.0,
                        inner: jj.1,
                    };
                    func(a, blag);
                });
            }
        }

        ///Unlike the rects session api, this function returns all elements within the specified
        ///rectangle AND all those that intersect with it. This more relaxed requirement means that
        ///we can no longer query non intersecting rectangles and be assured that the two respective
        ///sets of bots are disjoint.
        pub fn for_all_intersect_rect<F: FnMut(ColSingle<T>)>(
            &mut self,
            rect: &AABBox<T::Num>,
            fu: F,
        ) {
            match &mut self.0 {
                &mut DynTreeEnum::Xa(ref mut a) => {
                    colfind::for_all_intersect_rect(a, &rect.0, fu);
                }
                &mut DynTreeEnum::Ya(ref mut a) => {
                    colfind::for_all_intersect_rect(a, &rect.0, fu);
                }
            }
        }

        ///The dinotree's NumTrait does not inherit any kind of arithmetic traits.
        ///This showcases that the tree construction and pair finding collision algorithms
        ///do not involves any arithmetic. 
        ///However, when finding the nearest neighbor, we need to do some calculations to
        ///compute distance between points. So instead of giving the NumTrait arithmetic and thus
        ///add uneeded bounds for general use of this tree, the user must provide functions for arithmetic
        ///specifically for this function.
        ///The use can also specify what the minimum distance function is minizing based off of. For example
        ///minimizing based off the square distance will give you the same answer as minimizing based off 
        ///of the distant. 
        pub fn k_nearest<
            F: FnMut(ColSingle<T>),
            MF:Fn((T::Num,T::Num),&AABBox<T::Num>)->T::Num,
            MF2:Fn(T::Num,T::Num)->T::Num,
        >(
            &mut self,
            
            point: (T::Num, T::Num),
            num:usize,
            clos: F,
            mf:MF,
            mf2:MF2,
        ) {
            match &mut self.0 {
                &mut DynTreeEnum::Xa(ref mut a) => {
                    colfind::k_nearest::<XAXISS,_, _,_,_>(
                        a,
                        point,
                        num,
                        clos,
                        mf,
                        mf2
                    )
                }
                &mut DynTreeEnum::Ya(ref mut a) => {
                    colfind::k_nearest::<YAXISS,_, _,_,_>(
                        a,
                        point,
                        num,
                        clos,
                        mf,
                        mf2
                    )
                }
            };
        }

        ///Find all intersecting pairs sequentially.
        ///Notice that in this case, a FnMut is supplied instead of a Fn.
        pub fn intersect_every_pair_seq<F: FnMut(ColSingle<T>, ColSingle<T>)>(&mut self, clos: F) {
            match &mut self.0 {
                &mut DynTreeEnum::Xa(ref mut a) => {
                    colfind::for_every_col_pair_seq::<_, T, _, TreeTimerEmpty>(
                        a,
                        clos,
                    )
                }
                &mut DynTreeEnum::Ya(ref mut a) => {
                    colfind::for_every_col_pair_seq::<_, T, _, TreeTimerEmpty>(
                        a,
                        clos,
                    )
                }
            };
        }

        ///Find all intersecting pairs.
        ///Optionally return time data of each level of the tree.
        pub fn intersect_every_pair<F: Fn(ColSingle<T>, ColSingle<T>) + Send + Sync>(
            &mut self,
            clos: F,
        ) {
            let clos = self::closure_struct::ColMultiStruct::new(&clos);

            match &mut self.0 {
                &mut DynTreeEnum::Xa(ref mut a) => {
                    colfind::for_every_col_pair::<_, T, _, TreeTimerEmpty>(
                        a,
                        clos,
                    )
                }
                &mut DynTreeEnum::Ya(ref mut a) => {
                    colfind::for_every_col_pair::<_, T, _, TreeTimerEmpty>(
                        a,
                        clos,
                    )
                }
            };
        }

        pub fn intersect_every_pair_seq_debug<F: FnMut(ColSingle<T>, ColSingle<T>)>(
            &mut self,
            clos: F,
        ) -> Bag {
            match &mut self.0 {
                &mut DynTreeEnum::Xa(ref mut a) => {
                    colfind::for_every_col_pair_seq::<_, T, _, TreeTimer2>(
                        a,
                        clos,
                    )
                }
                &mut DynTreeEnum::Ya(ref mut a) => {
                    colfind::for_every_col_pair_seq::<_, T, _, TreeTimer2>(
                        a,
                        clos,
                    )
                }
            }
        }

        pub fn intersect_every_pair_debug<F: Fn(ColSingle<T>, ColSingle<T>) + Send + Sync>(
            &mut self,
            clos: F,
        ) -> Bag {
            let clos = self::closure_struct::ColMultiStruct::new(&clos);

            match &mut self.0 {
                &mut DynTreeEnum::Xa(ref mut a) => {
                    colfind::for_every_col_pair::<_, T, _, TreeTimer2>(a, clos)
                }
                &mut DynTreeEnum::Ya(ref mut a) => {
                    colfind::for_every_col_pair::<_, T, _, TreeTimer2>(a, clos)
                }
            }
        }
    }

}

/*
///The struct that this crate revolves around.
struct DinoTree<'a,A:AxisTrait,T:SweepTrait+'a>(
  DynTree<'a,A,T>
  );

impl<'a,A:AxisTrait,T:SweepTrait+'a> DinoTree<'a,A,T>{
   fn new<JJ:par::Joiner,H:DepthLevel,Z:MedianStrat<Num=T::Num>,K:TreeTimerTrait>(
        rest:&'a mut [T],tc:&mut TreeCache<A,T::Num>,medianstrat:&Z) -> (DinoTree<'a,A,T>,K::Bag) {
      let k=DynTree::new::<JJ,H,Z,K>(rest,tc,medianstrat);
      
      let d=DinoTree(k.0);

      //TODO remove this
      //assert_invariant(&d);

      (d,k.1)

  }
}
*/

//Pub so benches can access
#[cfg(test)]
mod test_support;

#[cfg(test)]
mod dinotree_test;
