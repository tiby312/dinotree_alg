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
        bench_alg: f64,
        bench_par:f64,
        bench_sweep:Option<f64>,
        bench_naive:Option<f64>
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
        

        let c0={
            let instant=Instant::now();
            
            let mut tree=DynTree::new(axgeom::XAXISS,(),&bots,|b|{   
                aabb_from_point_isize(b.pos,[5,5])
            });

            colfind::query_mut(&mut tree,|a, b| {
                a.inner.num+=1;
                b.inner.num+=1;
        
            });

            tree.apply_orig_order(&mut bots,|a,b|{
                b.num=a.inner.num;
            });

            instant_to_sec(instant.elapsed())
        };

        let c1={
            let instant=Instant::now();

            let mut tree=DynTree::new_seq(axgeom::XAXISS,(),&bots,|b|{   
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
            
            if num_bots<5000{
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

        records.push(Record{num_bots,bench_alg:c1,bench_par:c0,bench_sweep:c3,bench_naive:c4});
    }

    let rects=&mut records;
    use gnuplot::*;
    let x=rects.iter().map(|a|a.num_bots);
    let y1=rects.iter().take_while(|a|a.bench_naive.is_some()).map(|a|a.bench_naive.unwrap());
    let y2=rects.iter().take_while(|a|a.bench_sweep.is_some()).map(|a|a.bench_sweep.unwrap());
    let y3=rects.iter().map(|a|a.bench_alg);
    let y4=rects.iter().map(|a|a.bench_par);


    fg.axes2d()
        .set_pos_grid(2,1,0)
        .set_title("Comparison of Benching AABB Collision Detection Algorithms", &[])
        .lines(x.clone(), y1,  &[Caption("Naive"), Color("blue"), LineWidth(2.0)])
        .lines(x.clone(), y2,  &[Caption("Sweep and Prune"), Color("green"), LineWidth(2.0)])
        .lines(x.clone(), y3,  &[Caption("Dinotree Sequential"), Color("red"), LineWidth(2.0)])
        .lines(x.clone(), y4,  &[Caption("Dinotree Parallel"), Color("violet"), LineWidth(2.0)])
        .set_x_label("Number of Objects", &[])
        .set_y_label("Time taken in seconds", &[]);

}





fn handle_theory(s:&dists::spiral::Spiral,fg:&mut Figure){

    #[derive(Debug)]
    struct Record {
        num_bots: usize,
        num_comparison_alg: usize,
        num_comparison_naive: Option<usize>,
        num_comparison_sweep:Option<usize>
    }

    let stop_naive_at=600;
    let stop_sweep_at=4000;

    let mut records=Vec::new();

    for num_bots in (0usize..10000).step_by(100){
        let s2=s.clone();
        let mut bots:Vec<Bot>=s2.take(num_bots).map(|pos|{
            let pos=[pos[0] as isize,pos[1] as isize];
            Bot{num:0,pos}
        }).collect();
        

        let c1={
            let mut counter=datanum::Counter::new();

            let mut tree=DynTree::new_seq(axgeom::XAXISS,(),&bots,|b|{
                datanum::from_rect(&mut counter,aabb_from_point_isize(b.pos,[5,5]))  
            });

            colfind::query_seq_mut(&mut tree,|a, b| {
                a.inner.num+=2;
                b.inner.num+=2;
            });
            
            tree.apply_orig_order(&mut bots,|a,b|{
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
        records.push(Record{num_bots,num_comparison_alg:c1,num_comparison_naive:c2,num_comparison_sweep:c3});
    }

    
        

    let rects=&mut records;
    use gnuplot::*;
    let x=rects.iter().map(|a|a.num_bots);
    let y1=rects.iter().map(|a|a.num_comparison_alg);
    let y2=rects.iter().take_while(|a|a.num_comparison_naive.is_some()).map(|a|a.num_comparison_naive.unwrap());
    let y3=rects.iter().take_while(|a|a.num_comparison_sweep.is_some()).map(|a|a.num_comparison_sweep.unwrap());

    fg.axes2d()
        .set_pos_grid(2,1,1)
        .set_title("Comparison of AABB Collision Detection Algorithms", &[])
        .lines(x.clone(), y2,  &[Caption("Naive"), Color("blue"), LineWidth(4.0)])
        .lines(x.clone(), y3,  &[Caption("Sweep and Prune"), Color("green"), LineWidth(2.0)])
        .lines(x.clone(), y1,  &[Caption("Dinotree"), Color("red"), LineWidth(2.0)])
        .set_x_label("Number of Objects", &[])
        .set_y_label("Number of Comparisons", &[]);

}


pub fn handle(fb:&FigureBuilder){
    let s=dists::spiral::Spiral::new([400.0,400.0],12.0,1.5);

    let mut fg=fb.new("colfind_theory");
    handle_theory(&s,&mut fg);
    handle_bench(&s,&mut fg);

    fg.show();
}