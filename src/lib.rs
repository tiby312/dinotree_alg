#![feature(offset_to)]

extern crate axgeom;
extern crate compt;
extern crate rayon;
extern crate pdqselect;
extern crate ordered_float;

mod base_kdtree;
mod tree_alloc;
mod colfind;
mod dyntree;
mod oned;

pub mod graphics;
pub mod median;
pub mod tools;
pub mod support;
pub mod multirect;
pub mod treetimer;


pub use dyntree::DynTree;
//The TreeCache object is updated during the base kd tree construction.
//So TreeCache and KdTree are tied together.
//On the otherhand, we dont expose KdTree since it is only used
//intermediately in order to construct a DynTree.
pub use base_kdtree::TreeCache;

use compt::LevelDesc;
use axgeom::Rect;
use treetimer::*;

///Returns the level at which a parallel divide and conqur algorithm will switch to sequential
pub trait DepthLevel{
    ///Switch to sequential at this height.
    fn switch_to_sequential(a:LevelDesc)->bool;
}

///The underlying number type used for the bounding boxes,
///and for the dividers. 
pub trait NumTrait:Ord+Copy+Send+Sync+std::fmt::Debug+Default{}

///The interface through which the tree interacts with the objects being inserted into it.
pub trait SweepTrait:Send{
    ///The part of the object that is allowed to be mutated
    ///during the querying of the tree. It is important that
    ///the bounding boxes not be mutated during querying of the tree
    ///as that would break the invariants of the tree. (it might need to be moved
    ///to a different node)
    type Inner:Send;

    ///The number trait used to compare rectangles to
    ///find colliding pairs.
    type Num:NumTrait;

    ///Destructure into the bounding box and mutable parts.
    fn get_mut<'a>(&'a mut self)->(&'a Rect<Self::Num>,&'a mut Self::Inner);

    ///Destructue into the bounding box and inner part.
    fn get<'a>(&'a self)->(&'a Rect<Self::Num>,&'a Self::Inner);
}

///The interface through which users can use the tree for what it is for, querying.
pub trait DynTreeTrait{
   type T:SweepTrait<Num=Self::Num>;
   type Num:NumTrait;

   ///Finds all objects strictly within the specified rectangle.
   fn for_all_in_rect<F:FnMut(ColSingle<Self::T>)>(&mut self,rect:&axgeom::Rect<Self::Num>,fu:&mut F);

   ///Find all objects who's bounding boxes intersect in parallel.
   fn for_every_col_pair<H:DepthLevel,F:Fn(ColPair<Self::T>)+Sync,K:TreeTimerTrait>
        (&mut self,clos:F)->K::Bag;

   ///Find all objects who's bounding boxes intersect sequentially. 
   fn for_every_col_pair_seq<F:FnMut(ColPair<Self::T>),K:TreeTimerTrait>
        (&mut self,mut clos:F)->K::Bag;
}

///This contains the destructured SweepTrait for a colliding pair.
///The rect is read only while T::Inner is allowed to be mutated.
pub struct ColPair<'a,T:SweepTrait+'a>{
    pub a:(&'a Rect<T::Num>,&'a mut T::Inner),
    pub b:(&'a Rect<T::Num>,&'a mut T::Inner)
}

///Similar to ColPair, but for only one SweepTrait
pub struct ColSingle<'a,T:SweepTrait+'a>(pub &'a Rect<T::Num>,pub &'a mut T::Inner);

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
