//! An iterative mulithreaded hybrid kdtree/mark and sweep algorithm used for broadphase detection.
//! ## Goal
//! To provide a fast and simple to use broad-phase collision system.
//!
//! ## Notes                      
//!
//! Checkout included examples. 
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


extern crate axgeom;
extern crate compt;
extern crate ordered_float;
extern crate pdqselect;
extern crate rayon;
extern crate unsafe_unwrap;
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



mod inner_prelude {
    pub use dinotree_inner::*;
    pub use axgeom::Axis;
    pub use compt::LevelIter;
    pub use compt::Depth;
    pub use axgeom::Range;
    pub use ::*;
    pub use compt::CTreeIterator;
    //pub use par;
    pub use axgeom::AxisTrait;
    pub use std::marker::PhantomData;
    pub use NumTrait;
    pub use ::*;
}

///Provides functionality to draw the dividers of a dinotree.
pub mod graphics;

///Contains convenience structs.
pub mod support;

///Contains query code
pub mod colfind;

pub mod k_nearest;

pub mod nbody;

pub mod raycast;

pub mod find_element;

pub mod rect;

pub mod multirect;

///A collection of 1d functions that operate on lists of 2d objects.
mod oned;

///Contains misc tools
mod tools;

//pub use rects::Rects;
//pub use rects::RectIntersectError;
//mod rects;

//use dinotree_inner::support::DefaultDepthLevel;
//pub use dinotree_inner::AABBox;
pub use dinotree_inner::NumTrait;
pub use dinotree_inner::HasAabb;
use dinotree_inner::treetimer::TreeTimerTrait;
use dinotree_inner::par;
use axgeom::Rect;
use axgeom::XAXISS;
use axgeom::YAXISS;
//use colfind::ColMulti;
use smallvec::SmallVec;
use dinotree_inner::treetimer::TreeTimer2;
use dinotree_inner::treetimer::TreeTimerEmpty;
//use dinotree_inner::compute_tree_height;
use axgeom::AxisTrait;



pub struct BBoxDet<'a,Nu:NumTrait+'a,T:'a>{
    pub rect:&'a Rect<Nu>,
    pub inner:&'a mut T
}


///A generic container that implements the kdtree trait.
#[derive(Debug,Clone,Copy)]
pub struct BBox<Nu:NumTrait,T>{
    pub rect:Rect<Nu>,
    pub inner:T
}

impl<Nu:NumTrait,T> BBox<Nu,T>{
    pub fn destruct<'a>(&'a mut self)->BBoxDet<'a,Nu,T>{
        BBoxDet{rect:&self.rect,inner:&mut self.inner}
    }
}

impl<Nu:NumTrait,T> HasAabb for BBox<Nu,T>{
    type Num=Nu;
    
    ///Destructue into the bounding box and inner part.
    fn get<'a>(&'a self)->&Rect<Nu>{
        &self.rect
    }
}

/*
mod test{
    fn test(){

        let bot:BBox<isize,Bot>=Vec::new();
        
        let mut tree=DynTree::new(bot);

        tree.for_every_mut(|tree,bot|{
            tree.k_nearest([50,50],3,|bb|{
                bot.repel_each_other(bb);
            });

            tree.ray_cast([50,50],ray,|bb|{

            });
        });
    }
}

pub struct DynTreeExp<'a,A:AxisTrait+'a,N:NumTrait+'a,T+'a>{
    a:&'a mut DynTree<A,(),BBox<N,T>>,
    bot_to_ignore:&'a mut T
}
impl DynTreeExp{
    pub fn k_nearest(&mut self,point:[N;2],num_find:usize,func:impl FnMut(BBoxDet<'a,N,T>)){

    }
    pub fn raycast(&mut self,){

    }
}

pub fn for_every_mut <A:AxisTrait,N:NumTrait,T>(tree:&mut DynTree<A,(),BBox<N,T>>,mut func:impl FnMut(&mut DynTreeExp<A,(),BBot<N,T>>,BBoxDet<N,T>)){
    use compt::CTreeIterator;
    for b in tree.get_iter_mut().dfs_preorder_iter().flat_map(|a|a.range.iter_mut()){
        func(b.destruct());
    }
}
*/

pub fn for_every<A:AxisTrait,T:HasAabb>(tree:&DynTree<A,(),T>,mut func:impl FnMut(&T)){
    use compt::CTreeIterator;
    for b in tree.get_iter().dfs_preorder_iter().flat_map(|a|a.range.iter()){
        func(b);
    }
}



/*
fn create_callback<T:UnchangingAabb>(user_supplied:impl FnMut((&Rect<T::Num>,&mut T::Inner),(&Rect<T::Num>,&mut T::Inner))){

}
*/

/*
///Represents a destructured SweepTrait into the immutable bounding box reference,
///and the mutable reference to the rest of the object.
pub struct ColSingle<'a, T: SweepTrait + 'a> {
    pub rect: &'a AABBox<T::Num>,
    pub inner: &'a mut T::Inner,
}
*/

use dinotree_inner::DynTree;

//pub use ba::DinoTree;
//pub(crate) use ba::DynTreeEnum;

use std::marker::PhantomData;
