use crate::support::prelude::*;

use duckduckgeo;

/*
    sequential impulse solver
    http://myselph.de/gamePhysics/equalityConstraints.html
 Constraint Solver

    As mentioned during the overvie, modern physic engines seem to use mostly iterative solvers that work as follows (pseudocode): 

for i = 1 to nIterations
    for c in constraints
        P = computeImpulses(c);
        c.bodies.applyImpulses(P);

*/

#[derive(Copy, Clone)]
pub struct Bot {
    pos: Vec2<f32>,
    vel: Vec2<f32>, 
}

use std::time::{Duration, Instant};
use std::thread::sleep;

pub fn make_demo(dim: Rect<F32n>) -> Demo {
    let num_bot = 40000;

    let radius = 2.0;

    let mut bots: Vec<_> = UniformRandGen::new(dim.inner_into())
        .take(num_bot)
        .map(|pos| Bot {
            pos,
            vel: vec2same(0.0)
        })
        .collect();

    Demo::new(move |cursor, canvas, _check_naive| {

        let now = Instant::now();

        let mut k:Vec<BBoxMut<NotNan<f32>,_>> = bbox_helper::create_bbox_mut(&mut bots, |b| {
            Rect::from_point(b.pos, vec2same(radius))
                .inner_try_into()
                .unwrap()
        });

        let mut tree = DinoTree::new_par(&mut k);
    
        let a1=now.elapsed().as_millis();

        {
            let dim2 = dim.inner_into();
            tree.for_all_not_in_rect_mut(&dim, |mut a| {
                let a=a.inner_mut();
                duckduckgeo::collide_with_border(&mut a.pos,&mut a.vel, &dim2, 0.2);
            });
        }

        let vv = vec2same(200.0).inner_try_into().unwrap();
        let cc = cursor.inner_into();
        tree.for_all_in_rect_mut(&axgeom::Rect::from_point(cursor, vv), |mut b| {
            let b=b.inner_mut();
            
            let offset=b.pos-cursor.inner_into();
            if offset.magnitude()<200.0*0.5{
                let _ = duckduckgeo::repel_one(b.pos,&mut b.vel, cc, 0.001, 4.0);
            }
        });

        let a2=now.elapsed().as_millis();


        let num_iterations=8;
        let num_iterations_inv=1.0/num_iterations as f32;
        
        let mut collision_lists=tree.find_collisions_mut_par_ext(
            |_|vec!(Vec::new()),
            |a,mut b|a.append(&mut b),
            |arr,mut a,mut b|{
                if let Some(k)=Collision::new(radius,num_iterations_inv,a.inner_mut(),b.inner_mut()){
                    arr[0].push(k)   
                }
            },
            vec!(Vec::new())
        );
        /*
        print!("collision_lists: ");
        for a in collision_lists.iter(){
            print!("{} ",a.len());
        }
        println!();
        */
        //println!("collision size={}",collisions.len());
        /*
        let mut collisions=Vec::new();
        tree.find_collisions_mut(|mut a,mut b|{
            if let Some(k)=Collision::new(radius,num_iterations_inv,a.inner_mut(),b.inner_mut()){
                collisions.push(k)   
            }
        });
        */

        let a3=now.elapsed().as_millis();

                    
        let mag=0.03*num_iterations_inv - 0.01;
        for _ in 0..num_iterations{
            use rayon::prelude::*;
            collision_lists.par_iter_mut().for_each(|cols|{
                for col in cols.iter_mut(){
                    let [a,b]=col.bots.get_mut();
                    let vel=b.vel-a.vel;
                    let vn=col.bias+vel.dot(col.offset_normal)*mag;
                    //let vn=vn.max(0.0);
                    let k=col.offset_normal*vn;
                    a.vel-=k;
                    b.vel+=k;
                }
            });  
        }

        let a4=now.elapsed().as_millis();


        let mut circles = canvas.circles();
        
        for b in bots.iter_mut() {
            if b.vel.x.is_nan() || b.vel.y.is_nan(){
                b.vel=vec2same(0.0);
            }
            b.vel+=vec2(0.0,0.01);
            b.pos+=b.vel;
            circles.add(b.pos.into());
        }

        circles.send_and_uniforms(canvas,radius*2.0).with_color([1.0, 1.0, 0.0, 0.6]).draw();
        
        let a5=now.elapsed().as_millis();

        println!("yo= {} {} {} {} {}",a1,a2-a1,a3-a2,a4-a3,a5-a4);
    })
}




use std::sync::atomic::AtomicPtr;

struct Cpair(pub [AtomicPtr<Bot>;2]);
impl Cpair{
    fn get_mut(&mut self)->[&mut Bot;2]{
        let [a,b]=&mut self.0;
        [unsafe{&mut **a.get_mut()},unsafe{&mut **b.get_mut()}]
    }
}

struct Collision{
    bots:Cpair,
    offset_normal:Vec2<f32>,
    bias:f32,
}
impl Collision{
    fn new(radius:f32,num_iterations_inv:f32,a:&mut Bot,b:&mut Bot)->Option<Self>{
        let offset=b.pos-a.pos;
        //TODO this can be optimized. computing distance twice
        let offset_normal=offset.normalize_to(1.0);
        let distance=offset.magnitude();
        
        if distance>0.00001 && distance<radius*2.0{
            let bias=0.3*(radius*2.0-distance)*num_iterations_inv;
            Some(Collision{
                bots:Cpair([AtomicPtr::new(a as *mut _),AtomicPtr::new(b as *mut _)]),
                offset_normal,
                bias
            })
        }else{
            None
        }
    }
}

