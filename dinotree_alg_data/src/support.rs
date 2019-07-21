use axgeom::*;
use std;
use std::time::Duration;



pub const COLS:&[&str]=&["blue","green","red","violet","orange","pink","gray","brown","black"];




pub fn instant_to_sec(elapsed:Duration)->f64{
    use num_traits::cast::AsPrimitive;
    let secs:f64=elapsed.as_secs().as_();
    let nano:f64=elapsed.subsec_nanos().as_();
    secs + nano / 1_000_000_000.0      
}


use dinotree::*;

///Like dinotree_inner::BBox, but with a public constructor
#[derive(Copy,Clone)]
pub struct BBoxDemo<N:NumTrait,T>{
    rect:Rect<N>,
    pub inner:T
}
impl<N:NumTrait,T> BBoxDemo<N,T>{
    pub fn new(rect:Rect<N>,inner:T)->BBoxDemo<N,T>{
        BBoxDemo{rect,inner}
    }
}

use std::fmt::Formatter;
use std::fmt::Debug;
impl<N:NumTrait+Debug,T:Debug> Debug for BBoxDemo<N,T>{
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result{
        self.rect.fmt(f)?;
        self.inner.fmt(f)
    }
}

unsafe impl<N:NumTrait,T> HasAabb for BBoxDemo<N,T>{
    type Num=N;
    fn get(&self)->&Rect<Self::Num>{
        &self.rect
    }
}




