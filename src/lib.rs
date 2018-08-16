//! ## Goal
//! To provide a fast and simple to use broad-phase collision system.
//!
//! ## Notes                      
//!
//! Checkout the demo project to see how all these algorithms can be used.
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
pub mod tools;


use compt::timer::TreeTimerTrait;
use axgeom::Rect;
use smallvec::SmallVec;
use compt::timer::TreeTimerEmpty;
