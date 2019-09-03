//! ## Overview
//! This crate provides some useful 2d space querying algorithms that you can perform on a dinotree.
//! Checkout the inner demo project to see how all these algorithms can be used.
//!
//! ## Testing
//!
//! Simply testing for correctness doesnt mean the algorithms are working as expected. 
//! The dinotree_alg_data inner project measures the performance of a lot of these algorithms to give an even better feel that they are working
//! as anticipated. 
//!
//! ## Analysis
//! Please see the [dinotree_report](https://github.com/tiby312/dinotree_report) github project, for a writeup of the design and analysis of the algorithms in this project.
//!


#![no_std]

extern crate alloc;

use dinotree::compt;
use dinotree::axgeom;
use dinotree::rayon;

mod inner_prelude {

    pub use alloc::vec::Vec;
    pub use dinotree::prelude::*;    
    pub use compt::LevelIter;
    pub use compt::Depth;
    pub use axgeom::Range;
    pub use crate::*;
    pub use compt::Visitor;
    pub use axgeom::AxisTrait;
    pub use core::marker::PhantomData;
    pub use itertools::Itertools;
    pub use axgeom::Vec2;
    pub use axgeom::vec2;
    pub use core::pin::Pin;
    
}

///Provides functionality to draw the dividers of a dinotree.
pub mod graphics;

pub mod colfind;

///Allows user to intersect the tree with a seperate group of bots.
pub mod intersect_with;

///Contains all k_nearest code.
pub mod k_nearest;

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
