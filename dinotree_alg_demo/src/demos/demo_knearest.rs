use crate::support::prelude::*;
use std::cell::RefCell;


#[derive(Copy, Clone)]
struct Bot {
    id: usize,
    pos: Vec2<f32>,
    radius: Vec2<f32>,
}

impl analyze::HasId for Bot {
    fn get_id(&self) -> usize {
        self.id
    }
}

pub struct KnearestDemo {
    tree: DinoTreeOwned<axgeom::XAXISS, F32n, Bot>,
    dim: Rect<F32n>,
}


impl KnearestDemo {
    pub fn new(dim: Rect<F32n>) -> KnearestDemo {
        let bots: Vec<_> = UniformRandGen::new(dim.inner_into())
            .with_radius(2.0, 50.0)
            .take(40)
            .enumerate()
            .map(|(id, (pos, radius))| Bot { id, pos, radius })
            .collect();

        let tree = DinoTreeOwned::new_par(axgeom::XAXISS, bots, |bot| {
            Rect::from_point(bot.pos, bot.radius)
                .inner_try_into()
                .unwrap()
        });

        KnearestDemo { tree, dim }
    }
}

impl DemoSys for KnearestDemo {
    fn step(
        &mut self,
        cursor: Vec2<F32n>,
        c: &piston_window::Context,
        g: &mut piston_window::G2d,
        check_naive: bool,
    ) {
        let tree = &mut self.tree;

        for bot in tree.get_aabb_bots().iter() {
            draw_rect_f32([0.0, 0.0, 0.0, 0.3], bot.get().as_ref(), c, g);
        }

        struct Kn<'a, 'c: 'a> {
            draw: bool,
            c: &'a Context,
            g: RefCell<&'a mut G2d<'c>>,
        };

        impl<'a, 'c: 'a> Knearest for Kn<'a, 'c> {
            type T = BBoxPtr<F32n, Bot>;
            type N = F32n;

            fn distance_to_bot(&self, point: Vec2<Self::N>, bot: &Self::T) -> Self::N {
                if self.draw {
                    draw_rect_f32(
                        [0.0, 0.0, 1.0, 0.5],
                        bot.get().as_ref(),
                        self.c,
                        &mut self.g.borrow_mut(),
                    );
                }
                self.distance_to_rect(point, bot.get())
            }
            fn distance_to_rect(&self, point: Vec2<Self::N>, rect: &Rect<Self::N>) -> Self::N {
                let dis = rect.as_ref().distance_squared_to_point(point.inner_into());
                let dis = match dis {
                    Some(dis) => dis,
                    None => {
                        //If a point is insert a rect, the distance to it is zero.
                        //So if multiple points are inside of a rect, its not clear the order in which
                        //they should be returned.
                        //So in the case that a point is in the rect, we establish our own ordering,
                        //by falling back on the distance between the center of a rect and the point.
                        //Since the distance between a rect and a point that is outside of the rect is
                        //guarenteeded to be positive, we have all the negative numbers in which to
                        //apply our custom ordering for bots that are inside of the rect.

                        //The main reason that we are doing this is so that there arn't
                        //multiple solutions to the k_nearest problem so that we can easily
                        //verify the solution against the naive implementation.

                        //If you don't care about a single solution existing, you can simply return zero
                        //for the cases that the point is inside of the rect.

                        0.0
                        //-(bot.inner.pos-point.inner_into()).magnitude2()
                    }
                };
                f32n(dis)
            }
        }

        let cols = [
            [1.0, 0.0, 0.0, 0.8], //red closest
            [0.0, 1.0, 0.0, 0.8], //green second closest
            [0.0, 0.0, 1.0, 0.8], //blue third closets
        ];

        struct Res {
            rect: Rect<F32n>,
            mag: F32n,
        }
        let mut vv = {
            let mut kn = Kn {
                c: &c,
                g: RefCell::new(g),
                draw: true,
            };
            tree.get_mut().k_nearest_mut(cursor, 3, &mut kn, self.dim)
        };
        let mut vv: Vec<_> = vv
            .drain(..)
            .map(|a| Res {
                rect: *a.bot.get(),
                mag: a.mag,
            })
            .collect();

        if check_naive {
            let mut kn = Kn {
                c: &c,
                g: RefCell::new(g),
                draw: false,
            };
            analyze::NaiveAlgs::new(unsafe { tree.get_aabb_bots_mut_not_protected() }).assert_k_nearest_mut(
                cursor,
                3,
                &mut kn,
                self.dim,
            );
        }

        let vv_iter = dinotree_alg::util::SliceSplit::new(&mut vv, |a, b| a.mag == b.mag);

        for (a, color) in vv_iter.zip(cols.iter()) {
            
            if let Some(k) = a.first(){
                draw_circle_f32(*color,cursor.inner_into(),k.mag.into_inner(),c,g);
            }
            for b in a.iter() {

                draw_rect_f32(*color, b.rect.as_ref(), c, g);
            }
        }
    }
}
