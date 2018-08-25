extern crate compt;
extern crate piston_window;
extern crate axgeom;
extern crate rand;
extern crate dinotree;
extern crate ordered_float;
extern crate dinotree_inner;
extern crate rayon;
extern crate dinotree_geom;
extern crate csv;
extern crate serde;



pub(crate) mod data_theory;
pub(crate) mod data_bench;


use std::env;

fn main() {
	let args: Vec<String> = env::args().collect();
   
    match args[1].trim(){
        "data"=>{
            match args[2].trim(){
                "bench-colfind"=>{
                    let area=[area[0] as f64,area[1] as f64];
            
                    let k:Box<DemoSys>=Box::new(data_bench::bench_colfind::DataColFind::new(area));
                    k
                },
                "theory-colfind"=>{
                    let area=[area[0] as f64,area[1] as f64];
            
                    let k:Box<DemoSys>=Box::new(data_theory::theory_colfind::DataColFind::new(area));
                    k
                },
                "theory-colfind-3d"=>{
                    let area=[area[0] as f64,area[1] as f64];
            
                    let k:Box<DemoSys>=Box::new(data_theory::theory_colfind_3d::DataColFind3d::new(area));
                    k
                },
                "theory-sweep-3d"=>{
                    let area=[area[0] as f64,area[1] as f64];
            
                    let k:Box<DemoSys>=Box::new(data_theory::theory_sweep_3d::DataColFind3d::new(area));
                    k
                },
                _=>{
                    panic!("unknown arg");
                }
            }
        },
        _=>{
            panic!("unknown arg");
        }
    }
        
    println!("Hello, world!");
}
