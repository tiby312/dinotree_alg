use crate::support::prelude::*;
use std;

use axgeom::Ray;



#[derive(Copy, Clone)]
struct Bot {
    id: usize,
    center: Vec2<f32>,
}


pub fn make_demo(dim: Rect<F32n>,canvas:&mut SimpleCanvas) -> Demo {
    let radius = 10.0;
    let vv: Vec<_> = UniformRandGen::new(dim.inner_into())
        .enumerate()
        .map(|(id, center)| {
            let b = Bot { id, center };
            let r = Rect::from_point(center, vec2same(radius))
                .inner_try_into()
                .unwrap();
            bbox(r, b)
        })
        .take(300)
        .collect();

    let mut tree = DinoTreeOwned::new(vv);

    //Draw bots
    let mut r = canvas.circles();
    for bot in tree.get_bots().iter() {
        r.add(bot.inner().center.into());
    }
    let circle_save=r.save(canvas);

    


    Demo::new(move |cursor, canvas, check_naive| {
        circle_save.uniforms(canvas,radius*2.0).with_color([0.0, 0.0, 0.0, 0.3]).draw();
    
        {
            let mut ray_cast = canvas.lines(1.0);

            for dir in 0..360i32
            //let dir=200;
            {
                let dir = dir as f32 * (std::f32::consts::PI / 180.0);
                let x = (dir.cos() * 20.0) as f32;
                let y = (dir.sin() * 20.0) as f32;

                let ray = {
                    let k = vec2(x, y).inner_try_into().unwrap();
                    Ray {
                        point: cursor,
                        dir: k,
                    }
                };

                let mut radius=radius;
                
                if check_naive{
                    tree.as_tree_mut().assert_raycast_mut(ray,&mut radius,
                        move |_r,ray,rect| ray.inner_into::<f32>().cast_to_rect(rect.as_ref()).map(|a|f32n(a)),
                        move |r,ray,t|ray.inner_into::<f32>().cast_to_circle(t.inner().center, *r).map(|a| NotNan::new(a).unwrap()),
                     dim);
                }

                
                let res = tree
                    .as_tree_mut()
                    .raycast_mut(ray,&mut radius,
                        move |_r,ray,rect| ray.inner_into::<f32>().cast_to_rect(rect.as_ref()).map(|a|f32n(a)),
                        move |r,ray,t|ray.inner_into::<f32>().cast_to_circle(t.inner().center, *r).map(|a| NotNan::new(a).unwrap())
                , dim);
                
                
                /*
                let mut n=analyze::NaiveAlgs::new(tree.as_tree_mut().get_bots_mut());
                let res = n
                    .raycast_mut(ray,&mut radius,
                        move |_r,ray,rect| ray.inner_into::<f32>().cast_to_rect2(rect.as_ref()).map(|a|f32n(a)),
                        move |r,ray,t|ray.inner_into::<f32>().cast_to_circle(t.inner().center, *r).map(|a| NotNan::new(a).unwrap()),
                        dim);
                */

                let dis = match res {
                    RayCastResult::Hit((_, dis)) => dis.into_inner(),
                    RayCastResult::NoHit => 800.0,
                };

                let end = ray.inner_into().point_at_tval(dis);
                ray_cast.add(ray.point.inner_into().into(), end.into());
                
            }
            ray_cast.send_and_uniforms(canvas).with_color([1.0, 1.0, 1.0, 0.4]).draw();
        }
    })
}
