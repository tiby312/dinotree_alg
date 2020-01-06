use crate::support::prelude::*;
use duckduckgeo;

#[derive(Copy, Clone)]
pub struct Bot {
    pos: Vec2<f32>,
    vel: Vec2<f32>,
    force: Vec2<f32>,
    wall_move: [Option<(F32n, f32)>; 2],
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

#[derive(Copy, Clone)]
struct Wall(axgeom::Rect<F32n>);

pub fn make_demo(dim: Rect<F32n>,canvas:&mut SimpleCanvas) -> Demo {
    let radius = 5.0;
    let mut bots: Vec<_> = UniformRandGen::new(dim.inner_into())
        .take(4000)
        .map(|pos| Bot {
            pos,
            vel: vec2same(0.0),
            force: vec2same(0.0),
            wall_move: [None; 2],
        })
        .collect();

    let mut walls: Vec<_> = UniformRandGen::new(dim.inner_into())
        .with_radius(10.0, 60.0)
        .take(40)
        .map(|(pos, radius)| Wall(Rect::from_point(pos, radius).inner_try_into().unwrap()))
        .collect();

    let mut rects = canvas.rects();
    for wall in walls.iter() {
        rects.add(wall.0.inner_into());
    }
    let rect_save=rects.save();



    Demo::new(move |cursor, canvas, _check_naive| {
        for b in bots.iter_mut() {
            b.update();

            if let Some((pos, vel)) = b.wall_move[0] {
                b.pos.x = pos.into_inner();
                b.vel.x = vel;
            }

            if let Some((pos, vel)) = b.wall_move[1] {
                b.pos.y = pos.into_inner();
                b.vel.y = vel;
            }

            b.wall_move[0] = None;
            b.wall_move[1] = None;

            duckduckgeo::wrap_position(&mut b.pos, dim.inner_into());
        }
        bots[0].pos = cursor.inner_into();

        let mut k = bbox_helper::create_bbox_mut(&mut bots, |b| {
            Rect::from_point(b.pos, vec2same(radius))
                .inner_try_into()
                .unwrap()
        });

        {
            let mut walls = bbox_helper::create_bbox_mut(&mut walls, |wall| wall.0);
            let mut tree = DinoTree::new_par(&mut k);

            tree.intersect_with_mut(&mut walls, |mut bot, wall| {
                let fric = 0.8;

                let wallx = &wall.get().x;
                let wally = &wall.get().y;
                let vel = bot.inner().vel;

                let ret = match duckduckgeo::collide_with_rect::<f32>(
                    bot.get().as_ref(),
                    wall.get().as_ref(),
                )
                .unwrap()
                {
                    duckduckgeo::WallSide::Above => {
                        [None, Some((wally.start - radius, -vel.y * fric))]
                    }
                    duckduckgeo::WallSide::Below => {
                        [None, Some((wally.end + radius, -vel.y * fric))]
                    }
                    duckduckgeo::WallSide::LeftOf => {
                        [Some((wallx.start - radius, -vel.x * fric)), None]
                    }
                    duckduckgeo::WallSide::RightOf => {
                        [Some((wallx.end + radius, -vel.x * fric)), None]
                    }
                };
                bot.inner_mut().wall_move = ret;
            });

            let cc = cursor.inner_into();
            tree.for_all_in_rect_mut(
                &axgeom::Rect::from_point(cc, vec2same(100.0))
                    .inner_try_into()
                    .unwrap(),
                |mut b| {
                    let _ = duckduckgeo::repel_one(b.inner_mut(), cc, 0.001, 20.0);
                },
            );

            tree.find_collisions_mut_par(|mut a, mut b| {
                let _ = duckduckgeo::repel(a.inner_mut(), b.inner_mut(), 0.001, 2.0);
            });
        }

        
        rect_save.draw(canvas,[0.7,0.7,0.7,0.3]);

        let mut circles = canvas.circles();
        for bot in k.iter() {
            circles.add(bot.inner().pos);
        }
        circles.send_and_draw([1.0, 0.0, 0.5, 0.3],radius);
    })
}
/*
pub struct IntersectWithDemo {
    radius: f32,
    bots: Vec<Bot>,
    walls: Vec<Wall>,
    dim: Rect<F32n>,
}
impl IntersectWithDemo {
    pub fn new(dim: Rect<F32n>) -> IntersectWithDemo {

        IntersectWithDemo {
            radius: 5.0,
            bots,
            walls,
            dim,
        }
    }
}

impl DemoSys for IntersectWithDemo {
    fn step(
        &mut self,
        cursor: Vec2<F32n>,
        sys: &mut MySys,
        _check_naive: bool,
    ) {
    }
}
*/
