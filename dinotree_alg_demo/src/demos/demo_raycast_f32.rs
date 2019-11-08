use self::raycast::RayTrait;
use crate::support::prelude::*;
use dinotree_alg::raycast;
use dinotree_alg::raycast::RayIntersectResult;
use duckduckgeo;
use std;

struct RayT<'a, 'c: 'a> {
    pub radius: f32,
    pub c: &'a Context,
    pub g: &'a mut G2d<'c>,
}

impl<'a, 'c: 'a> RayTrait for RayT<'a, 'c> {
    type N = F32n;
    type T = BBoxPtr<F32n, Bot>;

    fn compute_distance_to_bot(
        &self,
        ray: &raycast::Ray<Self::N>,
        bot: &Self::T,
    ) -> RayIntersectResult<Self::N> {
        let ray: duckduckgeo::Ray<f32> = Ray {
            point: ray.point.inner_into(),
            dir: ray.dir.inner_into(),
        };

        match ray_intersects_circle(&ray, bot.inner().center, self.radius) {
            IntersectsBotResult::Hit(val) => RayIntersectResult::Hit(val),
            IntersectsBotResult::NoHit => RayIntersectResult::NoHit,
            IntersectsBotResult::Inside => {
                RayIntersectResult::Hit(0.0)

                //Return none if you do not want results that intersect the ray origin.
                //None
            }
        }
        .inner_try_into()
        .unwrap()
    }
    fn compute_distance_to_rect(
        &self,
        ray: &raycast::Ray<Self::N>,
        rect: &Rect<Self::N>,
    ) -> RayIntersectResult<Self::N> {
        let ray: duckduckgeo::Ray<f32> = Ray {
            point: ray.point.inner_into(),
            dir: ray.dir.inner_into(),
        };
        let rect: &Rect<f32> = rect.as_ref();

        let k = ray_intersects_box(&ray, &rect);
        match k {
            IntersectsBotResult::Hit(val) => RayIntersectResult::Hit(val),
            IntersectsBotResult::NoHit => RayIntersectResult::NoHit,
            IntersectsBotResult::Inside => {
                RayIntersectResult::Hit(0.0)
                //Return none if you do not want results that intersect the ray origin.
                //None
            }
        }
        .inner_try_into()
        .unwrap()
    }
}

#[derive(Copy, Clone)]
struct Bot {
    id: usize,
    center: Vec2<f32>,
}

impl dinotree_alg::assert::HasId for Bot {
    fn get_id(&self) -> usize {
        self.id
    }
}

pub struct RaycastF32Demo {
    tree: DinoTreeOwned<axgeom::XAXISS, F32n, Bot>,
    dim: Rect<F32n>,
    radius: f32,
}
impl RaycastF32Demo {
    pub fn new(dim: Rect<F32n>) -> Self {
        let radius = 20.0;
        let vv = UniformRandGen::new(dim.inner_into())
            .enumerate()
            .map(|(id, center)| Bot { id, center })
            .take(100)
            .collect();

        let tree = create_owned_par(axgeom::XAXISS, vv, |a| {
            Rect::from_point(a.center, vec2same(radius))
                .inner_try_into()
                .unwrap()
        });

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
        for bot in self.tree.get_aabb_bots().iter() {
            draw_rect_f32([0.0, 0.0, 0.0, 0.3], bot.get().as_ref(), c, g);
        }

        let tree = &mut self.tree;

        {
            for dir in 0..360i32 {
                let dir = dir as f32 * (std::f32::consts::PI / 180.0);
                let x = (dir.cos() * 20.0) as f32;
                let y = (dir.sin() * 20.0) as f32;

                let ray = {
                    let k = vec2(x, y).inner_try_into().unwrap();
                    raycast::Ray {
                        point: cursor,
                        dir: k,
                    }
                };

                if check_naive {
                    dinotree_alg::assert::assert_raycast(
                        unsafe { tree.get_aabb_bots_mut_not_protected() },
                        self.dim,
                        ray,
                        &mut RayT {
                            radius: self.radius,
                            c: &c,
                            g,
                        },
                    );
                }

                use raycast::*;

                let res = raycast_mut(
                    tree.get_mut(),
                    self.dim,
                    ray,
                    &mut RayT {
                        radius: self.radius,
                        c: &c,
                        g,
                    },
                );

                let (ppx,ppy) = match res{
                    RayCastResult::Hit(_,dis)=>{
                        let ppx = ray.point.x + ray.dir.x * dis;
                        let ppy = ray.point.y + ray.dir.y * dis;
                        (ppx, ppy)
                    },
                    RayCastResult::NoHit=>{
                        let ppx = ray.point.x + ray.dir.x * 800.0;
                        let ppy = ray.point.y + ray.dir.y * 800.0;
                        (ppx, ppy)
                    }
                };


                let arr = [
                    ray.point.x.into_inner() as f64,
                    ray.point.y.into_inner() as f64,
                    ppx.into_inner() as f64,
                    ppy.into_inner() as f64,
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
