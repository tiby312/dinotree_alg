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


pub fn make_demo(dim:Rect<F32n>)->Demo{
    let mut bots: Vec<_> = UniformRandGen::new(dim.inner_into())
        .with_radius(5.0, 20.0)
        .take(200)
        .enumerate()
        .map(|(id, (pos, radius))| {
            let pos: Vec2<f32> = pos;
            let pos = pos.inner_as::<i32>();
            let radius = radius.inner_as();
            let b = Bot { pos, radius, id };
            //let r= Rect::from_point(b.pos, b.radius);
            //bbox(r,b)
            b
        })
        .collect();

    let mut tree = DinoTreeOwnedBBoxPtr::new_par(bots, |b| Rect::from_point(b.pos, b.radius));

    Demo::new(move |cursor,sys,check_naive|{
        let mut rects = sys.rects([0.0, 1.0, 0.0, 0.2]);
        for bot in tree.as_owned().get_bots().iter() {
            rects.add(bot.get().inner_as());
        }
        rects.send_and_draw();
        drop(rects);

        let cc: Vec2<i32> = cursor.inner_into::<f32>().inner_as();
        let r1 = axgeom::Rect::new(cc.x - 100, cc.x + 100, cc.y - 100, cc.y + 100);
        let r2 = axgeom::Rect::new(100, 400, 100, 400);

        if check_naive {
            tree.as_owned_mut().get_bots_mut(|bots| {
                let mut na = analyze::NaiveAlgs::new(bots);
                na.assert_for_all_in_rect_mut(&r1);
                na.assert_for_all_in_rect_mut(&r2);
                na.assert_for_all_intersect_rect_mut(&r1);
                na.assert_for_all_intersect_rect_mut(&r2);
                na.assert_for_all_not_in_rect_mut(&r1);
            });
        }

        //test MultiRect
        {
            let mut rects = tree.as_owned_mut().as_tree_mut().multi_rect();

            let mut to_draw = Vec::new();
            let _ = rects.for_all_in_rect_mut(r1, |a| to_draw.push(a));

            let res = rects.for_all_in_rect_mut(r2, |a| {
                to_draw.push(a);
            });

            match res {
                Ok(()) => {
                    sys.rects([0.0, 0.0, 0.0, 0.5])
                        .add(r1.inner_as())
                        .add(r2.inner_as())
                        .send_and_draw();

                    let mut rects = sys.rects([0.0, 0.0, 0.0, 0.2]);
                    for r in to_draw.iter() {
                        rects.add(r.get().inner_as());
                    }
                }
                Err(_) => {
                    sys.rects([1.0, 0.0, 0.0, 0.5])
                        .add(r1.inner_as())
                        .add(r2.inner_as())
                        .send_and_draw();
                }
            }
        }

        //test for_all_intersect_rect
        let mut rects = sys.rects([0.0, 0.0, 1.0, 0.2]);
        tree
            .as_owned()
            .as_tree()
            .for_all_intersect_rect(&r1, |a| {
                rects.add(a.get().inner_as());
            });
        rects.send_and_draw();
        drop(rects);

        //test for_all_not_in_rect_mut
        let mut r1 = dim.inner_into::<f32>().inner_as::<i32>().clone();
        r1.grow(-40);

        sys.rects([1.0, 0.0, 0.0, 0.2]).add(r1.inner_as()).send_and_draw();

        let mut rects = sys.rects([1.0, 0.0, 1.0, 0.5]);
        tree
            .as_owned_mut()
            .as_tree_mut()
            .for_all_not_in_rect_mut(&r1, |b| {
                rects.add(b.get().inner_as());
            });
        rects.send_and_draw();

    })
}


