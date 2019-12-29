use crate::support::prelude::*;
use bit_vec::BitVec;
use duckduckgeo::collide_with_rect;
use duckduckgeo::WallSide;

#[derive(Debug)]
struct GridDim2D {
    dim: Rect<F32n>,
    xs: usize,
    ys: usize,
    inner: BitVec,
}

impl GridDim2D {
    fn new(xs: usize, ys: usize, dim: Rect<F32n>) -> GridDim2D {
        let inner = BitVec::from_elem(xs * ys, false);

        GridDim2D { xs, ys, inner, dim }
    }
    pub fn xdim(&self) -> usize {
        self.xs
    }
    pub fn ydim(&self) -> usize {
        self.ys
    }
    pub fn get(&self, x: usize, y: usize) -> bool {
        self.inner[x * self.ys + y]
    }
    pub fn set(&mut self, x: usize, y: usize) {
        self.inner.set(x * self.ys + y, true)
    }

    pub fn get_rect(&self, i: usize, j: usize) -> Rect<f32> {
        let dim = self.dim.as_ref();
        let xdim = self.xs;
        let ydim = self.ys;
        let xratio = i as f32 / xdim as f32;
        let yratio = j as f32 / ydim as f32;
        let width = dim.x.end - dim.x.start;
        let height = dim.y.end - dim.y.start;

        let xratio2 = (i as f32 + 1.0) / xdim as f32;
        let yratio2 = (j as f32 + 1.0) / ydim as f32;

        Rect::new(
            dim.x.start + xratio * width,
            dim.x.start + xratio2 * width,
            dim.y.start + yratio * height,
            dim.y.start + yratio2 * height,
        )
    }
    fn detect_collision(&self, bot: &Bot, radius: f32) -> Option<Rect<f32>> {
        if bot.vel.magnitude2() < 0.01 * 0.01 {
            return None;
        }

        let xdim = self.xs;
        let ydim = self.ys;

        let dim: &Rect<f32> = self.dim.as_ref();

        //https://math.stackexchange.com/questions/528501/how-to-determine-which-cell-in-a-grid-a-point-belongs-to
        let jj = bot.vel.normalize_to(radius);

        let x = bot.pos.x + jj.x;
        let y = bot.pos.y + jj.y;
        let width = dim.x.end - dim.x.start;
        let height = dim.y.end - dim.y.start;

        let i = (x * (xdim as f32 / width))
            .floor()
            .max(0.0)
            .min((xdim - 1) as f32);
        let j = (y * (ydim as f32 / height))
            .floor()
            .max(0.0)
            .min((ydim - 1) as f32);
        let i = i as usize;
        let j = j as usize;

        if self.get(i, j) {
            //This bot is inside of this thing yo
            Some(self.get_rect(i, j))
        } else {
            None
        }
    }
}

#[derive(Copy, Clone, Debug)]
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


pub fn make_demo(dim:Rect<F32n>)->Demo{
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

    let colors:Vec<_> = ColorGenerator::new().take(num_bot).collect();

    let mut grid = GridDim2D::new(20, 20, dim);

    for a in 0..20 {
        grid.set(a, a);
    }
    for a in 0..20 {
        grid.set(a, 5);
    }


    Demo::new(move |cursor,sys,_check_naive|{
        
        for b in bots.iter_mut() {
            b.update();
        }

        let mut k = bbox_helper::create_bbox_mut(&mut bots, |b| {
            Rect::from_point(b.pos, vec2same(radius))
                .inner_try_into()
                .unwrap()
        });

        {
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

            tree.find_collisions_mut_par(|mut a, mut b| {
                let _ = duckduckgeo::repel(a.inner_mut(), b.inner_mut(), 0.001, 2.0);
            });

            let mut squares = sys.squares([1.0, 0.5, 1.0, 0.3], radius);
            for i in 0..grid.xdim() {
                for j in 0..grid.ydim() {
                    if grid.get(i, j) {
                        squares.add(vec2(i, j).inner_as());
                    }
                }
            }
            drop(squares)
        }

        fn conv(a: u8) -> f32 {
            let a: f32 = a as f32;
            a / 256.0
        }

        let mut circles = sys.circles([1.0, 0.2, 0.2, 0.6], radius);
        for (bot, cols) in bots.iter_mut().zip(colors.iter()) {
            let rect = &axgeom::Rect::from_point(bot.pos, vec2(radius, radius));

            let cols = [conv(cols[0]), conv(cols[1]), conv(cols[2]), 0.6];

            if let Some(rr) = grid.detect_collision(bot, radius) {
                if let Some(k) = collide_with_rect::<f32>(rect, &rr) {
                    let wallx = rr.x;
                    let wally = rr.y;
                    let fric = 0.5;
                    let vel = bot.vel;
                    let wall_move = match k {
                        WallSide::Above => [None, Some((wally.start - radius, -vel.y * fric))],
                        WallSide::Below => [None, Some((wally.end + radius, -vel.y * fric))],
                        WallSide::LeftOf => [Some((wallx.start - radius, -vel.x * fric)), None],
                        WallSide::RightOf => [Some((wallx.end + radius, -vel.x * fric)), None],
                    };

                    if let Some((pos, vel)) = wall_move[0] {
                        bot.pos.x = pos;
                        bot.vel.x = vel;
                    }

                    if let Some((pos, vel)) = wall_move[1] {
                        bot.pos.y = pos;
                        bot.vel.y = vel;
                    }
                }
            }
            circles.add(bot.pos);
            //draw_rect_f32(cols, rect, c, g);
        }
        circles.send_and_draw();

    })

}

/*
pub struct GridDemo {
    radius: f32,
    bots: Vec<Bot>,
    colors: Vec<[u8; 3]>,
    dim: Rect<F32n>,
    grid: GridDim2D,
}
impl GridDemo {
    pub fn new(dim: Rect<F32n>) -> GridDemo {
    }
}

impl DemoSys for GridDemo {
    fn step(
        &mut self,
        cursor: Vec2<F32n>,
        sys: &mut MySys,
        _check_naive: bool,
    ) {
    }
}
*/