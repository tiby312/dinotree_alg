#![feature(offset_to)]

extern crate axgeom;
extern crate compt;
extern crate rayon;
extern crate pdqselect;
extern crate ordered_float;

mod base_kdtree;
mod tree_alloc;
mod colfind;
pub mod dyntree;
pub mod graphics;
pub mod median;
pub mod oned;
pub mod tools;
pub mod support;
pub mod multirect;

pub use colfind::LL;
pub use base_kdtree::TreeCache;



use compt::LevelDesc;
use axgeom::Rect;

pub trait Bleek{
    type T:SweepTrait;
    fn collide(&mut self,cc:ColPair<Self::T>);
}

pub trait BleekSync:Sync+Copy+Clone{
    type T:SweepTrait+Send;
    fn collide(&self,cc:ColPair<Self::T>);
}




///Returns the level at which a parallel divide and conqur algorithm will switch to sequential
pub trait DepthLevel{
    ///Switch to sequential at this height.
    ///This is highly system dependant of what a "good" level it would be to switch over.
    fn switch_to_sequential(a:LevelDesc)->bool;
}


///A default depth level from which to switch to sequential.
pub struct DefaultDepthLevel;

impl DepthLevel for DefaultDepthLevel{
    fn switch_to_sequential(a:LevelDesc)->bool{
        a.get_depth()>4
    }
}


//The underlying numbers used for the bounding boxes,
//and for the dividers. 
pub trait NumTrait:Ord+Copy+Send+Sync+std::fmt::Debug+Default{}

///Provides a way to destructure an object into a
///reference to a read only bounding box, and a mutable inner struct.
pub trait SweepTrait:Send{
    ///The part of the struct that is allowed to be mutated
    ///during the querying of the tree. It is important that
    ///the bounding boxes not be mutated during querying of the tree
    ///as that would break the invariants of the tree.
    type Inner:Send;

    ///The number trait used to compare rectangles to
    ///find colliding pairs.
    type Num:NumTrait;

    ///Destructure into the bounding box and mutable parts.
    fn get_mut<'a>(&'a mut self)->(&'a Rect<Self::Num>,&'a mut Self::Inner);

    ///Just get the bounding box.
    fn get<'a>(&'a self)->(&'a Rect<Self::Num>,&'a Self::Inner);
}



//use multirect::MultiRectTrait;
///This is the functionality that the collision systems in this crate provide.
///Trait that hides the Axis trait specialization
pub trait DynTreeTrait{
    type T:SweepTrait<Num=Self::Num>;
    type Num:NumTrait;
   fn for_all_in_rect<F:FnMut(ColSingle<Self::T>)>(&mut self,rect:&axgeom::Rect<Self::Num>,fu:&mut F);

   fn for_every_col_pair_seq<H:DepthLevel,F:Bleek<T=Self::T>>
        (&mut self,clos:&mut F,timer:&mut LL);
   
   fn for_every_col_pair<H:DepthLevel,F:BleekSync<T=Self::T>>
        (&mut self,clos:&F,timer:&mut LL);
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
