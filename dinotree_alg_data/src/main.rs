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

mod support;
mod data_theory;


use std::env;

fn main() {
	let args: Vec<String> = env::args().collect();   
    let area=[1024u32,768];

    if args.len()!=2{
        println!("valid arguments are \"bench-colfind\" \"theory-colfind\" \"theory-colfind-3d\" \"theory-sweep-3d\" ");
        return;
    }

    let mut curr=match args[1].trim(){
        "theory-colfind"=>{
            data_theory::theory_colfind::handle();
        },
        "theory-colfind-3d"=>{
            data_theory::theory_colfind_3d::handle();
            
        },
        "theory-tree-height"=>{
            data_theory::height_heur_comparison::handle();
        },
        "theory-construction"=>{
            data_theory::construction_vs_query::handle();
        }
        "all"=>{
            data_theory::theory_colfind::handle();
            data_theory::theory_colfind_3d::handle();
            data_theory::height_heur_comparison::handle();
            data_theory::construction_vs_query::handle();
        }
        _=>{
            panic!("unknown arg");
        }
    };
}
