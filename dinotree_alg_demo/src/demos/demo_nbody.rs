use crate::support::prelude::*;
use dinotree_alg;
use duckduckgeo;
use duckduckgeo::GravityTrait;

#[derive(Copy, Clone)]
struct NodeMass {
    rect: axgeom::Rect<F32n>,
    center: Vec2<f32>,
    mass: f32,
    force: Vec2<f32>,
}

impl duckduckgeo::GravityTrait for NodeMass {
    type N = f32;
    fn pos(&self) -> Vec2<f32> {
        self.center
    }
    fn mass(&self) -> f32 {
        self.mass
    }
    fn apply_force(&mut self, a: Vec2<f32>) {
        self.force += a;
    }
}

use core::marker::PhantomData;

#[derive(Clone, Copy)]
struct Bla<'a> {
    num_pairs_checked: usize,
    _p: PhantomData<&'a usize>,
}
impl<'b> NodeMassTrait for Bla<'b> {
    type No = NodeMass;
    type Item = BBoxMut<'b, F32n, Bot>;
    type Num = F32n;

    fn get_rect(a: &Self::No) -> &axgeom::Rect<F32n> {
        &a.rect
    }

    //gravitate this nodemass with another node mass
    fn handle_node_with_node(&self, a: &mut Self::No, b: &mut Self::No) {
        let _ = duckduckgeo::gravitate(a, b, 0.0001, 0.004);
    }

    //gravitate a bot with a bot
    fn handle_bot_with_bot(
        &self,
        mut a: PMut<Self::Item>,
        mut b: PMut<Self::Item>,
    ) {
        //self.num_pairs_checked+=1;
        let _ = duckduckgeo::gravitate(a.inner_mut(), b.inner_mut(), 0.0001, 0.004);
    }

    //gravitate a nodemass with a bot
    fn handle_node_with_bot(&self, a: &mut Self::No, mut b: PMut<Self::Item>) {
        let _ = duckduckgeo::gravitate(a, b.inner_mut(), 0.0001, 0.004);
    }

    fn new<'a, I: Iterator<Item = &'a Self::Item>>(
        &'a self,
        it: I,
        rect: axgeom::Rect<F32n>,
    ) -> Self::No {
        let mut total_x = 0.0;
        let mut total_y = 0.0;
        let mut total_mass = 0.0;

        for i in it {
            let m = i.inner().mass();
            total_mass += m;
            total_x += m * i.inner().pos.x;
            total_y += m * i.inner().pos.y;
        }

        let center = if total_mass != 0.0 {
            vec2(total_x / total_mass, total_y / total_mass)
        } else {
            vec2same(0.0)
        };
        NodeMass {
            center,
            mass: total_mass,
            force: vec2same(0.0),
            rect,
        }
    }

    fn apply_to_bots<'a, I: Iterator<Item = PMut<'a, Self::Item>>>(
        &'a self,
        a: &'a Self::No,
        it: I,
    ) {
        if a.mass > 0.000_000_1 {
            let total_forcex = a.force.x;
            let total_forcey = a.force.y;

            for mut i in it {
                let forcex = total_forcex * (i.inner().mass / a.mass);
                let forcey = total_forcey * (i.inner().mass / a.mass);
                i.as_mut().inner_mut().apply_force(vec2(forcex, forcey));
            }
        }
    }

    fn is_far_enough(&self, b: [F32n; 2]) -> bool {
        (b[0].into_inner() - b[1].into_inner()).abs() > 200.0
    }

    fn is_far_enough_half(&self, b: [F32n; 2]) -> bool {
        (b[0].into_inner() - b[1].into_inner()).abs() > 100.0
    }
}

