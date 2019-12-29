use crate::support::prelude::*;

use duckduckgeo;

#[derive(Copy, Clone)]
pub struct Bot {
    id: usize, //id used to verify pairs against naive
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

pub fn make_demo(dim: Rect<F32n>) -> Demo {
    let num_bot = 4000;

    let radius = 5.0;

    let mut bots: Vec<_> = UniformRandGen::new(dim.inner_into())
        .take(num_bot)
        .enumerate()
        .map(|(id, pos)| Bot {
            id,
            pos,
            vel: vec2same(0.0),
            force: vec2same(0.0),
        })
        .collect();

    Demo::new(move |cursor, sys, check_naive| {
        for b in bots.iter_mut() {
            b.update();
        }

        let mut k = bbox_helper::create_bbox_mut(&mut bots, |b| {
            Rect::from_point(b.pos, vec2same(radius))
                .inner_try_into()
                .unwrap()
        });
        let mut tree = DinoTree::new_par(&mut k);

        {
            let dim2 = dim.inner_into();
            tree.for_all_not_in_rect_mut(&dim, |mut a| {
                duckduckgeo::collide_with_border(a.inner_mut(), &dim2, 0.5);
            });
        }

        let vv = vec2same(100.0).inner_try_into().unwrap();
        let cc = cursor.inner_into();
        tree.for_all_in_rect_mut(&axgeom::Rect::from_point(cursor, vv), |mut b| {
            let _ = duckduckgeo::repel_one(b.inner_mut(), cc, 0.001, 20.0);
        });

        {
            let rects = sys.rects([0.0, 1.0, 1.0, 0.6]);
            let mut dd = Bla { rects };
            tree.draw(&mut dd, &dim);
            dd.rects.send_and_draw();
        }

        //draw lines to the bots.
        {
            let mut lines = sys.lines([1.0, 0.5, 1.0, 0.6], 2.0);
            draw_bot_lines(tree.axis(), tree.vistr(), &dim, &mut lines);
            lines.send_and_draw();
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

        let mut circles = sys.circles([1.0, 1.0, 0.0, 0.6], radius);
        for bot in bots.iter() {
            circles.add(bot.pos); //TODO we're not testing that the bots were draw in the right order
        }
        circles.send_and_draw();
    })
}

struct Bla<'a> {
    rects: very_simple_2d::very_simple_2d_core::RectSession<'a>,
}
impl<'a> DividerDrawer for Bla<'a> {
    type N = F32n;
    fn draw_divider<A: axgeom::Axis>(
        &mut self,
        axis: A,
        div: F32n,
        cont: [F32n; 2],
        length: [F32n; 2],
        _depth: usize,
    ) {
        let _div = div.into_inner();

        /*
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
        */
        let cont = Range::new(cont[0], cont[1]).inner_into();
        let length = Range::new(length[0], length[1]).inner_into();

        //let radius = (1isize.max(5 - depth as isize)) as f64;

        let rect = if axis.is_xaxis() {
            Rect { x: cont, y: length }
        } else {
            Rect { x: length, y: cont }
        };

        self.rects.add(rect);

        //rectangle([0.0, 1.0, 1.0, 0.2], square, self.c.transform, self.g);
    }
}

fn draw_bot_lines<A: axgeom::Axis>(
    axis: A,
    stuff: Vistr<NodeMut<BBoxMut<F32n, Bot>>>,
    rect: &axgeom::Rect<F32n>,
    lines: &mut very_simple_2d::very_simple_2d_core::LineSession,
) {
    use compt::Visitor;
    let (nn, rest) = stuff.next();
    let nn = nn.get();
    let mid = match rest {
        Some([start, end]) => match nn.div {
            Some(div) => {
                let (a, b) = rect.subdivide(axis, *div);

                draw_bot_lines(axis.next(), start, &a, lines);
                draw_bot_lines(axis.next(), end, &b, lines);

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
            let _bx = b.inner.pos.x;
            let _by = b.inner.pos.y;

            lines.add(b.inner.pos, vec2(midx, midy));

            counter += color_delta;
        }
    }
}
