use crate::support::prelude::*;

use std;
use std::cell::RefCell;

use axgeom::Ray;

mod ray_f32 {
    use super::*;

    pub struct RayT<'a> {
        pub rects: Option<RefCell<egaku2d::shapes::RectSession<'a>>>,
        pub height: usize,
    }

    impl<'a> RayCast for RayT<'a> {
        type T = BBox<F32n, Bot2>;
        type N = F32n;

        fn compute_distance_to_bot(
            &self,
            ray: &Ray<Self::N>,
            bot: &Self::T,
        ) -> axgeom::CastResult<Self::N> {
            if let Some(r) = &self.rects {
                r.borrow_mut().add(bot.get().inner_into().as_arr());
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

pub fn make_demo(dim: Rect<F32n>,canvas:&mut SimpleCanvas) -> Demo {
    let ii: Vec<_> = UniformRandGen::new(dim.inner_into())
        .with_radius(1.0, 5.0)
        .enumerate()
        .take(500)
        .map(|(id, (pos, radius))| {
            bbox(
                Rect::from_point(pos, radius).inner_try_into().unwrap(),
                Bot2 { id },
            )
        })
        .collect();

    let mut counter: f32 = 0.0;
    let mut tree = DinoTreeOwned::new_par(ii);

    let mut rects = canvas.rects();
    for bot in tree.get_bots().iter() {
        rects.add(bot.get().inner_into().as_arr());
    }
    let rect_save=rects.save();



    Demo::new(move |cursor, canvas, check_naive| {
        let ray: Ray<F32n> = {
            counter += 0.004;
            let point: Vec2<f32> = cursor.inner_into::<f32>().inner_as();
            //*counter=10.0;
            let dir = vec2(counter.cos() * 10.0, counter.sin() * 10.0);

            let dir = dir.inner_as();
            Ray { point, dir }.inner_try_into().unwrap()
        };

        rect_save.uniforms(canvas).with_color([0.0, 0.0, 0.0, 0.3]).draw();

        let height = tree.as_tree().get_height();

        if check_naive {
            tree.get_bots_mut(|bots| {
                analyze::NaiveAlgs::new(bots).assert_raycast_mut(
                    dim,
                    ray,
                    &mut ray_f32::RayT {
                        rects: None,
                        height,
                    },
                );
            });
        }

        let test = {
            let rects = canvas.rects();
            let mut rr = ray_f32::RayT {
                rects: Some(RefCell::new(rects)),
                height,
            };
            let test = tree.as_tree_mut().raycast_fine_mut(ray, &mut rr, dim);
            rr.rects.unwrap().borrow_mut().uniforms().with_color([4.0, 0.0, 0.0, 0.4]).send_and_draw();
            test
        };

        let ray: Ray<f32> = ray.inner_into();

        let dis = match test {
            RayCastResult::Hit((_, dis)) => dis.into_inner(),
            RayCastResult::NoHit => 800.0,
        };

        let end = ray.point_at_tval(dis);

        canvas.lines(2.0)
            .add(ray.point.as_arr(), end.as_arr())
            .uniforms()
            .with_color([1., 1., 1., 0.2])
            .send_and_draw();

        /*
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
        self.tree.as_tree().draw( &mut dd, &self.dim);
        */
    })
}
