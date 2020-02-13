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

pub fn make_demo(dim: Rect<F32n>) -> Demo {
    let num_bot = 4000;

    let radius = 5.0;

    let mut bots: Vec<_> = UniformRandGen::new(dim.inner_into())
        .take(num_bot)
        .map(|pos| Bot {
            pos,
            vel: vec2same(0.0)
        })
        .collect();

    Demo::new(move |cursor, canvas, _check_naive| {

        
        let mut k:Vec<BBoxMut<NotNan<f32>,_>> = bbox_helper::create_bbox_mut(&mut bots, |b| {
            Rect::from_point(b.pos, vec2same(radius))
                .inner_try_into()
                .unwrap()
        });

        let mut tree = DinoTree::new_par(&mut k);
    
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

        use std::sync::atomic::AtomicPtr;

        struct Collision{
            bots:[AtomicPtr<Bot>;2],
            offset:Vec2<f32>,
            offset_normal:Vec2<f32>,
            distance:f32,
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
                        bots:[AtomicPtr::new(a as *mut _),AtomicPtr::new(b as *mut _)],
                        offset,
                        offset_normal,
                        distance,
                        bias
                    })
                }else{
                    None
                }
            }

            fn get_mut(&mut self)->([&mut Bot;2],&Vec2<f32>,&Vec2<f32>,f32,f32){
                let [a,b]=&mut self.bots;
                
                (
                    [unsafe{&mut **a.get_mut()},unsafe{&mut **b.get_mut()}],
                    &self.offset,
                    &self.offset_normal,
                    self.distance,
                    self.bias
                )

            }

        }


        let num_iterations=10;
        let num_iterations_inv=1.0/num_iterations as f32;
        let mut collisions=tree.find_collisions_mut_par_ext(
            |_|Vec::new(),
            |a,mut b|a.append(&mut b),
            |arr,mut a,mut b|{
                if let Some(k)=Collision::new(radius,num_iterations_inv,a.inner_mut(),b.inner_mut()){
                    arr.push(k)   
                }
            },
            Vec::new()
        );
                    
        for _ in 0..num_iterations{
            for collision in collisions.iter_mut(){
                let ([a,b],_,&offset_normal,_,bias)=collision.get_mut();
                    
                let vel=b.vel-a.vel;
                let vn=bias+vel.dot(offset_normal)*(0.03*num_iterations_inv);
                let drag=-vel.dot(offset_normal)*0.01;
                let vn=vn.max(0.0);
                let k=offset_normal*(vn+drag);
                a.vel=a.vel-k;
                b.vel=b.vel+k;
            };  
        }

        for b in bots.iter_mut() {
            if b.vel.x.is_nan() || b.vel.y.is_nan(){
                b.vel=vec2same(0.0);
            }
            b.vel+=vec2(0.0,0.01);
            b.pos += b.vel;
        }

        let mut circles = canvas.circles();
        for bot in bots.iter() {
            circles.add(bot.pos.into());
        }
        circles.send_and_uniforms(canvas,radius*2.0).with_color([1.0, 1.0, 0.0, 0.6]).draw();
        
    })
}
