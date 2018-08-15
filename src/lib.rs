//! ## Goal
//! To provide a fast and simple to use broad-phase collision system.
//!
//! ## Notes                      
//!
//! Checkout the demo project to see how all these algorithms can be used.
//!
//! The mutable reference to each element in the callback functions do not point to elements
//! in the user supplied slice of elements. The elements are internally unsafely copied directly into a tree structure
//! and then unsafely copied back into the slice at the end. So do not try storing the mutable references as pointers
//! in the callback functions since they would point to unallocated memory once the tree is destroyed.
//!
//! ## Use of unsafety           
//!
//! The multirect api uses unsafety internally. We unsafely convert the refernces returned by the rect query
//! closure to have a longer lifetime.
//! This allows the user to store mutable references of non intersecting rectangles at the same time. 
//! The multirect api panics at run time if the user attemps to query
//! rectangles that intersect. This protects the invariant at runtime. So it this use unsafety can be hidden by the api.
//!
//! The bots are unsafely copied into a tree, and then usafely copied back out. The algorithm ensures
//! That even though the ordering is different, this is a bijection between the two sets.
//! So we can safely hide this unsafety from the user.
//! The bots are copied back in the trees drop() method. If the user panics inside of a callback function,
//! The changes to the bots up until that more during the traversal of the tree will take effect when the 
//! trees drop() occurrs.
//!
//! The sequential version of the pair intersection uses unsafe{} to re-use code from the parallel version.
//! That is protected at runtime. It will panic if the parallel version tries to copy the closure.
//!
//! Unsafety is used to construct the special variable node size tree structure that is populated with dsts.
//!

#![feature(iterator_step_by)]
#![feature(test)]
#![feature(trusted_len)]




extern crate axgeom;
extern crate compt;
extern crate rayon;

//So that we can import smallvec creation macro
#[macro_use]
extern crate smallvec;
extern crate dinotree_inner;

#[cfg(test)]
extern crate num;
#[cfg(test)]
extern crate cgmath;
#[cfg(test)]
extern crate collision;
#[cfg(test)]
extern crate rand;
#[cfg(test)]
extern crate test;

extern crate is_sorted;



mod inner_prelude {
    pub use dinotree_inner::*;
    pub use compt::LevelIter;
    pub use compt::Depth;
    pub use axgeom::Range;
    pub use ::*;
    pub use compt::CTreeIterator;
    //pub use par;
    pub use axgeom::AxisTrait;
    pub use std::marker::PhantomData;
    
    pub use ::*;
}

///Provides functionality to draw the dividers of a dinotree.
pub mod graphics;

///Contains query code
pub mod colfind;

///Allows use to intersect the tree with a seperate group of bots.
pub mod intersect_with;

///Contains all k_nearest code.
pub mod k_nearest;

///Contains all for_every_nearest code.
pub mod for_every_nearest;

///Contains all nbody code.
pub mod nbody;

///Contains all raycast code.
pub mod raycast;

///Contains find element code.
pub mod find_element;

///Contains rect code.
pub mod rect;

///Contains multirect code.
pub mod multirect;

///A collection of 1d functions that operate on lists of 2d objects.
mod oned;

///Contains misc tools
pub mod tools;


use compt::timer::TreeTimerTrait;
use axgeom::Rect;
use smallvec::SmallVec;
use compt::timer::TreeTimerEmpty;
