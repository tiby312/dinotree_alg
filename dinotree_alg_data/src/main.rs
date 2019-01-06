//#![feature(trusted_len)]
//#![feature(test)]
extern crate compt;
extern crate axgeom;
extern crate dinotree_alg;
extern crate ordered_float;
extern crate dinotree;
extern crate rayon;
extern crate duckduckgeo;
extern crate dists;
extern crate gnuplot;
//extern crate test;

pub fn black_box<T>(dummy: T) -> T {
    unsafe {
        let ret = std::ptr::read_volatile(&dummy);
        std::mem::forget(dummy);
        ret
    }
}

mod inner_prelude{
    pub(crate) use crate::FigureBuilder;
    pub use crate::support::*;
    pub use dinotree_alg::colfind;
    pub use dinotree::*;
    pub use dinotree::advanced::*;
    pub(crate) use axgeom;
    pub(crate) use crate::datanum;
    pub use gnuplot::*;
    pub(crate) use dists;
    pub use std::time::Instant;
    pub use std::time::Duration;
    pub use crate::black_box;
    pub use num_traits::cast::AsPrimitive;
}
#[macro_use]
mod support;
mod colfind;
mod spiral;
pub(crate) mod datanum;


use gnuplot::*;
pub struct FigureBuilder{}

impl FigureBuilder{
    fn build(&mut self,_filename:&str)->Figure{
        let mut fg = Figure::new();
        let ss=format!("target/graphs/{}.png",_filename);
        println!("Creating {}",ss);
        //fg.set_terminal("pngcairo",&ss);// size 1024, 800
        fg
    }
    fn finish(&mut self,mut figure:Figure){
        figure.show();
    }
}

fn main() {
    
    let mut fb=FigureBuilder{};
    
    colfind::construction_vs_query::handle(&mut fb);
    
    colfind::level_analysis::handle(&mut fb);
    
    colfind::theory_colfind::handle(&mut fb);
    
    colfind::copy_vs_nocopy::handle(&mut fb);
    
    colfind::rebal_strat::handle(&mut fb);

    colfind::parallel_heur_comparison::handle(&mut fb);
    
    spiral::handle(&mut fb);
    
    colfind::float_vs_integer::handle(&mut fb);
    
    colfind::theory_colfind::handle(&mut fb);
    
    colfind::theory_colfind_3d::handle(&mut fb);
    
    colfind::height_heur_comparison::handle(&mut fb);
    //nbody::theory::handle(&mut fb);
    

}
