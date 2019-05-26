//! ## Overview
//! This crate provides some useful 2d space querying algorithms that you can perform on a dinotree.
//!
//! ## Notes                      
//!
//! Checkout the inner demo project to see how all these algorithms can be used.
//!
//! ## Testing
//!
//! A quick way to verify to a good level a lot of these algorithms is visually, so the demo inner project is used
//! to verify correctness of a lot of these algorithms. Coupled with these visual demo's, comparisons against the naive algorithm is used
//! to catch more subtle corner case bugs.
//! 
//! Simply testing for correctness doesnt mean the algorithms are working as expected. 
//! The dinotree_alg_data inner project measures the performance of a lot of these algorithms to give an even better feel that they are working
//! as anticipated. 
//!
//! ## Analysis
//! Please see the [dinotree_report](https://github.com/tiby312/dinotree_report) github project, for a writeup of the design and analysis of the algorithms in this project.
//!

extern crate axgeom;
extern crate compt;
extern crate rayon;
extern crate smallvec;
extern crate dinotree;
extern crate itertools;

#[cfg(all(feature = "unstable", test))]
extern crate test;

extern crate is_sorted;



mod inner_prelude {
    pub use dinotree::advanced::*;
    pub use dinotree::*;
    pub use dinotree::copy::*;
    pub use dinotree::nocopy::*;
    pub use dinotree::notsorted::*;
    pub use compt::LevelIter;
    pub use compt::Depth;
    pub use axgeom::Range;
    pub use crate::*;
    pub use compt::Visitor;
    pub use axgeom::AxisTrait;
    pub use std::marker::PhantomData;
    pub use itertools::Itertools;
}

///Provides functionality to draw the dividers of a dinotree.
pub mod graphics;

pub mod colfind;

///Allows user to intersect the tree with a seperate group of bots.
pub mod intersect_with;

///Contains all k_nearest code.
pub mod k_nearest;

/*
///Contains all for_every_nearest code. *Experimental*
pub mod for_every_nearest;
*/

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

///Contains misc tools
mod tools;

use axgeom::Rect;
use smallvec::SmallVec;
