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
        bench_integer:f64,
    }

    fn instant_to_sec(elapsed:Duration)->f64{
         (elapsed.as_secs() as f64) + (elapsed.subsec_nanos() as f64 / 1000_000_000.0)
               
    }

    let mut records=Vec::new();

    for num_bots in (0..80000).step_by(200){
        let s2=s.clone();

        let mut bots:Vec<Bot>=s2.take(num_bots).enumerate().map(|(e,pos)|{
            let pos=[pos[0] as isize,pos[1] as isize];
            Bot{num:0,pos}
        }).collect();
        

        let bench_integer={
            let instant=Instant::now();
            
            let mut tree=DynTree::new(axgeom::XAXISS,(),&bots,|b|{   
                aabb_from_point_isize(b.pos,[5,5])
            });

            colfind::query_seq_mut(&mut tree,|a, b| {
                a.inner.num+=1;
                b.inner.num+=1;
            });

            tree.apply_orig_order(&mut bots,|a,b|{
                b.num=a.inner.num;
            });

            instant_to_sec(instant.elapsed())
        };

        let bench_float={
            let instant=Instant::now();

            let mut tree=DynTree::new_seq(axgeom::XAXISS,(),&bots,|b|{   
                unsafe{Conv::from_rect_unchecked(aabb_from_pointf64([b.pos[0] as f64,b.pos[1] as f64],[5.0,5.0]))}
            });

            colfind::query_seq_mut(&mut tree,|a, b| {
                a.inner.num+=1;
                b.inner.num+=1;
            });

            tree.apply_orig_order(&mut bots,|a,b|{
                b.num=a.inner.num;
            });

            instant_to_sec(instant.elapsed())
        };
        

        records.push(Record{num_bots,bench_float,bench_integer});
    }

    let rects=&mut records;
    use gnuplot::*;
    let x=rects.iter().map(|a|a.num_bots);
    let y1=rects.iter().map(|a|a.bench_float);
    let y2=rects.iter().map(|a|a.bench_integer);
    

    fg.axes2d()
        .set_title("Comparison of Benching AABB Collision Detection Algorithms", &[])
        .lines(x.clone(), y1,  &[Caption("Floating Point"), Color("blue"), LineWidth(2.0)])
        .lines(x.clone(), y2,  &[Caption("Integer"), Color("green"), LineWidth(2.0)])
        .set_x_label("Number of Objects", &[])
        .set_y_label("Time taken in seconds", &[]);

}




pub fn handle(fb:&FigureBuilder){
    let s=dists::spiral::Spiral::new([400.0,400.0],12.0,1.5);

    let mut fg=fb.new("colfind_float_vs_integer");
    handle_bench(&s,&mut fg);

    fg.show();
}