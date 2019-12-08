use crate::support::prelude::*;

#[derive(Copy, Clone, Debug)]
struct Bot {
    id: usize,
    radius: Vec2<i32>,
    pos: Vec2<i32>,
}

impl analyze::HasId for Bot {
    fn get_id(&self) -> usize {
        self.id
    }
}

pub struct MultiRectDemo {
    //tree: DinoTreeOwned<DefaultA, BBox<i32,Bot>>,
    tree: DinoTreeOwnedBBoxPtr<DefaultA,i32,Bot>,
    dim: Rect<i32>,
}
impl MultiRectDemo {
    pub fn new(dim: Rect<F32n>) -> MultiRectDemo {
        let bots: Vec<_> = UniformRandGen::new(dim.inner_into())
            .with_radius(5.0, 20.0)
            .take(200)
            .enumerate()
            .map(|(id, (pos, radius))| {
                let pos: Vec2<f32> = pos;
                let pos = pos.inner_as::<i32>();
                let radius = radius.inner_as();
                let b=Bot { pos, radius, id };
                //let r= Rect::from_point(b.pos, b.radius);
                //bbox(r,b)
                b
            })
            .collect();

        let tree = DinoTreeOwnedBBoxPtr::new_par( bots,|b|{
            Rect::from_point(b.pos, b.radius)
        });

        MultiRectDemo {
            dim: dim.inner_into::<f32>().inner_as(),
            tree,
        }
    }
}

impl DemoSys for MultiRectDemo {
    fn step(
        &mut self,
        cursor: Vec2<F32n>,
        c: &piston_window::Context,
        g: &mut piston_window::G2d,
        check_naive: bool,
    ) {
        for bot in self.tree.as_owned().get_bots().iter() {
            draw_rect_i32([0.0, 0.0, 0.0, 0.3], &bot.get(), c, g);
        }

        let cc: Vec2<i32> = cursor.inner_into::<f32>().inner_as();
        let r1 = axgeom::Rect::new(cc.x - 100, cc.x + 100, cc.y - 100, cc.y + 100);
        let r2 = axgeom::Rect::new(100, 400, 100, 400);

        if check_naive {
            self.tree.as_owned_mut().get_bots_mut(|bots|{
                let mut na=analyze::NaiveAlgs::new(bots);
                na.assert_for_all_in_rect_mut(&r1);
                na.assert_for_all_in_rect_mut(&r2);
                na.assert_for_all_intersect_rect_mut(&r1);
                na.assert_for_all_intersect_rect_mut(&r2);
                na.assert_for_all_not_in_rect_mut(&r1);
            });
            
        }

        //test MultiRect
        {
            let mut rects = self.tree.as_owned_mut().as_tree_mut().multi_rect();

            let mut to_draw = Vec::new();
            let _ = rects.for_all_in_rect_mut(r1, |a| to_draw.push(a));

            let res = rects.for_all_in_rect_mut(r2, |a| {
                to_draw.push(a);
            });

            match res {
                Ok(()) => {
                    draw_rect_i32([0.0, 0.0, 0.0, 0.3], &r1, c, g);
                    draw_rect_i32([0.0, 0.0, 0.0, 0.3], &r2, c, g);

                    for r in to_draw.iter() {
                        draw_rect_i32([1.0, 0.0, 0.0, 0.3], r.get(), c, g);
                    }
                }
                Err(_) => {
                    draw_rect_i32([1.0, 0.0, 0.0, 0.3], &r1, c, g);
                    draw_rect_i32([1.0, 0.0, 0.0, 0.3], &r2, c, g);
                }
            }
        }

        //test for_all_intersect_rect
        self.tree.as_owned().as_tree().for_all_intersect_rect(&r1, |a| {
            draw_rect_i32([0.0, 0.0, 1.0, 0.3], a.get(), c, g);
        });

        //test for_all_not_in_rect_mut
        let mut r1 = self.dim.clone();
        r1.grow(-40);
        draw_rect_i32([1.0, 0.0, 0.0, 0.2], &r1, c, g);
        
        self.tree.as_owned_mut().as_tree_mut().for_all_not_in_rect_mut( &r1, |b| {
            draw_rect_i32([1.0, 0.0, 1.0, 0.5], b.get(), c, g);
        });
    }
}
