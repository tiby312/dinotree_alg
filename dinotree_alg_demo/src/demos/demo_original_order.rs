use crate::support::prelude::*;
use dinotree_alg;
use duckduckgeo;

#[derive(Copy, Clone)]
pub struct Bot {
    _id: usize, //id used to verify pairs against naive
    pos: Vec2<f32>,
    vel: Vec2<f32>,
    force: Vec2<f32>,
}

impl duckduckgeo::BorderCollideTrait for Bot {
    type N = f32;
    fn pos_vel_mut(&mut self) -> (&mut Vec2<f32>, &mut Vec2<f32>) {
        (&mut self.pos, &mut self.vel)
    }
}

impl duckduckgeo::RepelTrait for Bot {
    type N = f32;
    fn pos(&self) -> Vec2<f32> {
        self.pos
    }
    fn add_force(&mut self, force: Vec2<f32>) {
        self.force += force;
    }
}

impl Bot {
    fn update(&mut self) {
        self.vel += self.force;
        //non linear drag
        self.vel *= 0.9;
        self.pos += self.vel;
        self.force = vec2same(0.0);
    }
}

pub struct OrigOrderDemo {
    radius: f32,
    bots: Vec<Bot>,
    colors: Vec<[u8; 3]>,
    dim: Rect<F32n>,
}
impl OrigOrderDemo {
    pub fn new(dim: Rect<F32n>) -> OrigOrderDemo {
        let num_bot = 4000;

        let radius = 5.0;

        let bots: Vec<_> = UniformRandGen::new(dim.inner_into())
            .take(num_bot)
            .enumerate()
            .map(|(id, pos)| Bot {
                _id: id,
                pos,
                vel: vec2same(0.0),
                force: vec2same(0.0),
            })
            .collect();

        let colors = ColorGenerator::new().take(num_bot).collect();
        OrigOrderDemo {
            radius,
            bots,
            colors,
            dim,
        }
    }
}

impl DemoSys for OrigOrderDemo {
    fn step(
        &mut self,
        cursor: Vec2<F32n>,
        c: &piston_window::Context,
        g: &mut piston_window::G2d,
        check_naive: bool,
    ) {
        let radius = self.radius;

        for b in self.bots.iter_mut() {
            b.update();
        }

        let mut k = build_helper::create_bbox_mut(&mut self.bots, |b| {
            Rect::from_point(b.pos, vec2same(radius))
                .inner_try_into()
                .unwrap()
        });
        let mut tree = DinoTree::new_par(axgeom::XAXISS, &mut k);

        {
            let dim2 = self.dim.inner_into();
            tree.for_all_not_in_rect_mut( &self.dim, |mut a| {
                duckduckgeo::collide_with_border(a.inner_mut(), &dim2, 0.5);
            });
        }

        let vv = vec2same(100.0).inner_try_into().unwrap();
        let cc = cursor.inner_into();
        tree.for_all_in_rect_mut( &axgeom::Rect::from_point(cursor, vv), |mut b| {
            let _ = duckduckgeo::repel_one(b.inner_mut(), cc, 0.001, 20.0);
        });

        {
            let mut dd = Bla { c: &c, g };
            tree.draw(&mut dd, &self.dim);
        }

        //draw lines to the bots.
        {
            draw_bot_lines(tree.axis(), tree.vistr(), &self.dim, c, g);
        }

        if !check_naive {
            tree.find_collisions_mut_par(|mut a, mut b| {
                let _ = duckduckgeo::repel(a.inner_mut(), b.inner_mut(), 0.001, 2.0);
            });
        } else {
            /*
            let mut res=Vec::new();
            colfind::QueryBuilder::new(&mut tree).query_seq(|mut a,mut b| {
                let a=a.inner_mut();
                let b=b.inner_mut();
                let _ = duckduckgeo::repel(a,b,0.001,2.0);
                let (a,b)=if a.id<b.id{
                    (a,b)
                }else{
                    (b,a)
                };
                res.push((a.id,b.id));
            });



            let mut res2=Vec::new();

            colfind::query_naive_mut(tree.get_bots_mut(),|mut a,mut b|{
                let a=a.inner_mut();
                let b=b.inner_mut();
                let (a,b)=if a.id<b.id{
                    (a,b)
                }else{
                    (b,a)
                };
                res2.push((a.id,b.id))
            });

            let cmp=|a:&(usize,usize),b:&(usize,usize)|{
                use std::cmp::Ordering;

                match a.0.cmp(&b.0){
                    Ordering::Less=>{
                        Ordering::Less
                    },
                    Ordering::Greater=>{
                        Ordering::Greater
                    },
                    Ordering::Equal=>{
                        match a.1.cmp(&b.1){
                            Ordering::Less=>{
                                Ordering::Less
                            },
                            Ordering::Greater=>{
                                Ordering::Greater
                            },
                            Ordering::Equal=>{
                                Ordering::Equal
                            }
                        }
                    }
                }
            };

            res.sort_by(cmp);
            res2.sort_by(cmp);
            println!("lens={:?}",(res.len(),res2.len()));
            assert_eq!(res.len(),res2.len());
            for (a,b) in res.iter().zip(res2.iter()){
                assert_eq!(a,b)
            }
            */
            unimplemented!()
        }

        fn conv(a: u8) -> f32 {
            let a: f32 = a as f32;
            a / 256.0
        }

        for (bot, cols) in self.bots.iter().zip(self.colors.iter()) {
            let rect = &axgeom::Rect::from_point(bot.pos, vec2(radius, radius));
            draw_rect_f32(
                [conv(cols[0]), conv(cols[1]), conv(cols[2]), 0.6],
                rect,
                c,
                g,
            );
        }
    }
}

