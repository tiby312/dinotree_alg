use inner_prelude::*;



#[derive(Copy,Clone)]
pub struct Bot{
    num:usize,
    pos:[isize;2]
}




fn test1(bots:&mut [Bot])->(usize,usize){
    
    let mut counter=datanum::Counter::new();

    let mut tree=DynTree::new_seq(axgeom::XAXISS,(),bots,|b|{
        datanum::from_rect(&mut counter,aabb_from_point_isize(b.pos,[5,5]))  
    });

    let a=counter.into_inner();

    colfind::query_seq_mut(&mut tree,|a, b| {
        a.inner.num+=2;
        b.inner.num+=2;
    });

    tree.apply_orig_order(bots,|a,b|{
        b.num=a.inner.num;
    });

    let b=counter.into_inner();

    return (a,(b-a));
}



fn test2(bots:&mut [Bot])->(f64,f64){
    
    let instant=Instant::now();

    let mut tree=DynTree::new_seq(axgeom::XAXISS,(),bots,|b|{
        aabb_from_point_isize(b.pos,[5,5])  
    });


    let a=instant_to_sec(instant.elapsed());
    

    colfind::query_seq_mut(&mut tree,|a, b| {
        a.inner.num+=2;
        b.inner.num+=2;
    });

    tree.apply_orig_order(bots,|a,b|{
        b.num=a.inner.num;
    });

    let b=instant_to_sec(instant.elapsed());

    return (a,(b-a));
}

fn test3(bots:&mut [Bot])->(f64,f64){
    
    let instant=Instant::now();

    let mut tree=DynTree::new(axgeom::XAXISS,(),bots,|b|{
        aabb_from_point_isize(b.pos,[5,5])  
    });


    let a=instant_to_sec(instant.elapsed());
    

    colfind::query_mut(&mut tree,|a, b| {
        a.inner.num+=2;
        b.inner.num+=2;
    });

    tree.apply_orig_order(bots,|a,b|{
        b.num=a.inner.num;
    });

    let b=instant_to_sec(instant.elapsed());

    return (a,(b-a));
}


pub fn handle(fb:&FigureBuilder){
    handle_num_bots(fb);
    handle_grow(fb);
}

