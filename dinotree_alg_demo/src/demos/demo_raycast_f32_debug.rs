use crate::support::prelude::*;
use dinotree_alg;
use std;
use std::cell::RefCell;

use axgeom::Ray;

mod ray_f32 {
    use super::*;

    use duckduckgeo;

    pub struct RayT<'a, 'c: 'a> {
        pub c: &'a Context,
        pub g: RefCell<&'a mut G2d<'c>>,
        pub height: usize,
        pub draw: bool,
    }

    impl<'a, 'c: 'a> RayCast for RayT<'a, 'c> {
        type T = BBoxPtr<F32n, Bot2>;
        type N = F32n;

        fn compute_distance_to_bot(
            &self,
            ray: &Ray<Self::N>,
            bot: &Self::T,
        ) -> axgeom::CastResult<Self::N> {
            if self.draw {
                draw_rect_f32(
                    [1.0, 0.0, 0.0, 0.5],
                    bot.get().as_ref(),
                    self.c,
                    &mut self.g.borrow_mut(),
                );
            }
            Self::compute_distance_to_rect(self, ray, bot.get())
        }

        fn compute_distance_to_rect(
            &self,
            ray: &Ray<Self::N>,
            rect: &Rect<Self::N>,
        ) -> axgeom::CastResult<Self::N> {
            ray.cast_to_rect(&rect)
        }
    }

}

#[derive(Copy, Clone, Debug)]
pub struct Bot2 {
    id: usize,
}

impl analyze::HasId for Bot2 {
    fn get_id(&self) -> usize {
        self.id
    }
}

pub struct RaycastF32DebugDemo {
    tree: DinoTreeOwned<DefaultA, F32n, Bot2>,
    counter: f32,
    dim: Rect<F32n>,
}
impl RaycastF32DebugDemo {
    pub fn new(dim: Rect<F32n>) -> RaycastF32DebugDemo {
        let vv: Vec<_> = (0..1000).map(|id| (Bot2 { id })).collect();

        let mut ii = UniformRandGen::new(dim.inner_into())
            .with_radius(1.0, 5.0)
            .map(|(pos, radius)| Rect::from_point(pos, radius).inner_try_into().unwrap());

        let tree = DinoTreeOwned::new_par( vv, |_a| ii.next().unwrap());

        RaycastF32DebugDemo {
            tree,
            counter: 0.0,
            dim,
        }
    }
}

impl DemoSys for RaycastF32DebugDemo {
    fn step(
        &mut self,
        cursor: Vec2<F32n>,
        c: &piston_window::Context,
        g: &mut piston_window::G2d,
        check_naive: bool,
    ) {
        let counter = &mut self.counter;

        let ray: Ray<F32n> = {
            *counter += 0.004;
            let point: Vec2<f32> = cursor.inner_into::<f32>().inner_as();
            //*counter=10.0;
            let mut dir = vec2(counter.cos() * 10.0, counter.sin() * 10.0);

            let dir = dir.inner_as();
            Ray { point, dir }.inner_try_into().unwrap()
        };

        for bot in self.tree.get_aabb_bots().iter() {
            draw_rect_f32([0.0, 0.0, 0.0, 0.3], bot.get().as_ref(), c, g);
        }

        let height = self.tree.get_height();

        if check_naive {
            analyze::NaiveAlgs::new(unsafe { self.tree.get_aabb_bots_mut_not_protected() }).assert_raycast_mut(
                self.dim,
                ray,
                &mut ray_f32::RayT {
                    draw: false,
                    c: &c,
                    g: RefCell::new(g),
                    height,
                },
            );
        }

        let test = self.tree.get_mut().raycast_fine_mut(
            ray,
            &mut ray_f32::RayT {
                draw: true,
                c: &c,
                g: RefCell::new(g),
                height,
            },
            self.dim,
        );

        let ray: Ray<f32> = ray.inner_into();


        let dis = match test {
            RayCastResult::Hit(_,dis)=>dis.into_inner(),
            RayCastResult::NoHit=>800.0
        };

        let Vec2{x:ppx,y:ppy}=ray.point_at_tval(dis);


        let arr = [
            ray.point.x as f64,
            ray.point.y as f64,
            ppx as f64,
            ppy as f64,
        ];
        line(
            [0.0, 0.0, 0.0, 1.0], // black
            2.0,                  // radius of line
            arr,                  // [x0, y0, x1,y1] coordinates of line
            c.transform,
            g,
        );

        struct Bla<'a, 'b: 'a> {
            c: &'a Context,
            g: &'a mut G2d<'b>,
        }
        impl<'a, 'b: 'a> DividerDrawer for Bla<'a, 'b> {
            type N = F32n;
            fn draw_divider<A: axgeom::Axis>(
                &mut self,
                axis: A,
                div: F32n,
                cont: [F32n; 2],
                length: [F32n; 2],
                depth: usize,
            ) {
                let div = div.into_inner();
                let length = [length[0].into_inner(), length[1].into_inner()];
                let cont = [cont[0].into_inner(), cont[1].into_inner()];

                let arr = if axis.is_xaxis() {
                    [div as f64, length[0] as f64, div as f64, length[1] as f64]
                } else {
                    [length[0] as f64, div as f64, length[1] as f64, div as f64]
                };

                let radius = (1isize.max(5 - depth as isize)) as f64;

                line(
                    [0.0, 0.0, 0.0, 0.5], // black
                    radius,               // radius of line
                    arr,                  // [x0, y0, x1,y1] coordinates of line
                    self.c.transform,
                    self.g,
                );

                let [x1, y1, w1, w2] = if axis.is_xaxis() {
                    [cont[0], length[0], cont[1] - cont[0], length[1] - length[0]]
                } else {
                    [length[0], cont[0], length[1] - length[0], cont[1] - cont[0]]
                };

                let [x1, y1, w1, w2] = [x1 as f64, y1 as f64, w1 as f64, w2 as f64];

                let square = [x1, y1, w1, w2];
                rectangle([0.0, 1.0, 1.0, 0.2], square, self.c.transform, self.g);
            }
        }

        let mut dd = Bla { c: &c, g };
        self.tree.get().draw( &mut dd, &self.dim);
    }
}
