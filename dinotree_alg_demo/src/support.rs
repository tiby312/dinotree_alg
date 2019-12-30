use std;

pub mod prelude {

    pub use crate::Demo;

    pub use super::f32n;
    pub use ordered_float::NotNan;
    pub use very_simple_2d::*;
    pub use super::ColorGenerator;
    pub use axgeom::*;
    pub use duckduckgeo::bot::*;
    pub use duckduckgeo::F32n;
    pub use duckduckgeo::*;
    pub use very_simple_2d::*;

    pub use dists;

    pub use dinotree_alg::analyze;
    pub use dinotree_alg::dinotree_owned::*;
    pub use dinotree_alg::prelude::*;
    pub use dists::uniform_rand::UniformRandGen;
    pub use duckduckgeo::array2_inner_into;
}

pub struct ColorGenerator {
    rgb: [u8; 3],
}
impl ColorGenerator {
    pub fn new() -> ColorGenerator {
        ColorGenerator {
            rgb: [50, 100, 200],
        }
    }
}

impl std::iter::FusedIterator for ColorGenerator {}
impl Iterator for ColorGenerator {
    type Item = [u8; 3];
    fn next(&mut self) -> Option<Self::Item> {
        self.rgb[0] = ((self.rgb[0] as usize + 2) % 256) as u8;
        self.rgb[1] = ((self.rgb[1] as usize + 3) % 256) as u8;
        self.rgb[2] = ((self.rgb[2] as usize + 5) % 256) as u8;
        Some(self.rgb)
    }
}

use duckduckgeo::F32n;
use ordered_float::NotNan;
pub fn f32n(a: f32) -> F32n {
    NotNan::new(a).unwrap()
}
