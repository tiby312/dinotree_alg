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
    let num_bot = 400;

    let radius = 10.0;

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
                duckduckgeo::collide_with_border(&mut a.pos,&mut a.vel, &dim2, 0.5);
            });
        }

        let vv = vec2same(50.0).inner_try_into().unwrap();
        let cc = cursor.inner_into();
        tree.for_all_in_rect_mut(&axgeom::Rect::from_point(cursor, vv), |mut b| {
            let b=b.inner_mut();
            
            let offset=b.pos-cursor.inner_into();
            if offset.magnitude()<50.0*0.5{
                let _ = duckduckgeo::repel_one(b.pos,&mut b.vel, cc, 0.001, 20.0);
            }
        });

        let mut collisions=Vec::new();
        tree.find_collisions_mut(|mut a,mut b|{
            collisions.push((a.inner_mut() as *mut Bot,b.inner_mut() as *mut Bot));
        });
        let num_iterations=10;
        for _ in 0..num_iterations{
            for (a,b) in collisions.iter_mut(){
                let a = unsafe{&mut **a};
                let b = unsafe{&mut **b};

                let offset=b.pos-a.pos;

                let dis=offset.magnitude();
                if dis<radius*2.0{
                    let normal=offset.normalize_to(1.0);

                    let vel=b.vel-a.vel;

                    let bias=0.001*(radius*2.0-dis)*num_iterations as f32;
                    //let bias=0.0;
                    let vn=bias+vel.dot(normal)*(0.0005*num_iterations as f32);


                    let drag=-vel.dot(normal)*0.01;
                    //let drag=0.0;

                    let vn=vn.max(0.0);

                    let avel=a.vel-normal*(vn+drag);
                    let bvel=b.vel+normal*(vn+drag);
                    if avel.x.is_nan() || avel.y.is_nan() || bvel.x.is_nan() || b.vel.y.is_nan(){

                    }else{
                        a.vel=avel;
                        b.vel=bvel;
                    }
                }
            };  
        }

        for b in bots.iter_mut() {
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
