use crate::inner_prelude::*;

#[derive(Copy,Clone)]
pub struct Bot{
    num:usize,
    pos:Vec2<isize>
}


fn test1(bots:&mut [Bot])->(f64,f64){
    
    let instant=Instant::now();

    let mut bb=create_bbox_mut(bots,|b|{
        axgeom::Rect::from_point(b.pos,vec2same(5))  
    });

    let mut tree=DinoTreeBuilder::new(axgeom::XAXISS,&mut bb).build_seq();


    let a=instant_to_sec(instant.elapsed());
    

    colfind::QueryBuilder::new(&mut tree).query_seq(|mut a,mut b| {
        a.inner_mut().num+=2;
        b.inner_mut().num+=2;
    });

    let b=instant_to_sec(instant.elapsed());

    (a,(b-a))
}

fn test3(bots:&mut [Bot],rebal_height:usize,query_height:usize)->(f64,f64){
    
    let instant=Instant::now();

    let mut bb=create_bbox_mut(bots,|b|{
        axgeom::Rect::from_point(b.pos,vec2same(5))  
    });
    let mut tree=DinoTreeBuilder::new(axgeom::XAXISS,&mut bb).with_height_switch_seq(rebal_height).build_par();
    
    let a=instant_to_sec(instant.elapsed());
    
    colfind::QueryBuilder::new(&mut tree).with_switch_height(query_height).query_par(|mut a,mut b|{
        a.inner_mut().num+=1;
        b.inner_mut().num+=1;
    });

    let b=instant_to_sec(instant.elapsed());

    (a,(b-a))
}


pub fn handle(fb:&mut FigureBuilder){

    let num_bots=20_000;
    let grow=0.2;

    let s=dists::spiral::Spiral::new([400.0,400.0],17.0,grow);
    

    let mut bots:Vec<Bot>=s.clone().take(num_bots).map(|pos|{
        Bot{num:0,pos:pos.inner_as()}
    }).collect();


    let height=compute_tree_height_heuristic(num_bots);
    
    let mut rebals=Vec::new();
    for rebal_height in (0..height).flat_map(|a|std::iter::repeat(a).take(16)){
        let (a,_b)=test3(&mut bots,rebal_height,4);
        rebals.push((rebal_height,a));    
    }

    let mut queries=Vec::new();
    for query_height in (0..height).flat_map(|a|std::iter::repeat(a).take(16)){
        let (_a,b)=test3(&mut bots,4,query_height);
        queries.push((query_height,b));    
    }


    let x1=rebals.iter().map(|a|a.0);
    let y1=rebals.iter().map(|a|a.1);
    let x2=queries.iter().map(|a|a.0);
    let y2=queries.iter().map(|a|a.1);


    let mut seqs=Vec::new();
    for _ in 0..100{
        let (a,b)=test1(&mut bots);
        seqs.push((a,b));
    }
    let xx=seqs.iter().map(|_|height-1);
    let yy1=seqs.iter().map(|a|a.0);
    let yy2=seqs.iter().map(|a|a.1);

    let mut fg= fb.build("parallel_height_heuristic");
    
    fg.axes2d()
        //.set_pos_grid(2,1,0)
        .set_title("Parallel Height heuristic for 20,000 elements with a spiral grow of 0.2", &[])
        .points(x1.clone(), y1,  &[Caption("Rebalance"), Color("brown"), LineWidth(4.0)])
        .points(x2.clone(), y2,  &[Caption("Query"), Color("red"), LineWidth(4.0)])
        
        .points(xx.clone(), yy1,  &[Caption("Rebalance Sequential"), Color("green"), LineWidth(4.0)])
        .points(xx.clone(), yy2,  &[Caption("Query Sequential"), Color("blue"), LineWidth(4.0)])
        

        .set_x_label("Height at which to switch to sequential", &[])
        .set_y_label("Time in seconds", &[])
        .set_x_grid(true)
        .set_y_grid(true);

    fb.finish(fg);

}