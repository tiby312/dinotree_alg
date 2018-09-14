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
    pub(crate) use FigureBuilder;
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


use gnuplot::*;
pub struct FigureBuilder{
}
impl FigureBuilder{
    fn new(&self,filename:&str)->Figure{
        let mut fg=Figure::new();
        let ss=format!("graphs/{}.png",filename);
        fg.set_terminal("pngcairo size 1024, 768",&ss);
        fg
    }
}
fn main() {


    let fb=FigureBuilder{};
    colfind::float_vs_integer::handle(&fb);
    return;

	let args: Vec<String> = env::args().collect();   
    let area=[1024u32,768];

    if args.len()!=2{
        println!("valid arguments are \"bench-colfind\" \"theory-colfind\" \"theory-colfind-3d\" \"theory-sweep-3d\" ");
        return;
    }

    let mut curr=match args[1].trim(){
        "colfind"=>{
            colfind::theory_colfind::handle(&fb);
        },
        "colfind-3d"=>{
            colfind::theory_colfind_3d::handle(&fb);
            
        },
        "colfind-tree-height"=>{
            colfind::height_heur_comparison::handle(&fb);
        },
        "colfind-construction"=>{
            colfind::construction_vs_query::handle(&fb);
        }
        "nbody"=>{
            nbody::theory::handle(&fb);
        }

        "all"=>{
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
