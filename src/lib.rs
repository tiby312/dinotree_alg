//! # Overview
//!
//! This crate hopes to provide an efficient 2D space partitioning data structure and useful query algorithms to perform on it
//! in a hopefully simple cohesive api.
//! It is a hybrid between a [KD Tree](https://en.wikipedia.org/wiki/K-d_tree) and [Sweep and Prune](https://en.wikipedia.org/wiki/Sweep_and_prune).
//! Uses `no_std`, but uses the `alloc` crate.
//! Please see the [dinotree-book](https://dinotree-book.netlify.com) which is a work in-progress high level explanation and analysis
//! of this crate.
//!
//! ## Screenshot
//!
//! Screenshot from the dinotree_alg_demo inner project from the [github repo of this crate](https://github.com/tiby312/dinotree_alg).
//! ![](https://raw.githubusercontent.com/tiby312/dinotree_alg/master/assets/screenshot.gif)
//!
//! ## Data Structure
//!
//! Using this crate, the user can create three flavors of the same fundamental data structure.
//! The different characteristics are exlored more in depth in the book mentioned in the overview section.
//!
//! + `(Rect<N>,&mut T)` *recommended
//! + `(Rect<N>,T)`
//! + `&mut (Rect<N>,T)`
//!
//! ## DinoTreeOwned
//!
//! A verion of the tree where the tree owns the elements in side of it.
//! The user is encouraged to use the lifetimed version, though, as that does not use unsafe{}.
//! But this might mean that the user has to re-construct the tree more often than it needs to be.
//! It is composed internally of the equivalent to `(Rect<N>,&mut T)`, the most well-rounded data layout as
//! described above.
//!
//! ## User Protection
//!
//! A lot is done to forbid the user from violating the invariants of the tree once constructed
//! while still allowing them to mutate elements of the tree. The user can mutably traverse down the tree
//! with a `VistrMut` and `ElemSliceMut`, but the elements that are returned have already been destructured in such a way
//! that the user only has read-only access to the `Rect<N>`, even if they do have write access to the inner `T`.
//!
//!
//! ## Usage Guidlines
//!
//! The AABB struct that the user must use is from the [axgeom](https://crates.io/crates/axgeom) crate.
//!
//! If you insert aabb's with zero width or zero height, it is unspecified behavior (but still safe).
//! It is expected that all elements in the tree take up some area. This is not inteded to be used
//! as a "point" tree. Using this tree for a point tree would be inefficient anyway since the data layout
//! assumes there is a aabb, which is composed of 4 numbers when a point would be just 2.
//!
//! That said, an aabb is composed of half-open ranges [start,end). So one could simulate a "point",
//! by putting in a very small epsilon value to ensure that end>start.
//!
//! ## Unsafety
//!
//! `MultiRectMut` uses unsafety to allow the user to have mutable references to elements
//! that belong to rectangle regions that don't intersect at the same time. This is why
//! the Aabb trait is unsafe.
//!


#![no_std]

#[macro_use]
extern crate alloc;
extern crate is_sorted;
extern crate pdqselect;

///Prelude to include by using: pub use dinotree::prelude::*
pub mod prelude {
    pub use crate::bbox::*;
    pub use crate::par;
    //pub use crate::pmut::*;
    pub use crate::query::*;
    pub use crate::tree::node::*;
    pub use crate::*;
}

mod inner_prelude {
    pub(crate) use super::*;
    pub(crate) use crate::tree;
    pub(crate) use crate::tree::analyze::*;
    pub use alloc::vec::Vec;
    pub use axgeom::*;
    pub(crate) use compt::Visitor;
    pub use core::iter::*;
    pub use core::marker::PhantomData;

    pub(crate) use crate::bbox::*;
    pub(crate) use crate::par;
    pub(crate) use crate::pmut::*;
    pub(crate) use crate::tree::*;
}

pub mod query;

use axgeom::*;

///Contains generic code used in all dinotree versions
pub use self::tree::*;
mod tree;

///Contains code to write generic code that can be run in parallel, or sequentially. The api is exposed
///in case users find it useful when writing parallel query code to operate on the tree.
pub mod par;

///A collection of 1d functions that operate on lists of 2d objects.
mod oned;

pub mod pmut;

///A collection of different bounding box containers.
pub mod bbox;

///Generic slice utillity functions.
pub mod util;

///The underlying number type used for the dinotree.
///It is auto implemented by all types that satisfy the type constraints.
///Notice that no arithmatic is possible. The tree is constructed
///using only comparisons and copying.
pub trait Num: Ord + Copy + Send + Sync {}
impl<T> Num for T where T: Ord + Copy + Send + Sync {}

///Trait to signify that this object has an axis aligned bounding box.
///get() must return a aabb with the same value in it while the element
///is in the dinotree. This is hard for the user not to do, this the user
///does not have &mut self, and the aabb is implied to belong to self.
///But it is still possible through the use of static objects or RefCell/ Mutex, etc.
///Using this type of methods the user could make different calls to get()
///return different aabbs.
///This is unsafe since we allow query algorithms to assume the following:
///If two object's aabb's don't intersect, then they can be mutated at the same time.
pub unsafe trait Aabb {
    type Num: Num;
    fn get(&self) -> &Rect<Self::Num>;
}

unsafe impl<N: Num> Aabb for Rect<N> {
    type Num = N;
    fn get(&self) -> &Rect<Self::Num> {
        self
    }
}

///Trait exposes an api where you can return a read-only reference to the axis-aligned bounding box
///and at the same time return a mutable reference to a seperate inner section.
pub trait HasInner: Aabb {
    type Inner;
    #[inline(always)]
    fn inner_mut(&mut self) -> &mut Self::Inner {
        self.get_inner_mut().1
    }
    #[inline(always)]
    fn inner(&self) -> &Self::Inner {
        self.get_inner().1
    }
    fn get_inner(&self) -> (&Rect<Self::Num>, &Self::Inner);
    fn get_inner_mut(&mut self) -> (&Rect<Self::Num>, &mut Self::Inner);
}


