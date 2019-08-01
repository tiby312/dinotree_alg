extern crate piston_window;
extern crate axgeom;
extern crate dinotree_alg;
extern crate ordered_float;
extern crate dinotree;
extern crate duckduckgeo;


use cgmath::Vector2;
use cgmath::vec2;
use num_traits::*;
use piston_window::*;
#[macro_use]
pub(crate) mod support;
pub(crate) mod demos;
use duckduckgeo::F64n;

pub trait DemoSys{
    fn step(&mut self,cursor:Vector2<F64n>,c:&piston_window::Context,g:&mut piston_window::G2d,check_naive:bool);
}

mod demo_iter{
    use crate::*;
    use crate::demos::*;
    pub struct DemoIter(usize);

    impl DemoIter{
        pub fn new()->DemoIter{
            DemoIter(0)
        }
        pub fn next(&mut self,area:[u32;2])->Box<DemoSys>{
            let area=vec2(area[0],area[1]).cast().unwrap();
            let curr=self.0;
            


            if self.0==8{
                self.0=0
            }
            let k:Box<DemoSys>=match curr{

                0=>{Box::new(demo_knearest::KnearestDemo::new(area))},
                1=>{Box::new(demo_raycast_f64::RaycastF64Demo::new(area))}
                /*
                1=>{Box::new(demo_multirect::MultiRectDemo::new(area))},
                2=>{Box::new(demo_raycast_isize::RaycastDemo::new(area))},
                
                4=>{Box::new(demo_nbody::DemoNbody::new(area))},
                5=>{Box::new(demo_original_order::OrigOrderDemo::new(area))},
                6=>{Box::new(demo_intersect_with::IntersectWithDemo::new(area))},
                7=>{Box::new(demo_rigid_body::RigidBodyDemo::new(area))}
                */
                _=>{unreachable!("Not possible")}
            };
            self.0+=1;
            k
        }
    }
}

fn main(){
    
    let area=[1024u32,768];

    let window = WindowSettings::new("dinotree test",area)
        .exit_on_esc(true)
        .fullscreen(false)
        .resizable(false);
        

    println!("window size={:?}",window.get_size());
    

    let mut window:PistonWindow=window.build().unwrap();


    let mut demo_iter=demo_iter::DemoIter::new();
    
    let mut curr=demo_iter.next(area);

    
    println!("Press \"N\" to go to the next example");
    println!("Press \"C\" to turn off verification against naive algorithms.");
    println!("Performance suffers from not batching draw calls (piston's built in rectangle drawing primitives are used instead of vertex buffers). These demos are not meant to showcase the performance of the algorithms. See the dinotree_alg_data project for benches.");


    let mut cursor:Vector2<F64n>=Vector2::zero();

    let mut check_naive=false;
    while let Some(e) = window.next() {
        e.mouse_cursor(|x, y| {
            cursor = vec2(x,y).cast().unwrap();
        });
        if let Some(Button::Keyboard(key)) = e.press_args() {
            if key == Key::N {
                curr=demo_iter.next(area);
            }
            if key == Key::C{
                check_naive= !check_naive;
                if check_naive{
                    println!("Naive checking is on. Some demo's will now check the tree algorithm against a naive non tree version");
                }else{
                    println!("Naive checking is off.");
                }
            }
        };

        window.draw_2d(&e, |c, mut g| {
            clear([1.0; 4], g);
            c.view();//trans(500.0,500.0);
            curr.step(cursor,&c,&mut g,check_naive);
        });
    }
    
}