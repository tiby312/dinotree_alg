use crate::support::prelude::*;
use axgeom::Rect;

#[derive(Copy, Clone, Debug)]
pub struct RigidBody {
    pub old_pos: Vec2<f32>,
    pub pos: Vec2<f32>,
    pub push_vec: Vec2<f32>,
    pub vel: Vec2<f32>,
    pub acc: Vec2<f32>,
}

impl duckduckgeo::BorderCollideTrait for RigidBody {
    type N = f32;
    fn pos_vel_mut(&mut self) -> (&mut Vec2<f32>, &mut Vec2<f32>) {
        (&mut self.pos, &mut self.vel)
    }
}

impl RigidBody {
    pub fn new(pos: Vec2<f32>) -> RigidBody {
        let a = vec2same(0.0);
        RigidBody {
            pos,
            push_vec: a,
            vel: a,
            acc: a,
            old_pos: a,
        }
    }
    pub fn create_loose(&self, radius: f32) -> Rect<F32n> {
        axgeom::Rect::from_point(self.pos, vec2same(radius))
            .inner_try_into()
            .unwrap()
    }

    pub fn handle_collision(&mut self, b: &mut RigidBody) {
        let a = self;

        let cc = 0.5;

        let pos_diff = b.pos - a.pos;

        let pos_diff_norm = pos_diff.normalize_to(1.0);

        let vel_diff = b.vel - a.vel;

        let im1 = 1.0;
        let im2 = 1.0;

        let vn = vel_diff.dot(pos_diff_norm);
        if vn > 0.0 {
            return;
        }

        let i = (-(1.0 + cc) * vn) / (im1 + im2);
        let impulse = pos_diff_norm * i;

        a.vel -= impulse * im1;
        b.vel += impulse * im2;
    }

    pub fn push_away(&mut self, b: &mut Self, radius: f32, max_amount: f32) -> Option<f32> {
        let mut diff = b.pos - self.pos;
        let dis = diff.magnitude();

        if dis >= radius * 2.0 {
            return None;
        }

        if dis < 0.000001 {
            self.push_vec += vec2(0.01, 0.0);
            b.push_vec -= vec2(0.01, 0.0);
            return None;
        }

        let fff = radius * 2.0 - dis + 0.0001;

        let (moved, mag) = if fff < max_amount {
            (true, fff)
        } else {
            (false, max_amount)
        };

        if mag < 0.0 {
            panic!("impossible");
        }
        diff *= mag / dis;

        self.push_vec -= diff;
        b.push_vec += diff;

        if moved {
            Some(dis)
        } else {
            None
        }
    }

    pub fn push_away_from_border(&mut self, rect2: &Rect<f32>, _push_rate: f32) {
        let a = self;
        let xx = rect2.get_range(axgeom::XAXIS);
        let yy = rect2.get_range(axgeom::YAXIS);

        let (pos, _vel) = &mut a.pos_vel_mut();

        if pos.x < xx.start {
            pos.x = xx.start;
        }
        if pos.x > xx.end {
            pos.x = xx.end;
        }
        if pos.y < yy.start {
            pos.y = yy.start;
        }
        if pos.y > yy.end {
            pos.y = yy.end;
        }
    }
}

pub fn handle_rigid_body(
    dim: &Rect<F32n>,
    bodies: &mut [RigidBody],
    ball_size: f32,
    push_rate: f32,
    num_rebal: usize,
    num_query: usize,
    func: impl Fn(&mut RigidBody, &mut RigidBody, f32) + Sync,
) {
    for a in bodies.iter_mut() {
        a.old_pos = a.pos;
    }

    for _ in 0..num_rebal {
        let mut k = bbox_helper::create_bbox_mut(bodies, |a| {
            a.create_loose(ball_size + push_rate * (num_query as f32))
        });

        let k2: &mut [BBoxMut<F32n, RigidBody>] = unsafe { &mut *(&mut k as &mut [_] as *mut [_]) };

        let mut tree = DinoTree::new(&mut k);

        for _ in 0..num_query {
            tree.find_collisions_mut_par(|mut a, mut b| {
                match a.inner_mut().push_away(b.inner_mut(), ball_size, push_rate) {
                    Some(dis) => {
                        func(a.inner_mut(), b.inner_mut(), dis);
                    }
                    _ => {}
                }
            });

            tree.for_all_not_in_rect_mut(dim, |mut a| {
                a.inner_mut().push_away_from_border(dim.as_ref(), push_rate)
            });

            //for mut body in tree.get_aabb_bots_mut().iter_mut(){
            for body in k2.iter_mut() {
                let body = body.inner_mut();
                let mm = body.push_vec.magnitude();
                if mm > 0.0000001 {
                    if mm > push_rate {
                        body.push_vec=body.push_vec.normalize_to(push_rate);
                    }
                    body.pos += body.push_vec;
                    body.push_vec = vec2same(0.0);
                }
            }
        }
    }

    for a in bodies.iter_mut() {
        let mut diff = a.pos - a.old_pos;

        if diff.magnitude() > 0.2 {
            diff = diff.normalize_to(0.2);
        }

        a.pos = a.old_pos + diff;
    }
}

pub fn make_demo(dim: Rect<F32n>) -> Demo {
    let mut bots: Vec<_> = UniformRandGen::new(dim.inner_into())
        .take(1000)
        .map(|pos| RigidBody::new(pos))
        .collect();

    bots[0].vel = vec2(1., 1.);
    let radius = 6.0;

    Demo::new(move |cursor, canvas, _check_naive| {
        handle_rigid_body(&dim, &mut bots, radius, radius * 0.2, 2, 4, |a, b, _dis| {
            a.handle_collision(b);
        });

        let mut k = bbox_helper::create_bbox_mut(&mut bots, |bot| bot.create_loose(radius));

        let mut tree = DinoTree::new(&mut k);

        tree.for_all_in_rect_mut(
            &axgeom::Rect::from_point(cursor, vec2same(100.0 + radius).inner_try_into().unwrap()),
            |mut b| {
                let diff = cursor.inner_into() - b.inner().pos;

                let dis = diff.magnitude();
                if dis < 60.0 {
                    b.inner_mut().acc -= diff * 0.05;
                }
            },
        );

        for b in bots.iter_mut() {
            //b.acc+=vec2(0.0,0.01);
            b.vel += b.acc;

            b.pos += b.vel;
            b.acc = vec2same(0.0);

            duckduckgeo::collide_with_border(b, dim.as_ref(), 0.5);
        }

        let mut circles = canvas.circles(radius);
        for bot in bots.iter() {
            circles.add(bot.pos);
        }
        circles.send_and_draw([0.7, 0.7, 0.7, 0.5]);
    })
}
