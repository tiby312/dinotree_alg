use std;
use axgeom::*;

pub mod prelude {

    pub use crate::DemoSys;
    pub use ordered_float::NotNan;
    pub(crate) use piston_window;
    pub use piston_window::*;
    
    pub(crate) use crate::support::*;
    
    pub use duckduckgeo::bot::*;
    pub use duckduckgeo::*;
    pub use axgeom;
    pub use axgeom::*;
    pub use duckduckgeo::F32n;

    pub use dists;

    pub use dists::uniform_rand::UniformRandGen;
    pub use duckduckgeo::array2_inner_into;
    pub use dinotree_alg::prelude::*;
    pub use dinotree_alg::analyze;
    pub use dinotree_alg::dinotree_owned::*;
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


use piston_window::*;



pub fn draw_circle_f32(col: [f32; 4], point:Vec2<f32>, radius:f32, c: &Context, g: &mut G2d) {
    //let ((x1, x2), (y1, y2)) = r1.get();
    {
        //let ((x1, x2), (y1, y2)) = ((x1 as f64, x2 as f64), (y1 as f64, y2 as f64));

        //let square = [x1, y1, x2 - x1, y2 - y1];
        let v=point.inner_as::<f64>();
        let radius=radius as f64;
        use piston_window::ellipse::circle;
        let current = Ellipse::new_border(col,3.0);
           
        current.draw(circle(v.x,v.y,radius),&c.draw_state, c.transform, g);
    }
}

pub fn draw_rect_f32(col: [f32; 4], r1: &Rect<f32>, c: &Context, g: &mut G2d) {
    let ((x1, x2), (y1, y2)) = r1.get();
    {
        let ((x1, x2), (y1, y2)) = ((x1 as f64, x2 as f64), (y1 as f64, y2 as f64));

        let square = [x1, y1, x2 - x1, y2 - y1];
        rectangle(col, square, c.transform, g);
    }
}
pub fn draw_rect_i32(col: [f32; 4], r1: &Rect<i32>, c: &Context, g: &mut G2d) {
    let ((x1, x2), (y1, y2)) = r1.get();
    {
        let ((x1, x2), (y1, y2)) = ((x1 as f64, x2 as f64), (y1 as f64, y2 as f64));

        let square = [x1, y1, x2 - x1, y2 - y1];
        rectangle(col, square, c.transform, g);
    }
}
