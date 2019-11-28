//! ## Overview
//!
//! This crate provides some useful 2D space querying algorithms that you can perform on a dinotree.
//! Checkout the inner demo and data projects to see how all these algorithms can be used.
//!
//! ## Unsafety
//!
//! `MultiRectMut` uses unsafety to allow the user to have mutable references to elements
//! that belong to rectangle regions that don't intersect at the same time.
//!

//#![no_std]

//#[macro_use]
//extern crate alloc;



mod inner_prelude {
    pub use alloc::vec::Vec;
    pub use crate::prelude::*;
    pub use axgeom;    
    pub use axgeom::*;
    pub use compt;
    pub use compt::*;
    pub use core::marker::PhantomData;
    pub use itertools::Itertools;
    pub(crate) use crate::query::tools;
}

///Functions that will compare query output to the naive solution.
pub mod assert;

///aabb broadphase collision detection
pub mod colfind;

///Provides functionality to draw the dividers of a dinotree.
pub mod graphics;

///Contains all k_nearest code.
pub mod k_nearest;


///Contains all raycast code.
pub mod raycast;

///Allows user to intersect the tree with a seperate group of bots.
pub mod intersect_with;

///[EXPERIMENTAL] Contains all nbody code.
#[cfg(feature = "nbody")]
pub mod nbody;


///Contains rect code.
pub mod rect;

///Contains misc tools
pub(crate) mod tools;