fn handle_num_bots(fb:&FigureBuilder){
    #[derive(Debug)]
    struct Record {
        num_bots:usize,
        theory: (usize,usize),
        bench:(f64,f64),
        bench_par:(f64,f64)        
    }

    let grow=0.2;

    let s=dists::spiral::Spiral::new([400.0,400.0],17.0,grow);
    let mut rects=Vec::new();

    for num_bots in (1..40_000).step_by(500){

        let mut bots:Vec<Bot>=s.clone().take(num_bots).map(|pos|{
            let pos=[pos[0] as isize,pos[1] as isize];
            Bot{num:0,pos}
        }).collect();
    

        let theory=test1(&mut bots);
        let bench=test2(&mut bots);
        let bench_par=test3(&mut bots);

        let r=Record{num_bots,theory,bench,bench_par};
        rects.push(r);      
    }


    let x=rects.iter().map(|a|a.num_bots);
    let y1=rects.iter().map(|a|a.theory.0);
    let y2=rects.iter().map(|a|a.theory.1);

    let mut fg= fb.new("colfind_rebal_vs_query_num_bots");
    
    fg.axes2d()
        .set_pos_grid(2,1,0)
        .set_title("Rebal vs Query Comparisons with a spiral grow of 0.2", &[])
        .lines(x.clone(), y1,  &[Caption("Rebalance"), Color("blue"), LineWidth(2.0)])
        .lines(x.clone(), y2,  &[Caption("Query"), Color("green"), LineWidth(2.0)])
        .set_x_label("Number of Elements", &[])
        .set_y_label("Number of Comparisons", &[]);


    let y1=rects.iter().map(|a|a.bench.0);
    let y2=rects.iter().map(|a|a.bench.1);
    let y3=rects.iter().map(|a|a.bench_par.0);
    let y4=rects.iter().map(|a|a.bench_par.1);

    fg.axes2d()
        .set_pos_grid(2,1,1)
        .set_title("Rebal vs Query Benches with a spiral grow of 0.2", &[])
        .lines(x.clone(), y1,  &[Caption("Rebal Sequential"), Color("blue"), LineWidth(2.0)])
        .lines(x.clone(), y2,  &[Caption("Query Sequential"), Color("green"), LineWidth(2.0)])
        .lines(x.clone(), y3,  &[Caption("Rebal Parallel"), Color("red"), LineWidth(2.0)])
        .lines(x.clone(), y4,  &[Caption("Query Parallel"), Color("brown"), LineWidth(2.0)])
        .set_x_label("Grow", &[])
        .set_y_label("Time in seconds", &[]);

    fg.show();


}
fn handle_grow(fb:&FigureBuilder){
    let num_bots=20_000;

    #[derive(Debug)]
    struct Record {
        grow:f64,
        theory: (usize,usize),
        bench:(f64,f64),
        bench_par:(f64,f64)        
    }

    let mut rects=Vec::new();

    for grow in (0..200).map(|a|0.1+(a as f64)*0.005){
        let s=dists::spiral::Spiral::new([400.0,400.0],17.0,grow);

        let mut bots:Vec<Bot>=s.take(num_bots).map(|pos|{
            let pos=[pos[0] as isize,pos[1] as isize];
            Bot{num:0,pos}
        }).collect();


        let theory=test1(&mut bots);
        let bench=test2(&mut bots);
        let bench_par=test3(&mut bots);

        let r=Record{grow,theory,bench,bench_par};
        rects.push(r);   
    }


    
    let x=rects.iter().map(|a|a.grow as f64);
    let y1=rects.iter().map(|a|a.theory.0);
    let y2=rects.iter().map(|a|a.theory.1);

    let mut fg= fb.new("colfind_rebal_vs_query_theory_spiral");
    
    fg.axes2d()
        .set_pos_grid(2,1,0)
        .set_title("Rebal vs Query Comparisons with 20000 objects", &[])
        .lines(x.clone(), y1,  &[Caption("Rebalance"), Color("blue"), LineWidth(2.0)])
        .lines(x.clone(), y2,  &[Caption("Query"), Color("green"), LineWidth(2.0)])
        .set_x_label("Grow", &[])
        .set_y_label("Number of comparisons", &[]);


    let y1=rects.iter().map(|a|a.bench.0);
    let y2=rects.iter().map(|a|a.bench.1);
    let y3=rects.iter().map(|a|a.bench_par.0);
    let y4=rects.iter().map(|a|a.bench_par.1);

    fg.axes2d()
        .set_pos_grid(2,1,1)
        .set_title("Rebal vs Query Benches with a 20,000 objects", &[])
        .lines(x.clone(), y1,  &[Caption("Rebal Sequential"), Color("blue"), LineWidth(2.0)])
        .lines(x.clone(), y2,  &[Caption("Query Sequential"), Color("green"), LineWidth(2.0)])
        .lines(x.clone(), y3,  &[Caption("Rebal Parallel"), Color("red"), LineWidth(2.0)])
        .lines(x.clone(), y4,  &[Caption("Query Parallel"), Color("brown"), LineWidth(2.0)])
        .set_x_label("Grow", &[])
        .set_y_label("Time in seconds", &[]);

    fg.show();
    
    

}

