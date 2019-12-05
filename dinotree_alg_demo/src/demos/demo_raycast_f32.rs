use crate::support::prelude::*;
use std;


use axgeom::Ray;

struct RayT<'a, 'c: 'a> {
    pub radius: f32,
    pub c: &'a Context,
    pub g: &'a mut G2d<'c>,
}

impl<'a, 'c: 'a> RayCast for RayT<'a, 'c> {
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
        c: &piston_window::Context,
        g: &mut piston_window::G2d,
        check_naive: bool,
    ) {
        //Draw bots
        for bot in self.tree.get_bots().iter() {
            draw_rect_f32([0.0, 0.0, 0.0, 0.3], bot.get().as_ref(), c, g);
        }

        let tree = &mut self.tree;
        let dim=self.dim;
        let radius=self.radius;
        {
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
                                c: &c,
                                g,
                            },
                        );
                    });
                }

                
                let res = tree.get_tree_mut().raycast_fine_mut(
                    ray,
                    &mut RayT {
                        radius: self.radius,
                        c: &c,
                        g,
                    },
                    self.dim,
                );



                let dis = match res {
                    RayCastResult::Hit((_,dis))=>dis.into_inner(),
                    RayCastResult::NoHit=>800.0
                };

                let Vec2{x:ppx,y:ppy}=ray.inner_into().point_at_tval(dis);

                let arr = [
                    ray.point.x.into_inner() as f64,
                    ray.point.y.into_inner() as f64,
                    ppx as f64,
                    ppy as f64,
                ];
                line(
                    [0.0, 0.0, 1.0, 0.2], // black
                    1.0,                  // radius of line
                    arr,                  // [x0, y0, x1,y1] coordinates of line
                    c.transform,
                    g,
                );
            }
        }
    }
}
