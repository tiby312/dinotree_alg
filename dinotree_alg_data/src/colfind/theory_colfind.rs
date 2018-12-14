use inner_prelude::*;


#[derive(Copy,Clone)]
pub struct Bot{
    pos:[isize;2],
    num:usize
}




fn handle_bench(s:&dists::spiral::Spiral,fg:&mut Figure,title:&str){

    use std::time::Instant;
    use std::time::Duration;
    #[derive(Debug)]
    struct Record {
        num_bots: usize,
        bench_alg: f64,
        bench_par:f64,
        bench_sweep:Option<f64>,
        bench_naive:Option<f64>,
        bench_nosort_par:Option<f64>,
        bench_nosort_seq:Option<f64>
    }

    fn instant_to_sec(elapsed:Duration)->f64{
         (elapsed.as_secs() as f64) + (elapsed.subsec_nanos() as f64 / 1000_000_000.0)
               
    }

    let mut records=Vec::new();

    for num_bots in (0..120_000).step_by(400){
        let s2=s.clone();

        let mut bots:Vec<Bot>=s2.take(num_bots).enumerate().map(|(_e,pos)|{
            let pos=[pos[0] as isize,pos[1] as isize];
            Bot{num:0,pos}
        }).collect();
        

        let c0={
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

        let c1={
            let instant=Instant::now();

            let mut tree=DinoTree::new_seq(axgeom::XAXISS,(),&bots,|b|{   
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
        
        let c3={
            if num_bots<50000{
                let mut bb:Vec<BBoxDemo<isize,Bot>>=bots.iter().map(|b|{
                    BBoxDemo::new(aabb_from_point_isize(b.pos,[5,5]),*b)
                }).collect();


                let instant=Instant::now();
                
                colfind::query_sweep_mut(axgeom::XAXISS,&mut bb,|a,b|{
                    a.inner.num-=2;
                    b.inner.num-=2;
                });

                for b in bb.iter(){
                    assert_eq!(b.inner.num,0);
                } 

                Some(instant_to_sec(instant.elapsed()))
            }else{
                None
            }
        };

        let c4={
            
            if num_bots<8000{
                let mut bb:Vec<BBoxDemo<isize,Bot>>=bots.iter().map(|b|{
                    let rect=aabb_from_point_isize(b.pos,[5,5]);
                    BBoxDemo::new(rect,*b)
                }).collect();

                let instant=Instant::now();
            
                colfind::query_naive_mut(&mut bb,|a,b|{
                    a.inner.num-=1;
                    b.inner.num-=1;
                });


                for (a,b) in bb.iter().zip(bots.iter_mut()){
                    *b=a.inner;
                }


                Some(instant_to_sec(instant.elapsed()))
            }else{
                None
            }
        };

        let c5={
            if num_bots<120000{
                let instant=Instant::now();

                let mut tree=advanced::NotSorted::new(axgeom::XAXISS,(),&bots,|b|{   
                    aabb_from_point_isize(b.pos,[5,5])
                });

                colfind::query_nosort_mut(&mut tree,|a, b| {
                    a.inner.num+=1;
                    b.inner.num+=1;
                });

                tree.0.apply(&mut bots,|a,b|{
                    b.num=a.inner.num;
                });


                Some(instant_to_sec(instant.elapsed()))
            }else {
                None
            }
        };

        let c6={
            if num_bots<120000{
                let instant=Instant::now();

                let mut tree=advanced::NotSorted::new_seq(axgeom::XAXISS,(),&bots,|b|{   
                    aabb_from_point_isize(b.pos,[5,5])
                });

                colfind::query_nosort_seq_mut(&mut tree,|a, b| {
                    a.inner.num+=1;
                    b.inner.num+=1;
                });

                tree.0.apply(&mut bots,|a,b|{
                    b.num=a.inner.num;
                });


                Some(instant_to_sec(instant.elapsed()))
            }else {
                None
            }
        };

        records.push(Record{num_bots,bench_alg:c1,bench_par:c0,bench_sweep:c3,bench_naive:c4,bench_nosort_par:c5,bench_nosort_seq:c6});
    }

    let rects=&mut records;
    use gnuplot::*;
    let x=rects.iter().map(|a|a.num_bots);
    let y1=rects.iter().take_while(|a|a.bench_naive.is_some()).map(|a|a.bench_naive.unwrap());
    let y2=rects.iter().take_while(|a|a.bench_sweep.is_some()).map(|a|a.bench_sweep.unwrap());
    let y3=rects.iter().map(|a|a.bench_alg);
    let y4=rects.iter().map(|a|a.bench_par);
    let y5=rects.iter().take_while(|a|a.bench_nosort_par.is_some()).map(|a|a.bench_nosort_par.unwrap());
    let y6=rects.iter().take_while(|a|a.bench_nosort_seq.is_some()).map(|a|a.bench_nosort_seq.unwrap());


    fg.axes2d()
        .set_pos_grid(2,1,1)
        .set_title(title, &[])
        .set_legend(Graph(1.0),Graph(1.0),&[LegendOption::Horizontal],&[])
        .lines(x.clone(), y1,  &[Caption("Naive"), Color("blue"), LineWidth(2.0)])
        .lines(x.clone(), y2,  &[Caption("Sweep and Prune"), Color("green"), LineWidth(2.0)])
        .lines(x.clone(), y3,  &[Caption("Dinotree Sequential"), Color("red"), LineWidth(2.0)])
        .lines(x.clone(), y4,  &[Caption("Dinotree Parallel"), Color("violet"), LineWidth(2.0)])
        .lines(x.clone(), y5,  &[Caption("KD Tree Parallel"), Color("black"), LineWidth(2.0)])
        .lines(x.clone(), y6,  &[Caption("KD Tree Sequential"), Color("brown"), LineWidth(2.0)])
        .set_x_label("Number of Objects", &[])
        .set_y_label("Time taken in seconds", &[]);

}





fn handle_theory(s:&dists::spiral::Spiral,fg:&mut Figure,title:&str){

    #[derive(Debug)]
    struct Record {
        num_bots: usize,
        num_comparison_alg: usize,
        num_comparison_naive: Option<usize>,
        num_comparison_sweep:Option<usize>,
        num_comparison_nosort:usize
    }

    let stop_naive_at=2000;
    let stop_sweep_at=20000;

    let mut records=Vec::new();

    for num_bots in (0usize..20000).step_by(200){
        let s2=s.clone();
        let mut bots:Vec<Bot>=s2.take(num_bots).map(|pos|{
            let pos=[pos[0] as isize,pos[1] as isize];
            Bot{num:0,pos}
        }).collect();
        

        let c1={
            let mut counter=datanum::Counter::new();

            let mut tree=DinoTree::new_seq(axgeom::XAXISS,(),&bots,|b|{
                datanum::from_rect(&mut counter,aabb_from_point_isize(b.pos,[5,5]))  
            });

            colfind::query_seq_mut(tree.as_ref_mut(),|a, b| {
                a.inner.num+=2;
                b.inner.num+=2;
            });
            
            tree.apply(&mut bots,|a,b|{
                *b=a.inner;
            });

            counter.into_inner()
        };
       
        let c2={
            if num_bots<stop_naive_at{
                let mut counter=datanum::Counter::new();
            
                let mut bb:Vec<BBoxDemo<datanum::DataNum,Bot>>=bots.iter().map(|b|{
                    let rect=aabb_from_point_isize(b.pos,[5,5]);
                    BBoxDemo::new(datanum::from_rect(&mut counter,rect),*b)
                }).collect();

                colfind::query_naive_mut(&mut bb,|a,b|{
                    a.inner.num-=1;
                    b.inner.num-=1;
                });


                for (a,b) in bb.iter().zip(bots.iter_mut()){
                    *b=a.inner;
                }
                Some(counter.into_inner())
            }else{
                None
            }
        };
        let c3={
            if num_bots<stop_sweep_at{
                let mut counter=datanum::Counter::new();
                let mut bb:Vec<BBoxDemo<datanum::DataNum,Bot>>=bots.iter().map(|b|{
                    let rect=aabb_from_point_isize(b.pos,[5,5]);
                    BBoxDemo::new(datanum::from_rect(&mut counter,rect),*b)
                }).collect();

                colfind::query_sweep_mut(axgeom::XAXISS,&mut bb,|a,b|{
                    a.inner.num-=1;
                    b.inner.num-=1;
                });

                //println!("Number of comparisions naive={}",counter.into_inner());   
                for (a,b) in bb.iter().zip(bots.iter_mut()){
                    *b=a.inner;
                }
                 
                Some(counter.into_inner())
            }else{
                None
            }
        };

        let c4={
            let mut counter=datanum::Counter::new();

            let mut tree=NotSorted::new_seq(axgeom::XAXISS,(),&bots,|b|{
                datanum::from_rect(&mut counter,aabb_from_point_isize(b.pos,[5,5]))  
            });

            colfind::query_nosort_seq_mut(&mut tree,|a, b| {
                a.inner.num+=2;
                b.inner.num+=2;
            });
            
            tree.0.apply(&mut bots,|a,b|{
                *b=a.inner;
            });

            counter.into_inner()
        };

        records.push(Record{num_bots,num_comparison_alg:c1,num_comparison_naive:c2,num_comparison_sweep:c3,num_comparison_nosort:c4});
    }

    
        

    let rects=&mut records;
    use gnuplot::*;
    let x=rects.iter().map(|a|a.num_bots);
    let y1=rects.iter().map(|a|a.num_comparison_alg);
    let y2=rects.iter().take_while(|a|a.num_comparison_naive.is_some()).map(|a|a.num_comparison_naive.unwrap());
    let y3=rects.iter().take_while(|a|a.num_comparison_sweep.is_some()).map(|a|a.num_comparison_sweep.unwrap());
    //let y4=rects.iter().take_while(|a|a.num_comparison_nosort.is_some()).map(|a|a.num_comparison_nosort.unwrap());
    let y4=rects.iter().map(|a|a.num_comparison_nosort);

    fg.axes2d()
        .set_pos_grid(2,1,0)
        .set_title(title, &[])
        .set_legend(Graph(1.0),Graph(1.0),&[LegendOption::Horizontal],&[])
        .lines(x.clone(), y2,  &[Caption("Naive"), Color("blue"), LineWidth(4.0)])
        .lines(x.clone(), y3,  &[Caption("Sweep and Prune"), Color("green"), LineWidth(2.0)])
        .lines(x.clone(), y1,  &[Caption("Dinotree"), Color("red"), LineWidth(2.0)])
        .lines(x.clone(), y4,  &[Caption("KDTree"), Color("brown"), LineWidth(2.0)])
        .set_x_label("Number of Objects", &[])
        .set_y_label("Number of Comparisons", &[]);

}


pub fn handle(fb:&mut FigureBuilder){
    {
        let s=dists::spiral::Spiral::new([400.0,400.0],12.0,1.0);
        let mut fg=fb.new("colfind_theory");
        handle_theory(&s,&mut fg,"Comparison of space partitioning algs with dinotree grow of 1.0");
        handle_bench(&s,&mut fg,"Comparison of space partitioning algs with dinotree grow of 1.0");
        //fg.echo(&mut std::io::stdout());
        fb.finish(fg);
    }
    
    {
        let s=dists::spiral::Spiral::new([400.0,400.0],12.0,0.05);
        let mut fg=fb.new("colfind_theory_0.05");
        handle_theory(&s,&mut fg,"Comparison of space partitioning algs with dinotree grow of 0.05");
        handle_bench(&s,&mut fg,"Comparison of space partitioning algs with dinotree grow of 0.05");
        fb.finish(fg)   
    }
    
    
}