#[derive(Copy, Clone)]
pub struct Bot {
    pos: Vec2<f32>,
    vel: Vec2<f32>,
    force: Vec2<f32>,
    mass: f32,
}
impl Bot {
    fn handle(&mut self) {
        let b = self;

        b.pos += b.vel;

        //F=MA
        //A=F/M
        let acc = b.force / b.mass;

        b.vel += acc;

        b.force = vec2same(0.0);
    }
    fn create_aabb(&self) -> axgeom::Rect<F32n> {
        let r = 5.0f32.min(self.mass.sqrt() / 10.0);
        axgeom::Rect::from_point(self.pos, vec2same(r))
            .inner_try_into()
            .unwrap()
    }
}
impl duckduckgeo::GravityTrait for Bot {
    type N = f32;
    fn pos(&self) -> Vec2<f32> {
        self.pos
    }
    fn mass(&self) -> f32 {
        self.mass
    }
    fn apply_force(&mut self, a: Vec2<f32>) {
        self.force += a;
    }
}

pub struct DemoNbody {
    dim: Rect<F32n>,
    bots: Vec<Bot>,
    no_mass_bots: Vec<Bot>,
    max_percentage_error: f32,
}
impl DemoNbody {
    pub fn new(dim: Rect<F32n>) -> DemoNbody {
        let mut bots: Vec<_> = UniformRandGen::new(dim.inner_into())
            .take(4000)
            .map(|pos| Bot {
                mass: 100.0,
                pos,
                vel: vec2same(0.0),
                force: vec2same(0.0),
            })
            .collect();

        //Make one of the bots have a lot of mass.
        bots.last_mut().unwrap().mass = 10000.0;

        let no_mass_bots: Vec<Bot> = Vec::new();

        DemoNbody {
            dim,
            bots,
            no_mass_bots,
            max_percentage_error: 0.0,
        }
    }
}

