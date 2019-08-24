extern crate piston_window;
extern crate axgeom;
extern crate dinotree_alg;
extern crate ordered_float;
extern crate dinotree;
extern crate duckduckgeo;


use axgeom::Vec2;
use axgeom::vec2;
use piston_window::*;
use axgeom::*;
use axgeom::ordered_float::*;

#[macro_use]
pub(crate) mod support;
pub(crate) mod demos;
use duckduckgeo::F32n;

pub trait DemoSys{
    fn step(&mut self,cursor:Vec2<F32n>,c:&piston_window::Context,g:&mut piston_window::G2d,check_naive:bool);
}

mod demo_iter{
    use crate::*;
    use crate::demos::*;
    pub struct DemoIter(usize);

    impl DemoIter{
        pub fn new()->DemoIter{
            DemoIter(0)
        }
        pub fn next(&mut self,area:Vec2<u32>)->Box<DemoSys>{
            let curr=self.0;
            

            let area=Rect::new(0.0,area.x as f32,0.0,area.y as f32);
            let area:Rect<F32n>=area.inner_try_into().unwrap();

            let k:Box<DemoSys>=match curr{
                0=>{Box::new(demo_nbody::DemoNbody::new(area))},
                0=>{Box::new(demo_raycast_f32_debug::RaycastF32DebugDemo::new(area))}
                1=>{Box::new(demo_raycast_f32::RaycastF32Demo::new(area))}
                2=>{Box::new(demo_liquid::LiquidDemo::new(area))},
                3=>{Box::new(demo_multirect::MultiRectDemo::new(area))},
                4=>{Box::new(demo_original_order::OrigOrderDemo::new(area))},
                5=>{Box::new(demo_intersect_with::IntersectWithDemo::new(area))},
                6=>{Box::new(demo_knearest::KnearestDemo::new(area))},
                7=>{Box::new(demo_rigid_body::RigidBodyDemo::new(area))}
                8=>{Box::new(demo_grid::GridDemo::new(area))}
                
                /*
                
                
                */
                _=>{unreachable!("Not possible")}
            };
            self.0+=1;

            if self.0==8{
                self.0=0
            }
            k
        }
    }
}

fn main(){
    
    let area=vec2(1024,768);

    let window = WindowSettings::new("dinotree test",[area.x,area.y])
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


    let mut cursor:Vec2<F32n>=vec2(0.0,0.0).inner_try_into().unwrap();

    let mut check_naive=false;
    while let Some(e) = window.next() {
        e.mouse_cursor(|[x, y]| {
            //cursor = vec2(x,y).inner_into::<f32>().inner_try_into::<F32n>().unwrap();
            cursor.x=NotNan::new(x as f32).unwrap();
            cursor.y=NotNan::new(y as f32).unwrap();
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

        window.draw_2d(&e, |c, mut g,_| {
            clear([1.0; 4], g);
            c.view();//trans(500.0,500.0);
            curr.step(cursor,&c,&mut g,check_naive);
        });
    }
    
}