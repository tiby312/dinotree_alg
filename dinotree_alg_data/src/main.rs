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
use std::env;

pub struct FigureBuilder{
    folder:String,
    last_file_name:Option<String>
}

impl FigureBuilder{
    fn new(folder:String)->FigureBuilder{
        FigureBuilder{folder,last_file_name:None}
    }
    fn build(&mut self,filename:&str)->Figure{
        let mut fg = Figure::new();
        let ss=format!("{}/{}.gplot",&self.folder,filename);
        //println!("Creating {}",ss);
        
        self.last_file_name=Some(ss);
        //fg.set_terminal("pngcairo",&ss);// size 1024, 800
        fg
    }
    fn finish(&mut self,mut figure:Figure){
        figure.echo_to_file(&self.last_file_name.take().unwrap());
        //figure.show();
    }
}

fn main() {

    //TODO
    //seperate into benches versus theory runs
    //run benches on laptop/new gaming laptop/android phone/web assembly, and compare differences.
    // 


    let args:Vec<String> = env::args().collect();
    //assert_eq!(args.len(),2,"First arguent needs to be gen or graph");

    match args[1].as_ref(){
        "gen"=>{
            let folder=args[2].clone();

            //std::fs::create_dir_all("target/gen");
            std::fs::create_dir_all(&folder);

            let mut fb=FigureBuilder::new(folder);
            
            /*
            colfind::copy_vs_nocopy::handle(&mut fb);
            
            colfind::construction_vs_query::handle(&mut fb);
            
            
            colfind::level_analysis::handle(&mut fb);
            
            colfind::theory_colfind::handle(&mut fb);
            
            
            colfind::rebal_strat::handle(&mut fb);

            colfind::parallel_heur_comparison::handle(&mut fb);
            */
            spiral::handle(&mut fb);
            /*
            colfind::float_vs_integer::handle(&mut fb);
            
            colfind::theory_colfind::handle(&mut fb);
            
            colfind::theory_colfind_3d::handle(&mut fb);
            */
            //colfind::height_heur_comparison::handle(&mut fb);
            
            //nbody::theory::handle(&mut fb);

        },
        "graph"=>{
            //gnuplot -p "colfind_rebal_vs_query_num_bots_grow_of_1.gplot"
            //std::fs::create_dir_all("target/graphs");

        },
        _=>{
            println!("First argument must be gen or graph");
        }
    }

    

}
