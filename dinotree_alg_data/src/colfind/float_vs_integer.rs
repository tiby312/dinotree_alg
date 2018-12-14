use inner_prelude::*;


#[derive(Copy,Clone)]
pub struct Bot{
    pos:[isize;2],
    num:usize
}




fn handle_bench(s:&dists::spiral::Spiral,fg:&mut Figure){

    use std::time::Instant;
    use std::time::Duration;
    #[derive(Debug)]
    struct Record {
        num_bots: usize,
        bench_float: f64,
        bench_float_par: f64,
        bench_integer:f64,
        bench_integer_par:f64,
        bench_f64:f64,
        bench_f64_par:f64
    }

    fn instant_to_sec(elapsed:Duration)->f64{
         (elapsed.as_secs() as f64) + (elapsed.subsec_nanos() as f64 / 1000_000_000.0)
               
    }

    let mut records=Vec::new();

    for num_bots in (0..80000).step_by(200){
        let s2=s.clone();

        let mut bots:Vec<Bot>=s2.take(num_bots).enumerate().map(|(_e,pos)|{
            let pos=[pos[0] as isize,pos[1] as isize];
            Bot{num:0,pos}
        }).collect();
        

        let bench_integer={
            let instant=Instant::now();
            
            let mut tree=DinoTree::new(axgeom::XAXISS,(),&bots,|b|{   
                aabb_from_point_isize(b.pos,[5,5])
            });

            colfind::query_seq_mut(tree.as_ref_mut(),|a, b| {
                a.inner.num+=1;
                b.inner.num+=1;
            });

            tree.apply(&mut bots,|a,b|{
                b.num=a.inner.num;
            });

            instant_to_sec(instant.elapsed())
        };

        let bench_float={
            let instant=Instant::now();

            let mut tree=DinoTree::new_seq(axgeom::XAXISS,(),&bots,|b|{   
                unsafe{ConvF32::from_rect_unchecked(aabb_from_pointf32([b.pos[0] as f32,b.pos[1] as f32],[5.0,5.0]))}
            });

            colfind::query_seq_mut(tree.as_ref_mut(),|a, b| {
                a.inner.num+=1;
                b.inner.num+=1;
            });

            tree.apply(&mut bots,|a,b|{
                b.num=a.inner.num;
            });

            instant_to_sec(instant.elapsed())
        };

        let bench_float_par={
            let instant=Instant::now();

            let mut tree=DinoTree::new(axgeom::XAXISS,(),&bots,|b|{   
                unsafe{ConvF32::from_rect_unchecked(aabb_from_pointf32([b.pos[0] as f32,b.pos[1] as f32],[5.0,5.0]))}
            });

            colfind::query_mut(tree.as_ref_mut(),|a, b| {
                a.inner.num+=1;
                b.inner.num+=1;
            });

            tree.apply(&mut bots,|a,b|{
                b.num=a.inner.num;
            });

            instant_to_sec(instant.elapsed())
        };

        let bench_integer_par={
            let instant=Instant::now();
            
            let mut tree=DinoTree::new(axgeom::XAXISS,(),&bots,|b|{   
                aabb_from_point_isize(b.pos,[5,5])
            });

            colfind::query_mut(tree.as_ref_mut(),|a, b| {
                a.inner.num+=1;
                b.inner.num+=1;
            });

            tree.apply(&mut bots,|a,b|{
                b.num=a.inner.num;
            });

            instant_to_sec(instant.elapsed())
        };

        let bench_f64={
            let instant=Instant::now();

            let mut tree=DinoTree::new_seq(axgeom::XAXISS,(),&bots,|b|{   
                unsafe{ConvF64::from_rect_unchecked(aabb_from_pointf64([b.pos[0] as f64,b.pos[1] as f64],[5.0,5.0]))}
            });

            colfind::query_seq_mut(tree.as_ref_mut(),|a, b| {
                a.inner.num+=1;
                b.inner.num+=1;
            });

            tree.apply(&mut bots,|a,b|{
                b.num=a.inner.num;
            });

            instant_to_sec(instant.elapsed())
        };

        let bench_f64_par={
            let instant=Instant::now();

            let mut tree=DinoTree::new(axgeom::XAXISS,(),&bots,|b|{   
                unsafe{ConvF64::from_rect_unchecked(aabb_from_pointf64([b.pos[0] as f64,b.pos[1] as f64],[5.0,5.0]))}
            });

            colfind::query_mut(tree.as_ref_mut(),|a, b| {
                a.inner.num+=1;
                b.inner.num+=1;
            });

            tree.apply(&mut bots,|a,b|{
                b.num=a.inner.num;
            });

            instant_to_sec(instant.elapsed())  
        };
        

        records.push(Record{num_bots,bench_float,bench_integer,bench_float_par,bench_integer_par,bench_f64,bench_f64_par});
    }

    let rects=&mut records;
    use gnuplot::*;
    let x=rects.iter().map(|a|a.num_bots);
    let y1=rects.iter().map(|a|a.bench_float);
    let y2=rects.iter().map(|a|a.bench_integer);
    let y3=rects.iter().map(|a|a.bench_float_par);
    let y4=rects.iter().map(|a|a.bench_integer_par);
    let y5=rects.iter().map(|a|a.bench_f64);
    let y6=rects.iter().map(|a|a.bench_f64_par);

    fg.axes2d()
        .set_title("Comparison of DinoTree Performance With Different Number Types With Grow=2.0", &[])
        .lines(x.clone(), y1,  &[Caption("f32"), Color("blue"), LineWidth(1.6)])
        .lines(x.clone(), y2,  &[Caption("isize"), Color("green"), LineWidth(1.6)])
        .lines(x.clone(), y3,  &[Caption("f32 parallel"), Color("red"), LineWidth(1.6)])
        .lines(x.clone(), y4,  &[Caption("isize parallel"), Color("orange"), LineWidth(1.6)])
        .lines(x.clone(), y5,  &[Caption("f64"), Color("violet"), LineWidth(1.6)])
        .lines(x.clone(), y6,  &[Caption("f64 parallel"), Color("yellow"), LineWidth(1.6)])
        .set_x_label("Number of Objects", &[])
        .set_y_label("Time taken in seconds", &[]);

}




pub fn handle(fb:&mut FigureBuilder){
    let s=dists::spiral::Spiral::new([400.0,400.0],12.0,2.0);

    let mut fg=fb.new("colfind_float_vs_integer");
    handle_bench(&s,&mut fg);
    fb.finish(fg);
}