use inner_prelude::*;



#[derive(Copy,Clone)]
pub struct Bot{
    num:usize,
    pos:[isize;2]
}




fn test1(bots:&mut [Bot])->f64{
    
    let instant=Instant::now();

   
    let mut tree=advanced::new_adv(RebalStrat1,axgeom::XAXISS,(),bots,|b|{
        aabb_from_point_isize(b.pos,[5,5])  
    },None,&mut SplitterEmpty,None);

    //black_box(tree);

    let a=instant_to_sec(instant.elapsed());
    a
}


fn test2(bots:&mut [Bot])->f64{
    
    let instant=Instant::now();

   
    let mut tree=advanced::new_adv(RebalStrat2,axgeom::XAXISS,(),bots,|b|{
        aabb_from_point_isize(b.pos,[5,5])  
    },None,&mut SplitterEmpty,None);

    //black_box(tree);

    let a=instant_to_sec(instant.elapsed());
    a
}



fn test3(bots:&mut [Bot])->f64{
    
    let instant=Instant::now();

   
    let mut tree=advanced::new_adv_seq(RebalStrat1,axgeom::XAXISS,(),bots,|b|{
       aabb_from_point_isize(b.pos,[5,5])  
    },None,&mut SplitterEmpty);

    //black_box(tree);

    let a=instant_to_sec(instant.elapsed());
    a
}



fn test4(bots:&mut [Bot])->f64{
    
    let instant=Instant::now();

   
    let mut tree=advanced::new_adv_seq(RebalStrat2,axgeom::XAXISS,(),bots,|b|{
        aabb_from_point_isize(b.pos,[5,5])  
    },None,&mut SplitterEmpty);

    //black_box(tree);

    let a=instant_to_sec(instant.elapsed());
    a
}




pub fn handle(fb:&mut FigureBuilder){
    
    handle_num_bots(fb,1.0);
}



fn handle_num_bots(fb:&mut FigureBuilder,grow:f64){
    #[derive(Debug)]
    struct Record {
        num_bots:usize,
        arr:[f64;4]    
    }

    //let grow=0.2;

    let s=dists::spiral::Spiral::new([400.0,400.0],17.0,grow);
    let mut rects=Vec::new();

    for num_bots in (1..500_000).step_by(1000){

        let mut bots:Vec<Bot>=s.clone().take(num_bots).map(|pos|{
            let pos=[pos[0] as isize,pos[1] as isize];
            Bot{num:0,pos}
        }).collect();
    

        let a=test1(&mut bots);
        let b=test2(&mut bots);
        let c=test3(&mut bots);
        let d=test4(&mut bots);

        let r=Record{num_bots,arr:[a,b,c,d]};
        rects.push(r);      
    }


    let x=rects.iter().map(|a|a.num_bots);
    let y1=rects.iter().map(|a|a.arr[0]);
    let y2=rects.iter().map(|a|a.arr[1]);
    let y3=rects.iter().map(|a|a.arr[2]);
    let y4=rects.iter().map(|a|a.arr[3]);
    
    let mut fg= fb.new(&format!("colfind_rebal_vs_query_num_bots_grow_of_{}",grow));
    
    fg.axes2d()
        .set_pos_grid(2,1,0)
        .set_title(&format!("Rebal vs Query Comparisons with a spiral grow of {}",grow), &[])
        .lines(x.clone(), y1,  &[Caption("test1"), Color("blue"), LineWidth(2.0)])
        .lines(x.clone(), y2,  &[Caption("test2"), Color("green"), LineWidth(2.0)])
        .lines(x.clone(), y3,  &[Caption("test3"), Color("red"), LineWidth(2.0)])
        .lines(x.clone(), y4,  &[Caption("test4"), Color("brown"), LineWidth(2.0)])
        .set_x_label("Number of Elements", &[])
        .set_y_label("Number of Comparisons", &[]);


    fb.finish(fg);


}
