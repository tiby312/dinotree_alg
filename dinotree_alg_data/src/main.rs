extern crate compt;
extern crate axgeom;
//extern crate rand;
extern crate dinotree_alg;
extern crate ordered_float;
extern crate dinotree;
extern crate rayon;
extern crate duckduckgeo;
extern crate dists;
extern crate gnuplot;

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

}
#[macro_use]
mod support;
mod colfind;
mod nbody;
mod spiral;
pub(crate) mod datanum;


use std::env;


use gnuplot::*;
pub struct FigureBuilder{
}

impl FigureBuilder{
    fn new(&self,filename:&str)->Figure{
        let mut fg=Figure::new();
        let ss=format!("target/graphs/{}.png",filename);
        println!("Creating {}",ss);
        fg.set_terminal("pngcairo size 800, 600",&ss);
        fg
    }
}

fn main() {
    use std::fs;
    fs::create_dir_all("target/graphs").unwrap();
    
    let fb=FigureBuilder{};
    
    
	let args: Vec<String> = env::args().collect();   
    let _area=[1024u32,768];

    if args.len()!=2{
        println!("valid arguments are \"bench-colfind\" \"theory-colfind\" \"theory-colfind-3d\" \"theory-sweep-3d\" ");
        return;
    }

    let _curr=match args[1].trim(){
        "colfind"=>{
            colfind::level_analysis::handle(&fb);
            colfind::theory_colfind::handle(&fb);
        },
        "colfind-3d"=>{
            colfind::theory_colfind_3d::handle(&fb);
            
        },
        "colfind-tree-height"=>{
            colfind::height_heur_comparison::handle(&fb);
            colfind::parallel_heur_comparison::handle(&fb);
            
        },
        "colfind-construction"=>{
            colfind::construction_vs_query::handle(&fb);
        }
        "colfind-float-integer"=>{
            colfind::float_vs_integer::handle(&fb);
        }
        "nbody"=>{
            nbody::theory::handle(&fb);
        }
        "spiral"=>{
            spiral::handle(&fb);
        }
        "all"=>{
            colfind::parallel_heur_comparison::handle(&fb);
            colfind::level_analysis::handle(&fb);
            spiral::handle(&fb);
            colfind::float_vs_integer::handle(&fb);
            colfind::theory_colfind::handle(&fb);
            colfind::theory_colfind_3d::handle(&fb);
            colfind::height_heur_comparison::handle(&fb);
            colfind::construction_vs_query::handle(&fb);
            nbody::theory::handle(&fb);
        }
        _=>{
            panic!("unknown arg");
        }
    };
}
