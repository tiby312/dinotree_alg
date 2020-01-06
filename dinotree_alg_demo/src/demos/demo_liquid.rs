use crate::support::prelude::*;
use duckduckgeo;

use axgeom::Rect;

#[derive(Copy, Clone, Debug)]
pub struct Liquid {
    pub pos: Vec2<f32>,
    pub vel: Vec2<f32>,
    pub acc: Vec2<f32>,
}

impl Liquid {
    pub fn new(pos: Vec2<f32>) -> Liquid {
        let z = vec2same(0.0);

        Liquid {
            pos,
            acc: z,
            vel: z,
        }
    }

    pub fn solve(&mut self, b: &mut Self, radius: f32) -> f32 {
        let diff = b.pos - self.pos;

        let dis_sqr = diff.magnitude2();

        if dis_sqr < 0.00001 {
            self.acc += vec2(1.0, 0.0);
            b.acc -= vec2(1.0, 0.0);
            return 0.0;
        }

        if dis_sqr >= (2. * radius) * (2. * radius) {
            return 0.0;
        }

        let dis = dis_sqr.sqrt();

        //d is zero if barely touching, 1 is overlapping.
        //d grows linearly with position of bots
        let d = 1.0 - (dis / (radius * 2.));

        let spring_force_mag = -(d - 0.5) * 0.02;

        let velociy_diff = b.vel - self.vel;
        let damping_ratio = 0.00027;
        let spring_dampen = velociy_diff.dot(diff) * (1. / dis) * damping_ratio;

        let spring_force = diff * (1. / dis) * (spring_force_mag + spring_dampen);

        self.acc += spring_force;
        b.acc -= spring_force;

        spring_force_mag
    }
}

impl duckduckgeo::BorderCollideTrait for Liquid {
    type N = f32;
    fn pos_vel_mut(&mut self) -> (&mut Vec2<f32>, &mut Vec2<f32>) {
        (&mut self.pos, &mut self.vel)
    }
}

impl duckduckgeo::RepelTrait for Liquid {
    type N = f32;
    fn pos(&self) -> Vec2<f32> {
        self.pos
    }
    fn add_force(&mut self, acc: Vec2<f32>) {
        self.acc += acc;
    }
}

pub fn make_demo(dim: Rect<F32n>) -> Demo {
    let radius = 50.0;
    let mut bots: Vec<_> = UniformRandGen::new(dim.inner_into())
        .take(2000)
        .map(|pos| Liquid::new(pos))
        .collect();

    Demo::new(move |cursor, canvas, _check_naive| {
        let mut k = bbox_helper::create_bbox_mut(&mut bots, |bot| {
            let p = bot.pos;
            let r = radius;
            Rect::new(p.x - r, p.x + r, p.y - r, p.y + r)
                .inner_try_into::<NotNan<f32>>()
                .unwrap()
        });

        let mut tree = DinoTree::new_par(&mut k);

        tree.find_collisions_mut_par(|mut a, mut b| {
            let _ = a.inner_mut().solve(b.inner_mut(), radius);
        });

        let vv = vec2same(100.0).inner_try_into().unwrap();
        let cc = cursor.inner_into();

        tree.for_all_in_rect_mut(&axgeom::Rect::from_point(cursor, vv), |mut b| {
            let _ = duckduckgeo::repel_one(b.inner_mut(), cc, 0.001, 100.0);
        });

        {
            let dim2 = dim.inner_into();
            tree.for_all_not_in_rect_mut(&dim, |mut a| {
                duckduckgeo::collide_with_border(a.inner_mut(), &dim2, 0.5);
            });
        }

        for b in bots.iter_mut() {
            b.pos += b.vel;
            b.vel += b.acc;
            b.acc = vec2same(0.0);
        }

        let mut circle = canvas.circles();
        for bot in bots.iter() {
            circle.add(bot.pos);
        }
        circle.send_and_draw([1.0, 0.6, 0.7, 0.5],2.0);
    })
}
