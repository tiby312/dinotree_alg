use crate::support::prelude::*;
use std;

use axgeom::Ray;

struct RayT {
    pub radius: f32,
}

impl RayCast for RayT {
    type N = F32n;
    type T = BBox<F32n, Bot>;

    fn compute_distance_to_bot(
        &self,
        ray: &Ray<Self::N>,
        bot: &Self::T,
    ) -> axgeom::CastResult<Self::N> {
        ray.inner_into::<f32>()
            .cast_to_circle(bot.inner().center, self.radius)
            .map(|a| NotNan::new(a).unwrap())
    }
    fn compute_distance_to_rect(
        &self,
        ray: &Ray<Self::N>,
        rect: &Rect<Self::N>,
    ) -> axgeom::CastResult<Self::N> {
        ray.cast_to_rect(rect)
    }
}

#[derive(Copy, Clone)]
struct Bot {
    id: usize,
    center: Vec2<f32>,
}

impl analyze::HasId for Bot {
    fn get_id(&self) -> usize {
        self.id
    }
}

pub fn make_demo(dim: Rect<F32n>,canvas:&mut SimpleCanvas) -> Demo {
    let radius = 20.0;
    let vv: Vec<_> = UniformRandGen::new(dim.inner_into())
        .enumerate()
        .map(|(id, center)| {
            let b = Bot { id, center };
            let r = Rect::from_point(center, vec2same(radius))
                .inner_try_into()
                .unwrap();
            bbox(r, b)
        })
        .take(100)
        .collect();

    let mut tree = DinoTreeOwned::new(vv);

    //Draw bots
    let mut r = canvas.circles(radius);
    for bot in tree.get_bots().iter() {
        r.add(bot.inner().center);
    }
    let circle_save=r.save();

    


    Demo::new(move |cursor, canvas, check_naive| {
        circle_save.draw(canvas,[0.0, 0.0, 0.0, 0.3]);
    
        {
            let mut ray_cast = canvas.lines(5.0);

            for dir in 0..360i32 {
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

                if check_naive {
                    tree.get_bots_mut(|bots| {
                        analyze::NaiveAlgs::new(bots).assert_raycast_mut(
                            dim,
                            ray,
                            &mut RayT { radius },
                        );
                    });
                }

                let res = tree
                    .as_tree_mut()
                    .raycast_fine_mut(ray, &mut RayT { radius }, dim);

                let dis = match res {
                    RayCastResult::Hit((_, dis)) => dis.into_inner(),
                    RayCastResult::NoHit => 800.0,
                };

                let end = ray.inner_into().point_at_tval(dis);
                ray_cast.add(ray.point.inner_into(), end);
            }
            ray_cast.send_and_draw([1.0, 1.0, 1.0, 0.3]);
        }
    })
}
