

use very_simple_2d::glutin;
use glutin::event::VirtualKeyCode;
use glutin::event::Event;
use glutin::event::WindowEvent;
use glutin::event_loop::ControlFlow;
use glutin::event::ElementState;
use axgeom::vec2;
use axgeom::Vec2;
use axgeom::*;

#[macro_use]
pub(crate) mod support;
pub(crate) mod demos;
use duckduckgeo::F32n;


pub trait DemoSys {
    fn step(
        &mut self,
        cursor: Vec2<F32n>,
        sys: very_simple_2d::DrawSession,
        check_naive: bool,
    );
}

mod demo_iter {
    use crate::demos::*;
    use crate::*;
    pub struct DemoIter(usize);

    impl DemoIter {
        pub fn new() -> DemoIter {
            DemoIter(0)
        }
        pub fn next(&mut self, area: Vec2<u32>) -> Box<dyn DemoSys> {
            let curr = self.0;

            let area = Rect::new(0.0, area.x as f32, 0.0, area.y as f32);
            let area: Rect<F32n> = area.inner_try_into().unwrap();

            let k: Box<dyn DemoSys> = match curr {
                0 => Box::new(demo_raycast_f32::RaycastF32Demo::new(area)),
                1 => Box::new(demo_raycast_f32_debug::RaycastF32DebugDemo::new(area)),
                2 => Box::new(demo_liquid::LiquidDemo::new(area)),
                3 => Box::new(demo_multirect::MultiRectDemo::new(area)),
                4 => Box::new(demo_original_order::OrigOrderDemo::new(area)),
                5 => Box::new(demo_intersect_with::IntersectWithDemo::new(area)),
                6 => Box::new(demo_knearest::KnearestDemo::new(area)),
                7 => Box::new(demo_rigid_body::RigidBodyDemo::new(area)),
                8 => Box::new(demo_grid::GridDemo::new(area)),
                9 => Box::new(demo_nbody::DemoNbody::new(area)),
                10 => Box::new(demo_raycast_grid::RaycastGridDemo::new(area)),
                _ => unreachable!("Not possible"),
            };
            self.0 += 1;

            if self.0 == 11 {
                self.0 = 0
            }
            k
        }
    }
}

fn main() {
    rayon::ThreadPoolBuilder::new()
        .num_threads(num_cpus::get_physical())
        .build_global()
        .unwrap();

    let area = vec2(1024, 768);

    let events_loop = glutin::event_loop::EventLoop::new();

    let mut sys = very_simple_2d::System::new(rect(0.,1024.,0.,768.),&events_loop);

    let mut demo_iter = demo_iter::DemoIter::new();

    let mut curr = demo_iter.next(area);

    println!("Press \"N\" to go to the next example");
    //println!("Press \"C\" to turn off verification against naive algorithms.");
    println!("Performance suffers from not batching draw calls (piston's built in rectangle drawing primitives are used instead of vertex buffers). These demos are not meant to showcase the performance of the algorithms. See the dinotree_alg_data project for benches.");


    let mut check_naive = false;
    let mut cursor=vec2same(0.);
    let mut timer=very_simple_2d::RefreshTimer::new(16);
    events_loop.run(move |event,_,control_flow| {
        match event {
            Event::WindowEvent{ event, .. } => match event {
                WindowEvent::KeyboardInput{input,..}=>{
                    if input.state==ElementState::Released{    
                        match input.virtual_keycode{
                            Some(VirtualKeyCode::Escape)=>{
                                *control_flow=ControlFlow::Exit;
                            },
                            Some(VirtualKeyCode::N)=>{
                                curr=demo_iter.next(area);
                            },
                            Some(VirtualKeyCode::C)=>{
                                check_naive!=check_naive;
                            }
                            _=>{}
                        }
                    }
                },
                WindowEvent::CloseRequested => {*control_flow=ControlFlow::Exit;},
                WindowEvent::Resized(_logical_size) => {
                    
                },
                WindowEvent::CursorMoved{modifiers:_,device_id:_,position:logical_position} => {
                    let glutin::dpi::LogicalPosition{x,y}=logical_position;
                    cursor=vec2(x as f32,y as f32);
                },
                WindowEvent::MouseInput{modifiers:_,device_id:_,state,button}=>{
                    if button==glutin::event::MouseButton::Left{
                        match state{
                            glutin::event::ElementState::Pressed=>{  
                                //mouse_active=true;  
                                
                            }
                            glutin::event::ElementState::Released=>{
                                //mouse_active=false;
                            }
                        }
                    }
                },
                _=>{}
            },
            Event::EventsCleared=>{
                if timer.is_ready(){
                    curr.step(cursor.inner_try_into().unwrap(),sys.get_sys(),check_naive);
                    sys.swap_buffers();
                }
            },
            _ => {},
        }    
    });

}
