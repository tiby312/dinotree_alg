#![feature(trusted_len)]
#![feature(test)]
extern crate compt;
extern crate axgeom;
extern crate dinotree_alg;
extern crate ordered_float;
extern crate dinotree;
extern crate rayon;
extern crate duckduckgeo;
extern crate dists;
extern crate gnuplot;
extern crate test;

mod inner_prelude{
    pub(crate) use FigureBuilder;
    pub use support::*;
    pub use dinotree_alg::colfind;
    pub use dinotree::*;
    pub use dinotree::advanced::*;
    pub(crate) use axgeom;
    pub(crate) use datanum;
    pub use gnuplot::*;
    pub(crate) use dists;
    pub use std::time::Instant;
    pub use std::time::Duration;
    pub(crate) use ordered_float::NotNan;
    pub(crate) use test::*;

}
#[macro_use]
mod support;
mod colfind;
//mod nbody;
mod spiral;
pub(crate) mod datanum;


use std::env;


use gnuplot::*;
pub struct FigureBuilder{
}

impl FigureBuilder{
    fn new(&mut self,filename:&str)->Figure{
        let mut fg=Figure::new();
        let ss=format!("target/graphs/{}.png",filename);
        //println!("Creating {}",ss);
        //fg.set_terminal("pngcairo size 800, 600",&ss);
        fg
    }
    fn finish(&mut self,mut figure:Figure){
        //figure.echo(&mut std::io::stdout());
        figure.show();
    }
}

fn main() {
    use std::fs;
    

    let mut fb=FigureBuilder{};
    
    colfind::theory_colfind::handle(&mut fb);
    
    colfind::copy_vs_nocopy::handle(&mut fb);
    
    colfind::rebal_strat::handle(&mut fb);
    /*

    
    
    colfind::parallel_heur_comparison::handle(&mut fb);
    colfind::level_analysis::handle(&mut fb);
    spiral::handle(&mut fb);
    colfind::float_vs_integer::handle(&mut fb);
    colfind::theory_colfind::handle(&mut fb);
    colfind::theory_colfind_3d::handle(&mut fb);
    colfind::height_heur_comparison::handle(&mut fb);
    colfind::construction_vs_query::handle(&mut fb);
    //nbody::theory::handle(&mut fb);
    */

}