impl DemoSys for DemoNbody {
    fn step(
        &mut self,
        cursor: Vec2<F32n>,
        c: &piston_window::Context,
        g: &mut piston_window::G2d,
        check_naive: bool,
    ) {
        let no_mass_bots = &mut self.no_mass_bots;
        let bots = &mut self.bots;

        let mut bots2: Vec<BBox<F32n, Bot>> = bots
            .iter()
            .map(|bot| BBox::new(bot.create_aabb(), *bot))
            .collect();
        let mut bots3 = bots.clone();

        let mut k = bbox_helper::create_bbox_mut(bots, |b| b.create_aabb());

        {
            let mut tree = DinoTree::new_par( &mut k);
           

            let border = self.dim;

            if !check_naive {
                tree.nbody_mut(
                    &mut Bla {
                        num_pairs_checked: 0,
                        _p: PhantomData,
                    },
                    border,
                );
            } else {
                let mut bla = Bla {
                    num_pairs_checked: 0,
                    _p: PhantomData,
                };
                tree.nbody_mut(&mut bla, border);
                let num_pair_alg = bla.num_pairs_checked;

                
                let (bots2, num_pair_naive) = {
                    let mut num_pairs_checked = 0;
                    /*
                    nbody::naive_mut(&mut bots2, |mut a, mut b| {
                        let _ =
                            duckduckgeo::gravitate(a.inner_mut(), b.inner_mut(), 0.00001, 0.004);
                        num_pairs_checked += 1;
                    });
                    */
                    unimplemented!();
                    (bots2, num_pairs_checked)
                    
                    
                };


                for b in bots3.iter_mut() {
                    b.force = vec2same(0.0);
                }

                {
                    let mut max_diff = None;

                    for (a, bb) in bots3.iter().zip(bots2.iter()) {
                        let b = &bb.inner;

                        let dis_sqr1 = a.force.magnitude2();
                        let dis_sqr2 = b.force.magnitude2();
                        let dis1 = dis_sqr1.sqrt();
                        let dis2 = dis_sqr2.sqrt();

                        let acc_dis1 = dis1 / a.mass;
                        let acc_dis2 = dis2 / a.mass;

                        let diff = (acc_dis1 - acc_dis2).abs();

                        let error: f32 = (acc_dis2 - acc_dis1).abs() / acc_dis2;

                        match max_diff {
                            None => max_diff = Some((diff, bb, error)),
                            Some(max) => {
                                if diff > max.0 {
                                    max_diff = Some((diff, bb, error))
                                }
                            }
                        }
                    }
                    let max_diff = max_diff.unwrap();
                    self.max_percentage_error = max_diff.2 * 100.0;

                    let f = {
                        let a: f32 = num_pair_alg as f32;
                        let b: f32 = num_pair_naive as f32;
                        a / b
                    };

                    println!("absolute acceleration err={:06.5} percentage err={:06.2}% current bot not checked ratio={:05.2}%",max_diff.0,self.max_percentage_error,f*100.0);

                    draw_rect_f32([1.0, 0.0, 1.0, 1.0], max_diff.1.get().as_ref(), c, g);
                }
            }

            tree.find_collisions_mut_par(|mut a, mut b| {
                let (a, b) = if a.inner().mass > b.inner().mass {
                    (a.inner_mut(), b.inner_mut())
                } else {
                    (b.inner_mut(), a.inner_mut())
                };

                if b.mass != 0.0 {
                    let ma = a.mass;
                    let mb = b.mass;
                    let ua = a.vel;
                    let ub = b.vel;

                    //Do perfectly inelastic collision.
                    let vx = (ma * ua.x + mb * ub.x) / (ma + mb);
                    let vy = (ma * ua.y + mb * ub.y) / (ma + mb);
                    assert!(!vx.is_nan() && !vy.is_nan());
                    a.mass += b.mass;

                    a.force += b.force;
                    a.vel = vec2(vx, vy);

                    b.mass = 0.0;
                    b.force = vec2same(0.0);
                    b.vel = vec2same(0.0);
                    b.pos = vec2same(0.0);
                }
            });

            if check_naive {
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

                        let arr = if axis.is_xaxis() {
                            [
                                div as f64,
                                length[0].into_inner() as f64,
                                div as f64,
                                length[1].into_inner() as f64,
                            ]
                        } else {
                            [
                                length[0].into_inner() as f64,
                                div as f64,
                                length[1].into_inner() as f64,
                                div as f64,
                            ]
                        };

                        let radius = (1isize.max(5 - depth as isize)) as f32;

                        line(
                            [0.0, 0.0, 0.0, 0.5], // black
                            radius as f64,        // radius of line
                            arr,                  // [x0, y0, x1,y1] coordinates of line
                            self.c.transform,
                            self.g,
                        );

                        let [x1, y1, w1, w2] = if axis.is_xaxis() {
                            [cont[0], length[0], cont[1] - cont[0], length[1] - length[0]]
                        } else {
                            [length[0], cont[0], length[1] - length[0], cont[1] - cont[0]]
                        };

                        let square = [
                            x1.into_inner() as f64,
                            y1.into_inner() as f64,
                            w1.into_inner() as f64,
                            w2.into_inner() as f64,
                        ];
                        rectangle([0.0, 1.0, 1.0, 0.2], square, self.c.transform, self.g);
                    }
                }

                let mut dd = Bla { c: &c, g };
                tree.draw(&mut dd, &border);
            }
        }
        //Draw bots.
        for bot in k.iter() {
            draw_rect_f32([0.0, 0.5, 0.0, 1.0], bot.rect.as_ref(), c, g);
        }

        {
            let mut new_bots = Vec::new();
            for b in bots.drain(..) {
                if b.mass == 0.0 {
                    no_mass_bots.push(b);
                } else {
                    new_bots.push(b);
                }
            }
            bots.append(&mut new_bots);
        };

        //Update bot locations.
        for bot in bots.iter_mut() {
            Bot::handle(bot);
            duckduckgeo::wrap_position(&mut bot.pos, *self.dim.as_ref());
        }

        if let Some(mut b) = no_mass_bots.pop() {
            b.mass = 30.0;
            b.pos = cursor.inner_into();
            b.force = vec2same(0.0);
            b.vel = vec2(1.0, 0.0);
            bots.push(b);
        }
    }
}
