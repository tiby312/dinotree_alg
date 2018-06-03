use inner_prelude::*;
pub use dinotree_inner::support::NumWrapper;
//pub use dinotree_inner::support::BBox;
use dinotree_inner::HasAabb;
use axgeom::*;


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