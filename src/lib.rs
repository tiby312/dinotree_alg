//! ## Goal
//! To provide provide useful algorithms that you can perform on a dinotree.
//!
//! ## Notes                      
//!
//! Checkout the demo project to see how all these algorithms can be used.
//!
//! ## Testing
//!
//! A quick way to verify to a good level a lot of these algorithms is visually, so the demo inner project is used
//! to verify correctness of a lot of these algorithms. Some of the more complicated algorithm demos additionally have an option to
//! verify the algorithm against the naive algorithm. This is to catch more subtle corner case bugs.
//! The demo projects are not deterministic. Its up to the user to try and trigger corner cases by moving their mouse around.
//! More rigorous automated + visual testing and benchmarking is to be desired. Once the design stabalizes, this will be a priority.
//! 

#![feature(iterator_step_by)]
#![feature(test)]
#![feature(trusted_len)]

extern crate axgeom;
extern crate compt;
extern crate rayon;

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
    pub use dinotree_inner::advanced::*;
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

///Allows user to intersect the tree with a seperate group of bots.
pub mod intersect_with;

///Contains all k_nearest code.
pub mod k_nearest;

///Contains all for_every_nearest code. *Experimental*
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
mod tools;

use axgeom::Rect;
use smallvec::SmallVec;
