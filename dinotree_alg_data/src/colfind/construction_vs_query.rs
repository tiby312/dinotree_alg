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
    handle_num_bots(fb,0.2);
    handle_num_bots(fb,2.0);
    handle_grow(fb);
}



fn handle_num_bots(fb:&FigureBuilder,grow:f64){
    #[derive(Debug)]
    struct Record {
        num_bots:usize,
        theory: (usize,usize),
        bench:(f64,f64),
        bench_par:(f64,f64)        
    }

    //let grow=0.2;

    let s=dists::spiral::Spiral::new([400.0,400.0],17.0,grow);
    let mut rects=Vec::new();

    for num_bots in (1..80_000).step_by(1000){

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

    let mut fg= fb.new(&format!("colfind_rebal_vs_query_num_bots_grow_of_{}",grow));
    
    fg.axes2d()
        .set_pos_grid(2,1,0)
        .set_title(&format!("Rebal vs Query Comparisons with a spiral grow of {}",grow), &[])
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
        .set_title(&format!("Rebal vs Query Benches with a spiral grow of {}",grow), &[])
        .lines(x.clone(), y1,  &[Caption("Rebal Sequential"), Color("blue"), LineWidth(2.0)])
        .lines(x.clone(), y2,  &[Caption("Query Sequential"), Color("green"), LineWidth(2.0)])
        .lines(x.clone(), y3,  &[Caption("Rebal Parallel"), Color("red"), LineWidth(2.0)])
        .lines(x.clone(), y4,  &[Caption("Query Parallel"), Color("brown"), LineWidth(2.0)])
        .set_x_label("Grow", &[])
        .set_y_label("Time in seconds", &[]);

    fg.show();


}



fn handle_grow(fb:&FigureBuilder){
    let num_bots=80_000;

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
        .set_title("Rebal vs Query Comparisons with 80,000 objects", &[])
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
        .set_title("Rebal vs Query Benches with a 80,000 objects", &[])
        .lines(x.clone(), y1,  &[Caption("Rebal Sequential"), Color("blue"), LineWidth(2.0)])
        .lines(x.clone(), y2,  &[Caption("Query Sequential"), Color("green"), LineWidth(2.0)])
        .lines(x.clone(), y3,  &[Caption("Rebal Parallel"), Color("red"), LineWidth(2.0)])
        .lines(x.clone(), y4,  &[Caption("Query Parallel"), Color("brown"), LineWidth(2.0)])
        .set_x_label("Grow", &[])
        .set_y_label("Time in seconds", &[]);

    fg.show();
    
    

}
