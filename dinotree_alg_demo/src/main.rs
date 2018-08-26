extern crate compt;
extern crate piston_window;
extern crate axgeom;
extern crate rand;
extern crate dinotree_alg;
extern crate ordered_float;
extern crate dinotree_inner;
extern crate rayon;
extern crate dinotree_geom;


use piston_window::*;
#[macro_use]
pub(crate) mod support;
pub(crate) mod demos;


pub trait DemoSys{
    fn step(&mut self,cursor:[f64;2],c:&piston_window::Context,g:&mut piston_window::G2d,check_naive:bool);
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


            if self.0==8{
                self.0=0
            }
            match curr{

                0=>{Box::new(demo_knearest::KnearestDemo::new(area))},
                1=>{Box::new(demo_multirect::MultiRectDemo::new(area))},
                2=>{Box::new(demo_for_every_nearest::KnearestEveryDemo::new(area))}
                3=>{Box::new(demo_raycast_isize::RaycastDemo::new(area))},
                4=>{Box::new(demo_raycast_f64::RaycastF64Demo::new(area))},
                5=>{Box::new(demo_nbody::DemoNbody::new(area))},
                6=>{Box::new(demo_original_order::OrigOrderDemo::new(area))}
                7=>{Box::new(demo_intersect_with::IntersectWithDemo::new(area))}
                _=>{panic!("Not possible")}
            }
        }
    }
}

fn main(){
    
    let area=[1024u32,768];

    let mut window: PistonWindow = WindowSettings::new("dinotree test",area)
        .exit_on_esc(true)
        .build()
        .unwrap();


    
    let mut demo_iter=demo_iter::DemoIter::new();
    
    let mut curr=demo_iter.next(area);

    
    println!("Press \"N\" to go to the next example");
    println!("Press \"C\" to turn off verification against naive algorithms.");
    let mut cursor=[0.0,0.0];

    let mut check_naive=false;
    while let Some(e) = window.next() {
        e.mouse_cursor(|x, y| {
            cursor = [x, y];
        });
        if let Some(Button::Keyboard(key)) = e.press_args() {
            if key == Key::N {
                curr=demo_iter.next(area);
            }
            if key == Key::C{
                check_naive=!check_naive;
                if check_naive{
                    println!("Naive checking is on. Some demo's will now check the tree algorithm against a naive non tree version");
                }else{
                    println!("Naive checking is off.");
                }
            }
        };

        window.draw_2d(&e, |c, mut g| {
            clear([1.0; 4], g);
            curr.step(cursor,&c,&mut g,check_naive);
        });
    }
    
}