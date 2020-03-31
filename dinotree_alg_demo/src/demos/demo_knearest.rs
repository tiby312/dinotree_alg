use crate::support::prelude::*;


#[derive(Copy, Clone)]
struct Bot {
    id: usize,
    rect:Rect<f32>
}

impl analyze::HasId for Bot {
    fn get_id(&self) -> usize {
        self.id
    }
}

pub fn make_demo(dim: Rect<F32n>,canvas:&mut SimpleCanvas) -> Demo {
    let bots: Vec<_> = UniformRandGen::new(dim.inner_into())
        .with_radius(2.0, 50.0)
        .take(40)
        .enumerate()
        .map(|(id, (pos, radius))| Bot { id,rect:Rect::from_point(pos, radius)})
        .collect();

    let mut tree = DinoTreeOwnedBBoxPtr::new(bots, |bot| {
        bot.rect
            .inner_try_into()
            .unwrap()
    });

    let mut rects = canvas.rects();
    for bot in tree.as_owned().get_bots().iter() {
        rects.add(bot.get().inner_into().into());
    }
    let rect_save=rects.save(canvas);


    Demo::new(move |cursor, canvas, check_naive| {
        struct Kn<'a> {
            _p:core::marker::PhantomData<&'a mut Bot>,
            rects: &'a mut egaku2d::shapes::RectSession
        };
        /*
        impl<'a> Knearest for Kn<'a> {
            type T = BBox<F32n, &'a mut Bot>;
            type N = F32n;

            fn distance_to_bot(&self, point: Vec2<Self::N>, bot: &Self::T) -> Self::N {
                self.rects.borrow_mut().add(bot.get().inner_into().into());
                self.distance_to_rect(point, bot.get())
            }

            fn distance_to_rect(&self, point: Vec2<Self::N>, rect: &Rect<Self::N>) -> Self::N {
                let r:&Rect<f32>=rect.as_ref();
                let dis = r.distance_squared_to_point(point.inner_into());
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
                    }
                };
                f32n(dis)
            }
        }
        */

        let cols = [
            [1.0, 0.0, 0.0, 0.6], //red closest
            [0.0, 1.0, 0.0, 0.6], //green second closest
            [0.0, 0.0, 1.0, 0.6], //blue third closets
        ];

        struct Res {
            rect: Rect<F32n>,
            mag: F32n,
        }

        let mut vv = {
            let mut rects = canvas.rects();
            let mut kn = Kn {
                _p:core::marker::PhantomData,
                rects:&mut rects,
            };

            fn distance_to_rect(rect:&Rect<f32>,point:Vec2<f32>)->f32{
                //let r:&Rect<f32>=rect.as_ref();
                let dis = rect.distance_squared_to_point(point);
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
                    }
                };
                dis
            }
            let (_,k)=tree.as_owned_mut()
                .as_tree_mut()
                .k_nearest_fine_mut(cursor, 3, &mut kn,
                    move |_a,point,rect|{
                        f32n(distance_to_rect(rect.as_ref(),point.inner_into()))
                    },
                    move |a,point,t|{
                        a.rects.add(t.get().inner_into().into());
                        f32n(distance_to_rect(t.get().as_ref(),point.inner_into()))
                    },
                    dim);
            kn.rects.send_and_uniforms(canvas).with_color([1.0, 0.5, 0.3, 0.3]).draw();
            k
        };

        let mut vv: Vec<_> = vv
            .drain(..)
            .map(|a| Res {
                rect: a.bot.rect.inner_into(),
                mag: a.mag,
            })
            .collect();

        if check_naive {
            //unimplemented!()
            /*
            let mut kn = Kn {
                c: &c,
                g: RefCell::new(g),
                draw: false,
            };
            let dim=self.dim;
            tree.as_owned_mut().get_bots_mut(|bots|{
                analyze::NaiveAlgs::new(bots).assert_k_nearest_mut(
                    cursor,
                    3,
                    &mut kn,
                    dim,
                );
            });
            */
        }

        vv.reverse();
        let vv_iter = dinotree_alg::util::SliceSplit::new(&mut vv, |a, b| a.mag == b.mag);

        rect_save.uniforms(canvas).with_color([0.0,0.0,0.0,0.3]).draw();    
        
        for (a, color) in vv_iter.zip(cols.iter()) {
            if let Some(k) = a.first() {
                canvas.circles()
                    .add(cursor.inner_into().into())
                    .send_and_uniforms(canvas,k.mag.into_inner().sqrt()*2.0)
                    .with_color(*color)
                    .draw();
            }

            let mut rects = canvas.rects();
            for b in a.iter() {
                rects.add(b.rect.inner_into().into());
            }
            rects.send_and_uniforms(canvas).with_color(*color).draw();
        }
    })
}
