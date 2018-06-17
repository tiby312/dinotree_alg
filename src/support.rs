use inner_prelude::*;
pub use dinotree_inner::support::NumWrapper;
//pub use dinotree_inner::support::BBox;
use dinotree_inner::HasAabb;
use axgeom::*;



///This object provides some more protection against a user
///incorrectly changing an objects aabb after it has been inserted into the
///tree.
///Note that this object has the Rect field visible and does not implement HasAAbb.
///On the other hand BBox doesnt have the rect field visible and DOES imeplement Has Aabb.
#[derive(Debug,Clone,Copy)]
pub struct BBoxVisible<Nu:NumTrait,T>{
    pub rect:Rect<Nu>,
    pub inner:T
}
impl<Nu:NumTrait,T> BBoxVisible<Nu,T>{
    pub fn into_bbox(self)->BBox<Nu,T>{
        BBox{rect:self.rect,inner:self.inner}
    }
}

///A generic container that implements the kdtree trait.
#[derive(Debug,Clone,Copy)]
pub struct BBox<Nu:NumTrait,T>{
    rect:Rect<Nu>,
    pub inner:T
}

//Only the version that hides rect from the user implements HasAabb
impl<Nu:NumTrait,T> HasAabb for BBox<Nu,T>{
    type Num=Nu;
    
    ///Destructue into the bounding box and inner part.
    fn get<'a>(&'a self)->&Rect<Nu>{
        &self.rect
    }
}

use for_every_nearest::HasCenter;
impl<Nu:NumTrait,T:HasCenter<Num=Nu>> HasCenter for BBox<Nu,T>{
    type Num=Nu;
    fn get_center(&self)->&[Self::Num;2]{
        self.inner.get_center()
    }
}
impl<Nu:NumTrait,T> BBox<Nu,T>{
    pub fn new(rect:Rect<Nu>,inner:T)->BBox<Nu,T>{
        BBox{rect,inner}
    }
    pub fn into_visible(self)->BBoxVisible<Nu,T>{
        BBoxVisible{rect:self.rect,inner:self.inner}
    }
}
    
    



/*
///A generic container that implements the kdtree trait.
#[derive(Debug)]
pub struct BBox<Nu:NumTrait,T:Send+Sync>{
    pub rect:AABBox<Nu>,
    pub val:T
}

impl<Nu:NumTrait,T:Send+Sync> SweepTrait for BBox<Nu,T>{
    type Inner=T;
    type Num=Nu;

    ///Destructure into the bounding box and mutable parts.
    fn get_mut<'a>(&'a mut self)->(&'a AABBox<Nu>,&'a mut Self::Inner){
        (&self.rect,&mut self.val)
    }
    
    ///Destructue into the bounding box and inner part.
    fn get<'a>(&'a self)->(&'a AABBox<Nu>,&'a Self::Inner){
        (&self.rect,&self.val)
    }
}

impl<Nu:NumTrait,T:Send+Sync+Clone> Clone for BBox<Nu,T>{
    fn clone(&self)->BBox<Nu,T>{
        BBox{rect:self.rect.clone(),val:self.val.clone()}
    }
}
impl<Nu:NumTrait,T:Send+Sync> BBox<Nu,T>{

    #[inline(always)]
    pub fn new(val:T,r:AABBox<Nu>)->BBox<Nu,T>{
        BBox{rect:r,val:val}
    }
}

*/