/*
fn handle1(fb:&FigureBuilder){


    #[derive(Debug)]
    struct Record {
        num_bots: usize,
        grow:f64,
        z: f64,
        z2:f64,
        z3:f64        
    }
    let mut rects=Vec::new();
    
    for grow in (0..50).map(|a|0.2+(a as f64)*0.2){

        for num_bots in (0..20000usize).step_by(1000){
            let s=dists::spiral::Spiral::new([400.0,400.0],17.0,grow);

            let mut bots:Vec<Bot>=s.take(num_bots).map(|pos|{
                let pos=[pos[0] as isize,pos[1] as isize];
                Bot{num:0,pos}
            }).collect();

            let z=test1(&mut bots);
            let z2=test2(&mut bots);
            let z3=test3(&mut bots);

            let r=Record{num_bots,grow,z,z2,z3};
            rects.push(r);   
        }
    }



    let x=rects.iter().map(|a|a.num_bots as f64);
    let y=rects.iter().map(|a|a.grow as f64);
    let z=rects.iter().map(|a|a.z);
    let z2=rects.iter().map(|a|a.z2);
    let z3=rects.iter().map(|a|a.z3);
    
    

    let mut fg=fb.new("query_over_total_theory");
    

    fg.axes3d().set_view(110.0,30.0)
        .set_title("Comparisons of Querying Over Total Comparisons", &[])
        .set_x_label("Number of Objects", &[])
        .set_y_label("Spareness of Objects", &[])
        .set_z_label("Query/Total Comparisons", &[Rotate(90.0),TextOffset(-3.0,0.0)])
        .points(x.clone(), y.clone(), z.clone(), &[PointSymbol('O'), Color("red"), PointSize(1.0)]);
                    

    fg.show();


 
    let mut fg=fb.new("query_over_total_bench");

    fg.axes3d().set_view(110.0,30.0)
        .set_title("Querying Bench Over Total Bench", &[])
        .set_x_label("Number of Objects", &[])
        .set_y_label("Spareness of Objects", &[])
        .set_z_label("Query/Total Time", &[Rotate(90.0),TextOffset(-3.0,0.0)])
        .points(x.clone(), y.clone(), z3.clone(), &[PointSymbol('O'), Color("red"), PointSize(0.5)])
        .points(x.clone(), y.clone(), z2.clone(), &[PointSymbol('O'), Color("violet"), PointSize(0.5)]);
                

    fg.show();
}

pub fn handle(fb:&FigureBuilder){
    handle1(fb);
    handle2(fb);
}
fn handle2(fb:&FigureBuilder){

    let mut rects=Vec::new();
    
    struct Record{
        grow:f64,
        bench1:f64,
        bench2:f64,
        comparison:f64
    }
    let num_bots=20000;

    for grow in (0..50).map(|a|0.2+(a as f64)*0.2){
        let s=dists::spiral::Spiral::new([400.0,400.0],17.0,grow);



        //let mut bots:Vec<Bot>=(0..num_bots).map(|a|Bot{num:0}).collect();
        let mut bots:Vec<Bot>=s.take(num_bots).map(|pos|{
            let pos=[pos[0] as isize,pos[1] as isize];
            Bot{num:0,pos}
        }).collect();

        let comparison=test1(&mut bots);
        let bench1=test2(&mut bots);
        let bench2=test3(&mut bots);

        let r=Record{grow,bench1,bench2,comparison};
        rects.push(r);   
    }



    let x=rects.iter().map(|a|a.grow as f64);
    let y1=rects.iter().map(|a|a.bench1);
    let y2=rects.iter().map(|a|a.bench2);
    let y3=rects.iter().map(|a|a.comparison);


    let mut fg= fb.new("colfind_construction_vs_qyery");
    
    fg.axes2d()
        .set_pos_grid(2,1,0)
        .set_title("Querying Bench Over Total Bench with 20000 objects", &[])
        .lines(x.clone(), y1,  &[Caption("Sequential"), Color("blue"), LineWidth(2.0)])
        .lines(x.clone(), y2,  &[Caption("Parallel"), Color("green"), LineWidth(2.0)])
        .set_x_label("Grow", &[])
        .set_y_label("Query/Total Time", &[]);

    fg.axes2d()
        .set_pos_grid(2,1,1)
        .set_title("Querying Comparisons Over Total Comparisons with a 20000 objects", &[])
        .lines(x.clone(), y3,  &[Caption("Sequential"), Color("blue"), LineWidth(2.0)])
        .set_x_label("Grow", &[])
        .set_y_label("Query/Total Time", &[]);

    fg.show();

}
*/