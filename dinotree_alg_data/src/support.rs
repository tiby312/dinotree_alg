
use std;
use std::time::Duration;

use std::time::Instant;
use crate::inner_prelude::*;







/*

pub type ABSpiralIter=dists::spiral::Spiral;
pub fn create_abspiral(grow:f32)->dists::spiral::Spiral{
    dists::spiral::Spiral::new([0.0,0.0],12.0,grow)
}

pub fn abspiral_aabb_create(pos:Vec2<f32>)->Rect<f32>{
    axgeom::Rect::from_point(pos,vec2same(5.0))
}


pub fn abspiral_aabb_create_isize(pos:Vec2<i32>)->Rect<i32>{
    axgeom::Rect::from_point(pos,vec2same(5))
}

pub fn create_abspiral_isize(grow:f32)->dists::spiral::SpiralInt{
    dists::spiral::Spiral::new([0.0,0.0],12.0,grow).as_isize();
    
}
*/





 
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



pub const COLS:&[&str]=&["blue","green","red","violet","orange","brown","gray","black","pink"];




pub fn instant_to_sec(elapsed:Duration)->f64{
    let secs:f64=elapsed.as_secs() as f64;
    let nano:f64=elapsed.subsec_nanos() as f64;
    secs + nano / 1_000_000_000.0      
}


