use crate::inner_prelude::*;

pub fn handle(fb: &mut FigureBuilder) {
    handle1(fb);
    handle2(fb);
}

fn handle1(fb: &mut FigureBuilder) {
    let mut fg = fb.build("spiral_data");

    let num_bots = 10000;
    let mut rects = Vec::new();
    for grow in (0..100).map(|a| {
        let a: f32 = a as f32;
        0.2 + a * 0.02
    }) {
        let s = dists::spiral::Spiral::new([0.0, 0.0], 17.0, grow);

        let mut bots: Vec<Vec2<f32>> = s.take(num_bots).collect();

        let mut bb = create_bbox_mut(&mut bots, |b| {
            axgeom::Rect::from_point(*b, vec2same(5.0))
                .inner_try_into::<NotNan<f32>>()
                .unwrap()
        });

        let mut tree = DinoTreeBuilder::new(axgeom::XAXISS, &mut bb).build_par();

        let mut num_intersection = 0;
        colfind::QueryBuilder::new(&mut tree).query_seq(|_a, _b| {
            num_intersection += 1;
        });

        /*
        tree.apply_orig_order(&mut bots,|a,b|{
            b.num=a.inner.num;
        });
        */

        rects.push((grow, num_intersection));
    }

    let x = rects.iter().map(|a| a.0);
    let y = rects.iter().map(|a| a.1);
    fg.axes2d()
    	.set_title("Number of Intersections with 10000 objects with a AABB for size 10 and a spiral separation of 17.0", &[])
        .lines(x, y,  &[Caption("Naive"), Color("red"), LineWidth(4.0)])
        .set_x_label("Spiral Grow", &[])
        .set_y_label("Number of Intersections", &[]);

    fb.finish(fg);
}

fn handle2(fb: &mut FigureBuilder) {
    fn make(grow: f32) -> Vec<Vec2<f32>> {
        let num_bots = 1000;

        let s = dists::spiral::Spiral::new([0.0, 0.0], 17.0, grow);

        let bots: Vec<Vec2<f32>> = s.take(num_bots).collect();
        bots
    };

    let mut fg = fb.build("spiral_visualize");

    let a = make(0.1);
    let ax = a.iter().map(|a| a.x);
    let ay = a.iter().map(|a| a.y);

    fg.axes2d()
        .set_pos_grid(2, 2, 0)
        .set_x_range(Fix(-500.0), Fix(500.0))
        .set_y_range(Fix(-500.0), Fix(500.0))
        .set_title("Grow of 0.1 of size 10000", &[])
        .points(ax, ay, &[Caption("Naive"), Color("red"), LineWidth(4.0)])
        .set_x_label("x", &[])
        .set_y_label("y", &[]);
    //fg.show();

    let a = make(0.5);
    let ax = a.iter().map(|a| a.x);
    let ay = a.iter().map(|a| a.y);

    fg.axes2d()
        .set_pos_grid(2, 2, 1)
        .set_x_range(Fix(-500.0), Fix(500.0))
        .set_y_range(Fix(-500.0), Fix(500.0))
        .set_title("Grow of 0.3 of size 10000", &[])
        .points(ax, ay, &[Caption("Naive"), Color("red"), LineWidth(4.0)])
        .set_x_label("x", &[])
        .set_y_label("y", &[]);

    //fb.finish(fg);

    let a = make(3.0);
    let ax = a.iter().map(|a| a.x);
    let ay = a.iter().map(|a| a.y);

    fg.axes2d()
        .set_pos_grid(2, 2, 2)
        .set_x_range(Fix(-500.0), Fix(500.0))
        .set_y_range(Fix(-500.0), Fix(500.0))
        .set_title("Grow of 3.0 of size 10000", &[])
        .points(ax, ay, &[Caption("Naive"), Color("red"), LineWidth(4.0)])
        .set_x_label("x", &[])
        .set_y_label("y", &[]);

    //fb.finish(fg);
    //fg.show();

    let a = make(6.0);
    let ax = a.iter().map(|a| a.x);
    let ay = a.iter().map(|a| a.y);

    fg.axes2d()
        .set_pos_grid(2, 2, 3)
        .set_x_range(Fix(-500.0), Fix(500.0))
        .set_y_range(Fix(-500.0), Fix(500.0))
        .set_title("Grow of 6.0 of size 10000", &[])
        .points(ax, ay, &[Caption("Naive"), Color("red"), LineWidth(4.0)])
        .set_x_label("x", &[])
        .set_y_label("y", &[]);
    //fg.show();
    fb.finish(fg);
}
