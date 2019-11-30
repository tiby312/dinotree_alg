use crate::inner_prelude::*;

#[derive(Copy, Clone)]
pub struct Bot {
    pos: Vec2<f32>,
    num: usize,
}

fn handle_bench_inner(grow: f32, fg: &mut Figure, title: &str, yposition: usize) {
    #[derive(Debug)]
    struct Record {
        num_bots: usize,
        bench_alg: f64,
        bench_par: f64,
        bench_sweep: Option<f64>,
        bench_naive: Option<f64>,
        bench_nosort_par: Option<f64>,
        bench_nosort_seq: Option<f64>,
    }

    let mut records = Vec::new();

    for num_bots in (0..40_000).rev().step_by(500) {
        let mut scene = bot::BotSceneBuilder::new(num_bots)
            .with_grow(grow)
            .build_specialized(|pos| Bot { pos, num: 0 });
        let mut bots = &mut scene.bots;
        let prop = &scene.bot_prop;

        for b in bots.iter_mut() {
            b.num = 0;
        }

        let mut bb = bbox_helper::create_bbox_mut(&mut bots, |b| prop.create_bbox_nan(b.pos));

        let c0 = {
            let instant = Instant::now();

            let mut tree = DinoTree::new_par(&mut bb);

            tree.find_collisions_mut_par(|mut a, mut b| {
                a.inner_mut().num += 1;
                b.inner_mut().num += 1;
            });

            instant_to_sec(instant.elapsed())
        };

        let c1 = {
            let instant = Instant::now();

            let mut tree = DinoTree::new( &mut bb);

            tree.find_collisions_mut(|mut a, mut b| {
                a.inner_mut().num += 1;
                b.inner_mut().num += 1;
            });

            instant_to_sec(instant.elapsed())
        };

        let c3 = {
            if num_bots < 20000 {
                let instant = Instant::now();

                analyze::NaiveAlgs::new(&mut bb).find_collisions_sweep_mut(axgeom::XAXISS, |mut a, mut b| {
                    a.inner_mut().num -= 2;
                    b.inner_mut().num -= 2;
                });

                for b in bb.iter() {
                    assert_eq!(b.inner.num, 0);
                }

                Some(instant_to_sec(instant.elapsed()))
            } else {
                None
            }
        };

        let c4 = {
            if num_bots < 8000 {
                let instant = Instant::now();

                analyze::NaiveAlgs::new(&mut bb).find_collisions_mut(|mut a, mut b| {
                    a.inner_mut().num -= 1;
                    b.inner_mut().num -= 1;
                });

                Some(instant_to_sec(instant.elapsed()))
            } else {
                None
            }
        };

        let c5 = {
            let instant = Instant::now();

            let mut tree = NotSorted::new_par(&mut bb);

            tree.find_collisions_mut_par(|mut a, mut b| {
                a.inner_mut().num += 1;
                b.inner_mut().num += 1;
            });

            Some(instant_to_sec(instant.elapsed()))
        };

        let c6 = {
            let instant = Instant::now();

            let mut tree = NotSorted::new( &mut bb);

            tree.find_collisions_mut(|mut a, mut b| {
                a.inner_mut().num += 1;
                b.inner_mut().num += 1;
            });

            Some(instant_to_sec(instant.elapsed()))
        };

        records.push(Record {
            num_bots,
            bench_alg: c1,
            bench_par: c0,
            bench_sweep: c3,
            bench_naive: c4,
            bench_nosort_par: c5,
            bench_nosort_seq: c6,
        });
    }

    records.reverse();

    let rects = &mut records;
    use gnuplot::*;
    let x = rects.iter().map(|a| a.num_bots);
    let y1 = rects
        .iter()
        .take_while(|a| a.bench_naive.is_some())
        .map(|a| a.bench_naive.unwrap());
    let y2 = rects
        .iter()
        .take_while(|a| a.bench_sweep.is_some())
        .map(|a| a.bench_sweep.unwrap());
    let y3 = rects.iter().map(|a| a.bench_alg);
    let y4 = rects.iter().map(|a| a.bench_par);
    let y5 = rects
        .iter()
        .take_while(|a| a.bench_nosort_par.is_some())
        .map(|a| a.bench_nosort_par.unwrap());
    let y6 = rects
        .iter()
        .take_while(|a| a.bench_nosort_seq.is_some())
        .map(|a| a.bench_nosort_seq.unwrap());

    fg.axes2d()
        .set_pos_grid(2, 1, yposition as u32)
        .set_title(title, &[])
        .set_legend(Graph(1.0), Graph(1.0), &[LegendOption::Horizontal], &[])
        .lines(
            x.clone(),
            y1,
            &[Caption("Naive"), Color("blue"), LineWidth(2.0)],
        )
        .lines(
            x.clone(),
            y2,
            &[Caption("Sweep and Prune"), Color("green"), LineWidth(2.0)],
        )
        .lines(
            x.clone(),
            y3,
            &[Caption("Dinotree Sequential"), Color("red"), LineWidth(2.0)],
        )
        .lines(
            x.clone(),
            y4,
            &[
                Caption("Dinotree Parallel"),
                Color("violet"),
                LineWidth(2.0),
            ],
        )
        .lines(
            x.clone(),
            y5,
            &[Caption("KD Tree Parallel"), Color("black"), LineWidth(2.0)],
        )
        .lines(
            x.clone(),
            y6,
            &[
                Caption("KD Tree Sequential"),
                Color("brown"),
                LineWidth(2.0),
            ],
        )
        .set_x_label("Number of Objects", &[])
        .set_y_label("Time taken in seconds", &[]);
}

