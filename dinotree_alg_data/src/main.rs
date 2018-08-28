extern crate compt;
extern crate piston_window;
extern crate axgeom;
extern crate rand;
extern crate dinotree_alg;
extern crate ordered_float;
extern crate dinotree_inner;
extern crate rayon;
extern crate dinotree_geom;
extern crate csv;
extern crate serde;

#[macro_use]
extern crate serde_derive;


mod support;
mod spiral;

mod data_theory;
mod data_bench;


use piston_window::*;
pub trait DemoSys{
    fn step(&mut self,cursor:[f64;2],c:&piston_window::Context,g:&mut piston_window::G2d)->bool;
}

use std::env;

fn main() {
	let args: Vec<String> = env::args().collect();
   
    let area=[1024u32,768];


    let mut window: PistonWindow = WindowSettings::new("dinotree test",area)
        .exit_on_esc(true)
        .build()
        .unwrap();

    if args.len()!=2{
        println!("valid arguments are \"bench-colfind\" \"theory-colfind\" \"theory-colfind-3d\" \"theory-sweep-3d\" ");
        return;
    }

    let mut curr=match args[1].trim(){
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
    };
        
    let mut cursor=[0.0,0.0];

        
    while let Some(e) = window.next() {
        e.mouse_cursor(|x, y| {
            cursor = [x, y];
        });
        if let Some(Button::Keyboard(key)) = e.press_args() {
            if key == Key::N {
            }
            if key == Key::C{
            
            }
        };

        let mut val=false;
        window.draw_2d(&e, |c, mut g| {
            clear([1.0; 4], g);
            val=curr.step(cursor,&c,&mut g);
        });
        if val{
            break;
        }
    }
}