struct Bla<'a, 'b: 'a> {
    c: &'a Context,
    g: &'a mut G2d<'b>,
}
impl<'a, 'b: 'a> DividerDrawer for Bla<'a, 'b> {
    type N = F32n;
    fn draw_divider<A: axgeom::AxisTrait>(
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

        let square = [
            x1.into_inner() as f64,
            y1.into_inner() as f64,
            w1.into_inner() as f64,
            w2.into_inner() as f64,
        ];
        rectangle([0.0, 1.0, 1.0, 0.2], square, self.c.transform, self.g);
    }
}
fn draw_bot_lines<A: axgeom::AxisTrait>(
    axis: A,
    stuff: Vistr<NodeMut<BBoxMut<F32n, Bot>>>,
    rect: &axgeom::Rect<F32n>,
    c: &Context,
    g: &mut G2d,
) {
    use compt::Visitor;
    let (nn, rest) = stuff.next();
    let nn = nn.get();
    let mid = match rest {
        Some([start, end]) => match nn.div {
            Some(div) => {
                let (a, b) = rect.subdivide(axis, *div);

                draw_bot_lines(axis.next(), start, &a, c, g);
                draw_bot_lines(axis.next(), end, &b, c, g);

                let ((x1, x2), (y1, y2)) = rect.inner_into::<f32>().get();
                let midx = if !axis.is_xaxis() {
                    x1 + (x2 - x1) / 2.0
                } else {
                    div.into_inner()
                };

                let midy = if axis.is_xaxis() {
                    y1 + (y2 - y1) / 2.0
                } else {
                    div.into_inner()
                };

                Some((midx, midy))
            }
            None => None,
        },
        None => {
            let ((x1, x2), (y1, y2)) = rect.inner_into::<f32>().get();
            let midx = x1 + (x2 - x1) / 2.0;

            let midy = y1 + (y2 - y1) / 2.0;

            Some((midx, midy))
        }
    };

    if let Some((midx, midy)) = mid {
        let color_delta = 1.0 / nn.bots.len() as f32;
        let mut counter = 0.0;
        for b in nn.bots.iter() {
            let bx = b.inner.pos.x;
            let by = b.inner.pos.y;

            line(
                [counter, 0.2, 0.0, 0.3],                         // black
                2.0,                                              // radius of line
                [midx as f64, midy as f64, bx as f64, by as f64], // [x0, y0, x1,y1] coordinates of line
                c.transform,
                g,
            );

            counter += color_delta;
        }
    }
}