fn handle_theory_inner(grow: f32, fg: &mut Figure, title: &str, yposition: usize) {
    #[derive(Debug)]
    struct Record {
        num_bots: usize,
        num_comparison_alg: usize,
        num_comparison_naive: Option<usize>,
        num_comparison_sweep: Option<usize>,
        num_comparison_nosort: usize,
    }

    let stop_naive_at = 9_000;
    let stop_sweep_at = 30_000;

    let mut records = Vec::new();

    for num_bots in (0usize..30_000).step_by(500) {
        let mut scene = bot::BotSceneBuilder::new(num_bots)
            .with_grow(grow)
            .build_specialized(|pos| Bot { pos, num: 0 });

        let mut bots = &mut scene.bots;
        let prop = &scene.bot_prop;

        for b in bots.iter_mut() {
            b.num = 0;
        }

        let c1 = {
            let mut counter = datanum::Counter::new();

            let mut bb = bbox_helper::create_bbox_mut(&mut bots, |b| {
                datanum::from_rect(&mut counter, prop.create_bbox_nan(b.pos))
            });

            let mut tree = DinoTree::new(&mut bb);

            tree.find_collisions_mut(|mut a, mut b| {
                a.inner_mut().num += 2;
                b.inner_mut().num += 2;
            });

            counter.into_inner()
        };

        let c2 = {
            if num_bots < stop_naive_at {
                let mut counter = datanum::Counter::new();

                let mut bb = bbox_helper::create_bbox_mut(&mut bots, |b| {
                    datanum::from_rect(&mut counter, prop.create_bbox_nan(b.pos))
                });

                analyze::NaiveAlgs::new(&mut bb).find_collisions_mut(|mut a, mut b| {
                    a.inner_mut().num -= 1;
                    b.inner_mut().num -= 1;
                });

                Some(counter.into_inner())
            } else {
                None
            }
        };
        let c3 = {
            if num_bots < stop_sweep_at {
                let mut counter = datanum::Counter::new();

                let mut bb = bbox_helper::create_bbox_mut(&mut bots, |b| {
                    datanum::from_rect(&mut counter, prop.create_bbox_nan(b.pos))
                });

                analyze::NaiveAlgs::new(&mut bb).find_collisions_sweep_mut(axgeom::XAXISS, |mut a, mut b| {
                    a.inner_mut().num -= 1;
                    b.inner_mut().num -= 1;
                });

                Some(counter.into_inner())
            } else {
                None
            }
        };

        let c4 = {
            let mut counter = datanum::Counter::new();

            let mut bb = bbox_helper::create_bbox_mut(&mut bots, |b| {
                datanum::from_rect(&mut counter, prop.create_bbox_nan(b.pos))
            });

            let mut tree = NotSorted::new( &mut bb);

            tree.find_collisions_mut(|mut a, mut b| {
                a.inner_mut().num += 2;
                b.inner_mut().num += 2;
            });

            counter.into_inner()
        };

        let r = Record {
            num_bots,
            num_comparison_alg: c1,
            num_comparison_naive: c2,
            num_comparison_sweep: c3,
            num_comparison_nosort: c4,
        };

        records.push(r);
    }

    let rects = &mut records;
    use gnuplot::*;
    let x = rects.iter().map(|a| a.num_bots);
    let y1 = rects.iter().map(|a| a.num_comparison_alg);
    let y2 = rects
        .iter()
        .take_while(|a| a.num_comparison_naive.is_some())
        .map(|a| a.num_comparison_naive.unwrap());
    let y3 = rects
        .iter()
        .take_while(|a| a.num_comparison_sweep.is_some())
        .map(|a| a.num_comparison_sweep.unwrap());
    let y4 = rects.iter().map(|a| a.num_comparison_nosort);

    fg.axes2d()
        .set_pos_grid(2, 1, yposition as u32)
        .set_title(title, &[])
        .set_legend(Graph(1.0), Graph(1.0), &[LegendOption::Horizontal], &[])
        .lines(
            x.clone(),
            y2,
            &[Caption("Naive"), Color("blue"), LineWidth(4.0)],
        )
        .lines(
            x.clone(),
            y3,
            &[Caption("Sweep and Prune"), Color("green"), LineWidth(2.0)],
        )
        .lines(
            x.clone(),
            y1,
            &[Caption("Dinotree"), Color("red"), LineWidth(2.0)],
        )
        .lines(
            x.clone(),
            y4,
            &[Caption("KDTree"), Color("brown"), LineWidth(2.0)],
        )
        .set_x_label("Number of Objects", &[])
        .set_y_label("Number of Comparisons", &[]);
}

pub fn handle_theory(fb: &mut FigureBuilder) {
    let mut fg = fb.build("colfind_theory");

    handle_theory_inner(
        1.0,
        &mut fg,
        "Comparison of space partitioning algs with abspiral(x,1.0)",
        0,
    );
    handle_theory_inner(
        0.05,
        &mut fg,
        "Comparison of space partitioning algs with abspiral(x,0.05)",
        1,
    );
    fb.finish(fg)
}

pub fn handle_bench(fb: &mut FigureBuilder) {
    let mut fg = fb.build("colfind_bench");
    handle_bench_inner(
        1.0,
        &mut fg,
        "Comparison of space partitioning algs with abspiral(x,1.0)",
        0,
    );
    handle_bench_inner(
        0.05,
        &mut fg,
        "Comparison of space partitioning algs with abspiral(x,0.05)",
        1,
    );

    fb.finish(fg);
}
