
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
#[macro_use]
extern crate serde_derive;

use piston_window::*;
#[macro_use]
pub(crate) mod support;
pub(crate) mod demos;
//pub(crate) mod data_theory;
//pub(crate) mod data_bench;

pub trait DemoSys{
    fn step(&mut self,cursor:[f64;2],c:&piston_window::Context,g:&mut piston_window::G2d);
}

mod demo_iter{
    use super::*;
    use demos::*;
    pub struct DemoIter(usize);

    impl DemoIter{
        pub fn new()->DemoIter{
            DemoIter(0)
        }
        pub fn next(&mut self,area:[u32;2])->Box<DemoSys>{
            let area=[area[0] as f64,area[1] as f64];
            let curr=self.0;
            self.0+=1;


            if self.0==9{
                self.0=0
            }
            match curr{

                0=>{Box::new(demo_test_raycast::TestRaycastDemo::new(area))},
                1=>{Box::new(demo_knearest::KnearestDemo::new(area))},
                2=>{Box::new(demo_multirect::MultiRectDemo::new(area))},
                3=>{Box::new(demo_for_every_nearest::KnearestEveryDemo::new(area))}
                4=>{Box::new(demo_raycast_isize::RaycastDemo::new(area))},
                5=>{Box::new(demo_raycast_f64::RaycastF64Demo::new(area))},
                6=>{Box::new(demo_nbody::DemoNbody::new(area))},
                7=>{Box::new(demo_original_order::OrigOrderDemo::new(area))}
                8=>{Box::new(demo_intersect_with::IntersectWithDemo::new(area))}
                _=>{panic!("Not possible")}
            }
        }
    }
}

use std::env;

fn main(){
    let args: Vec<String> = env::args().collect();
    
    let area=[1024u32,768];

    let mut window: PistonWindow = WindowSettings::new("dinotree test",area)
        .exit_on_esc(true)
        .build()
        .unwrap();


    
    let mut demo_iter=demo_iter::DemoIter::new();
    
    let mut curr=if args.len()==1{
            demo_iter.next(area)
    }else{
        /*
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
        */
        panic!("Not implemented yet");
    };

    
    println!("Press \"C\" to go to the next example");
    
    let mut cursor=[0.0,0.0];

    while let Some(e) = window.next() {
        e.mouse_cursor(|x, y| {
            cursor = [x, y];
        });
        if let Some(Button::Keyboard(key)) = e.press_args() {
            if key == Key::C {
                curr=demo_iter.next(area);
            }
            //println!("Pressed keyboard key '{:?}'", key);
        };

        window.draw_2d(&e, |c, mut g| {
            clear([1.0; 4], g);

            curr.step(cursor,&c,&mut g);


        });
    }
    
}