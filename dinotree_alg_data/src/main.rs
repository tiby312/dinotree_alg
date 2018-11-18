extern crate compt;
extern crate axgeom;
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
    fn new(&mut self,filename:&str)->Figure{
        let mut fg=Figure::new();
        let ss=format!("target/graphs/{}.png",filename);
        //println!("Creating {}",ss);
        //fg.set_terminal("pngcairo size 800, 600",&ss);
        fg
    }
    fn finish(&mut self,mut figure:Figure){
        figure.echo(&mut std::io::stdout());
        //figure.show();
    }
}

fn main() {
    use std::fs;
    //fs::create_dir_all("target/graphs").unwrap();
    
    let mut fb=FigureBuilder{};
    
    
	let args: Vec<String> = env::args().collect();   
    let _area=[1024u32,768];

    if args.len()!=2{
        println!("valid arguments are \"bench-colfind\" \"theory-colfind\" \"theory-colfind-3d\" \"theory-sweep-3d\" ");
        return;
    }

    let _curr=match args[1].trim(){
        "colfind"=>{
            //colfind::level_analysis::handle(&fb);
            colfind::theory_colfind::handle(&mut fb);
        },
        "colfind-3d"=>{
            colfind::theory_colfind_3d::handle(&mut fb);
            
        },
        "colfind-tree-height"=>{
            colfind::height_heur_comparison::handle(&mut fb);
            colfind::parallel_heur_comparison::handle(&mut fb);
            
        },
        "colfind-construction"=>{
            colfind::construction_vs_query::handle(&mut fb);
        }
        "colfind-float-integer"=>{
            colfind::float_vs_integer::handle(&mut fb);
        }
        "nbody"=>{
            nbody::theory::handle(&mut fb);
        }
        "spiral"=>{
            spiral::handle(&mut fb);
        }
        "all"=>{
            colfind::parallel_heur_comparison::handle(&mut fb);
            colfind::level_analysis::handle(&mut fb);
            spiral::handle(&mut fb);
            colfind::float_vs_integer::handle(&mut fb);
            colfind::theory_colfind::handle(&mut fb);
            colfind::theory_colfind_3d::handle(&mut fb);
            colfind::height_heur_comparison::handle(&mut fb);
            colfind::construction_vs_query::handle(&mut fb);
            nbody::theory::handle(&mut fb);
        }
        _=>{
            panic!("unknown arg");
        }
    };

    println!();
}
