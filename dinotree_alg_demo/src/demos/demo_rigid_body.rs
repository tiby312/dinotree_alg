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
    impulse: Vec2<f32>, 
    impulse_avg: Vec2<f32>,
    force: Vec2<f32>,
}
impl Bot{

    pub fn handle_collision(&mut self, b: &mut Bot) {
        let a = self;

        let cc = 0.5;

        let pos_diff = b.pos - a.pos;

        let pos_diff_norm = pos_diff.normalize_to(1.0);

        let vel_diff = b.vel - a.vel;

        let im1 = 1.0;
        let im2 = 1.0;

        let vn = vel_diff.dot(pos_diff_norm);
        if vn > 0.0 {
            return;
        }

        let i = (-(1.0 + cc) * vn) / (im1 + im2);
        let impulse = pos_diff_norm * i;

        a.vel -= impulse * im1;
        b.vel += impulse * im2;
    }

}


pub fn make_demo(dim: Rect<F32n>) -> Demo {
    let num_bot = 8000;

    let radius = 2.0;

    let mut bots: Vec<_> = UniformRandGen::new(dim.inner_into())
        .take(num_bot)
        .map(|pos| Bot {
            pos,
            impulse:vec2same(0.0),
            impulse_avg:vec2same(0.0),
            vel: vec2same(0.0),
            force: vec2same(0.0),
        })
        .collect();

    Demo::new(move |cursor, canvas, _check_naive| {

        let mut k = bbox_helper::create_bbox_mut(&mut bots, |b| {
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
                let _ = duckduckgeo::repel_one(b.pos,&mut b.force, cc, 0.001, 20.0);
            }
        });

        
        let num_iterations=4;
        let step= radius;
        for _ in 0..num_iterations{
            let mut k:Vec<BBoxMut<NotNan<f32>,_>> = bbox_helper::create_bbox_mut(&mut bots, |b| {
                Rect::from_point(b.pos, vec2same(radius))
                    .inner_try_into()
                    .unwrap()
            });

            let mut tree = DinoTree::new_par(&mut k);
    
            tree.find_collisions_mut_par(|mut a, mut b| {
                let a=a.inner_mut();
                let b=b.inner_mut();


                let offset=b.pos-a.pos;

                let p=offset.normalize_to(1.0);

                let d=2.0*radius-offset.magnitude();
                if d>0.0{
                    let mag={
                        if  d < step{
                            a.handle_collision(b);
                            d
                        }else{
                            step
                        }
                        
                    };
                    
                    let k1=a.impulse-p*mag;
                    let k2=b.impulse+p*mag;
                    if !k1.x.is_nan() && !k2.x.is_nan() && !k1.y.is_nan() && !k2.y.is_nan(){
                        a.impulse=k1;
                        b.impulse=k2;
                    }else{
                        a.impulse=vec2(1.0,0.0);
                        b.impulse=vec2(-1.0,0.0);
                    }
                }
            });  

            for b in bots.iter_mut(){
                b.impulse=b.impulse.truncate_at(step);
                b.impulse_avg=b.impulse_avg*0.7+b.impulse*0.3;
                b.pos+=b.impulse_avg;
                b.impulse=vec2same(0.0);
            }          
        }
    

        for b in bots.iter_mut() {
            b.vel += b.force;
            b.force=vec2same(0.0);
            b.pos += b.vel;
        }


        let mut circles = canvas.circles();
        for bot in bots.iter() {
            circles.add(bot.pos.into()); //TODO we're not testing that the bots were draw in the right order
        }
        circles.send_and_uniforms(canvas,radius*2.0).with_color([1.0, 1.0, 0.0, 0.6]).draw();
        
    })
}
