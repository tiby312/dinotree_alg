use crate::support::prelude::*;

use duckduckgeo;

#[derive(Copy, Clone)]
pub struct Bot {
    pos: Vec2<f32>,
    vel: Vec2<f32>, 
    push: Vec2<f32>, 
    acc_avg_push: Vec2<f32>,
    force: Vec2<f32>,
}


pub fn make_demo(dim: Rect<F32n>) -> Demo {
    let num_bot = 4000;

    let radius = 5.0;

    let mut bots: Vec<_> = UniformRandGen::new(dim.inner_into())
        .take(num_bot)
        .map(|pos| Bot {
            pos,
            push:vec2same(0.0),
            acc_avg_push:vec2same(0.0),
            vel: vec2same(0.0),
            force: vec2same(0.0),
        })
        .collect();

    Demo::new(move |cursor, canvas, check_naive| {
        

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

        

        if !check_naive {
            tree.find_collisions_mut_par(|mut a, mut b| {
                let a=a.inner_mut();
                let b=b.inner_mut();


                let _ = duckduckgeo::repel([(a.pos,&mut a.push), (b.pos,&mut b.push)], 0.001, 10.0);
            });
        } else {
          
        }

        for b in bots.iter_mut() {
            b.push=b.push.truncate_at(1.0);

            //weighted average.
            let pp=(b.acc_avg_push*0.6+b.push*0.4) / 1.0;
            b.pos+=pp;
            //let pp=pp*0.4;
            
            b.acc_avg_push=pp;
            b.push=vec2same(0.0);

            b.vel += b.force;
            //non linear drag
            b.vel *= 0.9;
            b.pos += b.vel;
            b.force = vec2same(0.0);
        }


        let mut circles = canvas.circles();
        for bot in bots.iter() {
            circles.add(bot.pos.into()); //TODO we're not testing that the bots were draw in the right order
        }
        circles.send_and_uniforms(canvas,radius).with_color([1.0, 1.0, 0.0, 0.6]).draw();
        
    })
}
