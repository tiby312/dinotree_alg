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
//#![cfg_attr(not(feature="std"), no_std)]
//#[cfg(all(feature="alloc", not(feature="std")))]
extern crate alloc;


mod inner_prelude {
    pub use dinotree::rayon;
    pub use alloc::vec::Vec;
    pub use dinotree::prelude::*;
    pub use dinotree::axgeom;    
    pub use dinotree::axgeom::*;
    pub use dinotree::compt;
    pub use dinotree::compt::*;
    pub use core::marker::PhantomData;
    pub use itertools::Itertools;
    pub(crate) use crate::tools;
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

///Contains rect code.
pub mod rect;

///Contains misc tools
pub(crate) mod tools;
