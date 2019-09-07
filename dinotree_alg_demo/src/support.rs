use std;
use dinotree::axgeom::*;

pub mod prelude{

    pub use crate::DemoSys;
    pub(crate) use piston_window;
    pub use ordered_float::NotNan;
    pub use piston_window::*;
    /*
    pub use dinotree::*;
    pub use dinotree::nocopy::*;
    pub use dinotree::copy::*;
    */
    pub(crate) use crate::support::*;
    //pub use dinotree::BBox;
    
    pub use duckduckgeo::*;
    pub use duckduckgeo::bot::*;

    pub use duckduckgeo::F32n;

    pub use dists;

    pub use duckduckgeo::array2_inner_into;
    pub use dinotree::axgeom::*;
    pub use dinotree::axgeom;
    pub use dinotree::compt;
    pub use dists::uniform_rand::UniformRandGen;
    //pub use core::pin::Pin;
    //pub use dinotree::*;
    pub use dinotree::prelude::*;
}





pub struct ColorGenerator{
    rgb:[u8;3]
}
impl ColorGenerator{
    pub fn new()->ColorGenerator{
        ColorGenerator{rgb:[50,100,200]}
    }
}

impl std::iter::FusedIterator for ColorGenerator{}
impl Iterator for ColorGenerator{
    type Item=[u8;3];
    fn next(&mut self)->Option<Self::Item>{
        self.rgb[0]=((self.rgb[0] as usize + 1) % 256) as u8;
        self.rgb[1]=((self.rgb[1] as usize + 1) % 256) as u8;
        self.rgb[2]=((self.rgb[2] as usize + 1) % 256) as u8;
        Some(self.rgb)
    }
}

use duckduckgeo::F32n;
use ordered_float::NotNan;
pub fn f32n(a:f32)->F32n{
    NotNan::new(a).unwrap()
}





use piston_window::*;



pub fn draw_rect_f32(col:[f32;4],r1:&Rect<f32>,c:&Context,g:&mut G2d){
    let ((x1,x2),(y1,y2))=r1.get();        
    {
        let ((x1,x2),(y1,y2))=((x1 as f64,x2 as f64),(y1 as f64,y2 as f64));
           
        let square = [x1,y1,x2-x1,y2-y1];
        rectangle(col, square, c.transform, g);
    }
}
pub fn draw_rect_i32(col:[f32;4],r1:&Rect<i32>,c:&Context,g:&mut G2d){
    let ((x1,x2),(y1,y2))=r1.get();        
    {
        let ((x1,x2),(y1,y2))=((x1 as f64,x2 as f64),(y1 as f64,y2 as f64));
           
        let square = [x1,y1,x2-x1,y2-y1];
        rectangle(col, square, c.transform, g);
    }
}

