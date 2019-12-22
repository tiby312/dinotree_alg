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

        ray.inner_into::<f32>().cast_to_circle(bot.inner().center,self.radius).map(|a|NotNan::new(a).unwrap())
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

pub struct RaycastF32Demo {
    tree: DinoTreeOwned<DefaultA, BBox<F32n, Bot>>,
    dim: Rect<F32n>,
    radius: f32,
}
impl RaycastF32Demo {
    pub fn new(dim: Rect<F32n>) -> Self {
        let radius = 20.0;
        let vv:Vec<_> = UniformRandGen::new(dim.inner_into())
            .enumerate()
            .map(|(id, center)| {
                let b=Bot { id, center };
                let r=Rect::from_point(center, vec2same(radius))
                .inner_try_into()
                .unwrap();
                bbox(r,b)
            })
            .take(100)
            .collect();

        let tree=DinoTreeOwned::new(vv);

        Self { tree, dim, radius }
    }
}

impl DemoSys for RaycastF32Demo {
    fn step(
        &mut self,
        cursor: Vec2<F32n>,
        mut sys:very_simple_2d::DrawSession,
        check_naive: bool,
    ) {
        //Draw bots
        let mut r=sys.circles([0.0,0.0,0.0,0.3],self.radius);
        for bot in self.tree.get_bots().iter() {
            r.add(bot.inner().center);
        }
        r.draw();
        drop(r);

        let tree = &mut self.tree;
        let dim=self.dim;
        let radius=self.radius;
        {
            let mut ray_cast=sys.lines([1.0,1.0,1.0,0.3],5.0);

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
                    tree.get_bots_mut(|bots|{
                        analyze::NaiveAlgs::new(bots).assert_raycast_mut(
                            dim,
                            ray,
                            &mut RayT {
                                radius,
                            },
                        );
                    });
                }

                
                let res = tree.as_tree_mut().raycast_fine_mut(
                    ray,
                    &mut RayT {
                        radius: self.radius,
                    },
                    self.dim,
                );



                let dis = match res {
                    RayCastResult::Hit((_,dis))=>dis.into_inner(),
                    RayCastResult::NoHit=>800.0
                };

                let end=ray.inner_into().point_at_tval(dis);
                ray_cast.add(ray.point.inner_into(),end);
                
            }
            ray_cast.draw();
        }
    }
}
