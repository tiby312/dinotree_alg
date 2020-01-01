use axgeom::vec2;
use axgeom::Vec2;
use axgeom::*;
use glutin::event::ElementState;
use glutin::event::Event;
use glutin::event::VirtualKeyCode;
use glutin::event::WindowEvent;
use glutin::event_loop::ControlFlow;
use very_simple_2d::glutin;

#[macro_use]
pub(crate) mod support;
pub(crate) mod demos;
use duckduckgeo::F32n;

use self::support::prelude::*;

pub struct Demo(Box<dyn FnMut(Vec2<F32n>, &mut SimpleCanvas, bool)>);
impl Demo {
    pub fn new(func: impl FnMut(Vec2<F32n>, &mut SimpleCanvas, bool) + 'static) -> Self {
        Demo(Box::new(func))
    }
    pub fn step(&mut self, point: Vec2<F32n>, sys: &mut SimpleCanvas, check_naive: bool) {
        self.0(point, sys, check_naive);
    }
}

mod demo_iter {
    use crate::demos::*;
    use crate::*;
    pub struct DemoIter(usize);

    impl DemoIter {
        pub fn new() -> DemoIter {
            DemoIter(0)
        }
        pub fn next(&mut self, area: Vec2<u32>,canvas:&mut SimpleCanvas) -> Demo {
            let curr = self.0;

            let area = Rect::new(0.0, area.x as f32, 0.0, area.y as f32);
            let area: Rect<F32n> = area.inner_try_into().unwrap();

            let k: Demo = match curr {
                0 => demo_raycast_f32::make_demo(area,canvas),
                1 => demo_raycast_f32_debug::make_demo(area,canvas),
                2 => demo_liquid::make_demo(area),
                3 => demo_multirect::make_demo(area,canvas),
                4 => demo_original_order::make_demo(area),
                5 => demo_intersect_with::make_demo(area,canvas),
                6 => demo_knearest::make_demo(area,canvas),
                7 => demo_rigid_body::make_demo(area),
                8 => demo_nbody::make_demo(area),
                9 => demo_raycast_grid::make_demo(area,canvas),
                _ => unreachable!("Not possible"),
            };
            self.0 += 1;

            if self.0 == 10 {
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

    let area = vec2(800, 600);

    let events_loop = glutin::event_loop::EventLoop::new();

    let mut sys = very_simple_2d::WindowedSystem::new(area.inner_as(), &events_loop,"dinotree_alg demo");
    //let mut sys=very_simple_2d::FullScreenSystem::new(&events_loop);
    //sys.set_viewport_min(600.);

    let mut demo_iter = demo_iter::DemoIter::new();

    let mut curr = demo_iter.next(area,sys.canvas_mut());

    println!("Press \"N\" to go to the next example");

    let check_naive = false;
    let mut cursor = vec2same(0.);
    let mut timer = very_simple_2d::RefreshTimer::new(16);
    events_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::KeyboardInput { input, .. } => {
                    if input.state == ElementState::Released {
                        match input.virtual_keycode {
                            Some(VirtualKeyCode::Escape) => {
                                *control_flow = ControlFlow::Exit;
                            }
                            Some(VirtualKeyCode::N) => {
                                curr = demo_iter.next(area,sys.canvas_mut());
                            }
                            Some(VirtualKeyCode::C) => {
                                check_naive != check_naive;
                            }
                            _ => {}
                        }
                    }
                }
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                }
                WindowEvent::Resized(_logical_size) => {}
                WindowEvent::CursorMoved {
                    modifiers: _,
                    device_id: _,
                    position: logical_position,
                } => {
                    let dpi=sys.get_hidpi_factor();
                    let glutin::dpi::PhysicalPosition { x, y } = logical_position.to_physical(dpi);
                    cursor = vec2(x as f32, y as f32);
                }
                WindowEvent::MouseInput {
                    modifiers: _,
                    device_id: _,
                    state,
                    button,
                } => {
                    if button == glutin::event::MouseButton::Left {
                        match state {
                            glutin::event::ElementState::Pressed => {
                                //mouse_active=true;
                            }
                            glutin::event::ElementState::Released => {
                                //mouse_active=false;
                            }
                        }
                    }
                }
                _ => {}
            },
            Event::EventsCleared => {
                if timer.is_ready() {
                    let k = sys.canvas_mut();
                    k.clear_color([0.2, 0.2, 0.2]);
                    curr.step(cursor.inner_try_into().unwrap(), k, check_naive);
                    sys.swap_buffers();
                }
            }
            _ => {}
        }
    });
}
