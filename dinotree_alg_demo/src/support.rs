use axgeom::*;
use std;


pub mod prelude{
    pub use crate::DemoSys;
    pub(crate) use piston_window;
    pub use ordered_float::NotNan;
    pub use piston_window::*;
    pub use dinotree::*;
    pub use dinotree::nocopy::*;
    pub use dinotree::copy::*;
    pub(crate) use axgeom;
    pub(crate) use crate::support::*;
    pub use dinotree::BBox;
    
    pub use duckduckgeo::*;
    pub use duckduckgeo::bot::*;

    pub use cgmath::prelude::*;
    pub use cgmath::Vector2;
    pub use cgmath::vec2;
    pub use duckduckgeo::F64n;
}


use duckduckgeo::F64n;



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





use piston_window::*;



pub fn draw_rect_f64n(col:[f32;4],r1:&Rect<F64n>,c:&Context,g:&mut G2d){
    let ((x1,x2),(y1,y2))=r1.get();        
    {
        let ((x1,x2),(y1,y2))=((x1.into_inner(),x2.into_inner()),(y1.into_inner(),y2.into_inner()));
           
        let square = [x1,y1,x2-x1,y2-y1];
        rectangle(col, square, c.transform, g);
    }
}
pub fn draw_rect_isize(col:[f32;4],r1:&Rect<isize>,c:&Context,g:&mut G2d){
    let ((x1,x2),(y1,y2))=r1.get();        
    {
        let ((x1,x2),(y1,y2))=((x1 as f64,x2 as f64),(y1 as f64,y2 as f64));
           
        let square = [x1,y1,x2-x1,y2-y1];
        rectangle(col, square, c.transform, g);
    }
}

