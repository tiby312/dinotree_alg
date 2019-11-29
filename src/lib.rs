//!
//! 
//!
//! ~~~~text
//! 2d Tree Divider Representation:
//!
//!
//!    o   ┆┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┃         ┆         o
//!  ┈┈┈┈┈┈┆     o      o     ┃     o   ┆   o                 o
//!  ───────o─────────────────┃         o┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈
//!                ┆       o  o   o     ┆
//!        o       ┆    o     ┃┈┈┈┈┈o┈┈┈┆       o
//!                ┆   o      ┃         o             o
//!                ┆┈┈┈┈┈┈┈┈┈┈┃         ┆                   o
//!      o         o    o     ┃───────o────────────────────────
//!                ┆          ┃                ┆   o
//!  ┈┈┈┈┈┈┈┈┈┈┈┈┈┈┆      o   o   o            ┆┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈
//!     o          ┆          ┃┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┈┆         o
//!          o     ┆   o      ┃        o       ┆   o
//!                ┆          ┃                ┆
//!
//! Axis alternates every level.
//! Divider placement is placed at the median at each level.
//! Objects that intersect a divider belong to that node.
//! Every divider keeps track of how thick a line would have to be
//! to 'cover' all the bots it owns.
//! All the objects in a node are sorted along that node's axis.
//!
//!
//! ~~~~
//! # Overview
//!
//! Provides the dinotree data structure and ways to traverse it. No actual query algorithms are provided in this crate.
//! Only the data structure and a way to construct and traverse it are provided in this crate.
//!
//!
//! ## Data Structure
//!
//! Using this crate, the user can create three flavors of the same fundamental data structure. They each 
//! have different characteristics that may make you want to use them over the others. You can make a dinotree
//! composed of the following:
//!
//!
//! + `(Rect<N>,&mut T)` is the most well rounded and most performant in most cases.
//! The aabb's themselves don't have a level of indirection. Broad-phase
//! algorithms need to look at these very often. It's only when these algorithms
//! detect a intersection do they need to look further, which doesnt happen as often.
//! So a level of indirection here is not so bad. The fact that T is a pointer, also
//! means that more aabb's will be in cache at once, further speeding up algorithms
//! that need to look at the aabb's very often.
//!
//!
//! + `(Rect<N>,T)` performs slightly better during the querying phase, but suffers
//! during the construction phase. There is also no easy way to return the elements back
//! to their original positions on destructing of the tree (something you don't need to worry about with pointers).
//! One benefit of using this tree, is that it owns the elements completely, so there are no lifetime references to worry about.
//! The performance of this type of tree is also heavily influenced by the size of T.
//!
//! + `&mut (Rect<N>,T)` has comparable tree construction times to `(Rect<N>,&mut T)` given that we are just sorting and swapping
//! pointers, but there is no cache-coherence during the query phase, so this can 
//! cause real slow down to query algorithms if there are many overlapping elements.
//!
//!
//! ## DinoTreeOwned
//!
//! A verion of the tree where the tree owns the elements in side of it.
//! The user is encouraged to use the lifetimed version, though, as that does not use unsafe{}.
//! But this might mean that the user has to re-construct the tree more often than it needs to be.
//! It is composed internally of the equivalent to `(Rect<N>,&mut T)`, the most well-rounded data layout as
//! described above. 
//!
//! ## NotSorted
//!
//! For comparison, a normal kd-tree is provided by `NotSorted`. In this tree, the elements are not sorted
//! along an axis at each level. Construction of `NotSorted` is faster than `DinoTree` since it does not have to
//! sort bots that belong to each node along an axis. But most query algorithms can usually take advantage of this
//! extra property.
//!
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
//! TODO talk about Rect<N> properties.
//!
//! If you insert aabb's with zero width or zero height, it is unspecified behavior (but still safe).
//! It is expected that all elements in the tree take up some area. This is not inteded to be used
//! as a "point" tree. Using this tree for a point tree would be inefficient since the data layout
//! assumes there is a aabb, which is composed of 4 numbers when a point would be just 2.
//!
//! That said, an aabb is composed of half-open ranges [start,end). So one could simulate a "point",
//! by putting in a very small epsilon value to ensure that end>start.
//! 
//!

#![no_std]

#[macro_use]
extern crate alloc;
extern crate is_sorted;
extern crate pdqselect;

///Prelude to include by using: pub use dinotree::prelude::*
pub mod prelude{
    pub use crate::*;
    pub use crate::elem::*;
    pub use crate::bbox::*;  
    pub use crate::query::*;
    pub use crate::par;
    pub use crate::tree::node::*;
}


mod inner_prelude {
    pub use axgeom::*;
    pub use core::iter::*;
    pub use core::marker::PhantomData;
    pub use alloc::vec::Vec;
    pub(crate) use super::*;
    pub(crate) use compt::Visitor;
    pub(crate) use crate::tree;
    pub(crate) use crate::tree::analyze_inner::*;

    pub(crate) use crate::tree::*;
    pub(crate) use crate::elem::*;
    pub(crate) use crate::bbox::*;
    pub(crate) use crate::par;
}

pub mod query;


use axgeom::*;

///Contains code to check the data structure is valid.
mod assert_invariants;

///Contains generic code used in all dinotree versions
pub use self::tree::*;
mod tree;

///Contains code to write generic code that can be run in parallel, or sequentially. The api is exposed
///in case users find it useful when writing parallel query code to operate on the tree.
pub mod par;

///A collection of 1d functions that operate on lists of 2d objects.
mod oned;

///Provies a slice that produces BBox's where users can only interact
///with through the HasInner trait so as to protect the invariants of the tree.
pub mod elem;

///A collection of different bounding box containers.
pub mod bbox;

pub mod util;




///The underlying number type used for the dinotree.
///It is auto implemented by all types that satisfy the type constraints.
///Notice that no arithmatic is possible. The tree is constructed
///using only comparisons and copying.
pub trait NumTrait: Ord + Copy + Send + Sync {}
impl<T> NumTrait for T where T: Ord + Copy + Send + Sync {}


///Trait to signify that this object has an axis aligned bounding box.
///get() must return a aabb with the same value in it while the element
///is in the dinotree. This is hard for the user not to do, this the user
///does not have &mut self, and the aabb is implied to belong to self.
///But it is still possible through the use of static objects or RefCell/ Mutex, etc.
///Using this type of methods the user could make different calls to get()
///return different aabbs.
///This is unsafe since we allow query algorithms to assume the following:
///If two object's aabb's don't intersect, then they can be mutated at the same time.
pub unsafe trait HasAabb{
    type Num: NumTrait;
    fn get(&self) -> &Rect<Self::Num>;
}

///Trait exposes an api where you can return a read-only reference to the axis-aligned bounding box
///and at the same time return a mutable reference to a seperate inner section.
pub trait HasInner:HasAabb{
    type Inner;
    #[inline(always)]
    fn inner_mut(&mut self)->&mut Self::Inner{
        self.get_inner_mut().1
    }
    #[inline(always)]
    fn inner(&self)->&Self::Inner{
        self.get_inner().1
    }
    fn get_inner(&self)->(&Rect<Self::Num>,&Self::Inner);
    fn get_inner_mut(&mut self)->(&Rect<Self::Num>,&mut Self::Inner);
}







