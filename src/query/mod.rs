//! ## Overview
//!
//! This module provides some useful 2D space querying algorithms that you can perform on a dinotree.
//! Checkout the inner demo and data projects to see how all these algorithms can be used.
//!
//! ## Unsafety
//!
//! `MultiRectMut` uses unsafety to allow the user to have mutable references to elements
//! that belong to rectangle regions that don't intersect at the same time.
//!


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


pub use crate::query::raycast::{Ray,RayIntersectResult,RayTrait,RayCastResult};
pub use crate::query::k_nearest::{Knearest,KnearestResult};
pub use crate::query::graphics::DividerDrawer;
#[cfg(feature = "nbody")]
pub use crate::query::nbody::NodeMassTrait;



///aabb broadphase collision detection
pub(crate) mod colfind;

///Provides functionality to draw the dividers of a dinotree.
pub(crate) mod graphics;

///Contains all k_nearest code.
pub(crate) mod k_nearest;


///Contains all raycast code.
pub(crate) mod raycast;

///Allows user to intersect the tree with a seperate group of bots.
pub(crate) mod intersect_with;

///[EXPERIMENTAL] Contains all nbody code.
#[cfg(feature = "nbody")]
pub(crate) mod nbody;

///Contains rect code.
pub(crate) mod rect;

///Contains misc tools
pub(crate) mod tools;

