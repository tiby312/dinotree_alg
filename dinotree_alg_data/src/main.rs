extern crate compt;
extern crate axgeom;
extern crate rand;
extern crate dinotree_alg;
extern crate ordered_float;
extern crate dinotree_inner;
extern crate rayon;
extern crate dinotree_geom;
extern crate dists;
extern crate gnuplot;

mod inner_prelude{

    pub use support::*;
    pub use dinotree_alg::colfind;
    pub(crate) use std;
    pub use dinotree_inner::*;
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
pub(crate) mod datanum;


use std::env;

fn main() {

    
	let args: Vec<String> = env::args().collect();   
    let area=[1024u32,768];

    if args.len()!=2{
        println!("valid arguments are \"bench-colfind\" \"theory-colfind\" \"theory-colfind-3d\" \"theory-sweep-3d\" ");
        return;
    }

    let mut curr=match args[1].trim(){
        "colfind"=>{
            colfind::theory_colfind::handle();
        },
        "colfind-3d"=>{
            colfind::theory_colfind_3d::handle();
            
        },
        "colfind-tree-height"=>{
            colfind::height_heur_comparison::handle();
        },
        "colfind-construction"=>{
            colfind::construction_vs_query::handle();
        }
        "nbody"=>{
            nbody::theory::handle();
        }
        "all"=>{
            colfind::theory_colfind::handle();
            colfind::theory_colfind_3d::handle();
            colfind::height_heur_comparison::handle();
            colfind::construction_vs_query::handle();
            nbody::theory::handle();
        }
        _=>{
            panic!("unknown arg");
        }
    };
}
