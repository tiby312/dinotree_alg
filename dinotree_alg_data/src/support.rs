use axgeom::*;
use std;
use std::time::Duration;

use std::time::Instant;
use dinotree::advanced::Splitter;


 
fn into_secs(elapsed: std::time::Duration) -> f64 {
    (elapsed.as_secs() as f64) + (f64::from(elapsed.subsec_nanos()) / 1_000_000_000.0)
}


///Measure the time each level of a recursive algorithm takes that supports the Splitter trait.
///Note that the number of elements in the returned Vec could be less than the height of the tree.
///This can happen if the recursive algorithm does not recurse all the way to the leafs because it
///deemed it not necessary.
#[derive(Default)]
pub struct LevelTimer {
    levels: Vec<f64>,
    time: Option<Instant>,
}

impl LevelTimer {
    #[inline]
    pub fn new() -> LevelTimer {
        LevelTimer {
            levels: Vec::new(),
            time: None,
        }
    }

    #[inline]
    pub fn into_inner(self) -> Vec<f64> {
        self.levels
    }
    #[inline]
    fn node_end_common(&mut self) {
        let time = self.time.unwrap();

        let elapsed = time.elapsed();
        //println!("elapsed={:?}",into_secs(elapsed));
        self.levels.push(into_secs(elapsed));
        self.time = None;
    }
}
impl Splitter for LevelTimer {
    #[inline]
    fn div(&mut self) -> Self {
        self.node_end_common();

        let length = self.levels.len();

        LevelTimer {
            levels: core::iter::repeat(0.0).take(length).collect(),
            time: None,
        }
    }
    #[inline]
    fn add(&mut self, a: Self) {
        let len = self.levels.len();
        for (a, b) in self.levels.iter_mut().zip(a.levels.iter()) {
            *a += *b;
        }
        if len < a.levels.len() {
            self.levels.extend_from_slice(&a.levels[len..]);
        }
    }
    #[inline]
    fn node_start(&mut self) {
        assert!(self.time.is_none());
        self.time = Some(Instant::now());
    }
    #[inline]
    fn node_end(&mut self) {
        self.node_end_common();
    }
}



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




