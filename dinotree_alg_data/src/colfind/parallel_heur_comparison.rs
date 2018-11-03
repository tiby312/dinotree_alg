use inner_prelude::*;

#[derive(Copy,Clone)]
pub struct Bot{
    num:usize,
    pos:[isize;2]
}


struct Bo{}
impl colfind::ColMulti for Bo{
    type T=BBox<isize,Bot>;
    fn collide(&mut self,a:&mut Self::T,b:&mut Self::T){
        a.inner.num+=1;
        b.inner.num+=1;
    }
}
impl Splitter for Bo{
    fn div(self)->(Self,Self){
        (Bo{},Bo{})
    }
    fn add(self,a:Self)->Self{
        self
    }
    fn node_start(&mut self){}
    fn node_end(&mut self){}
}






fn test1(bots:&mut [Bot])->(f64,f64){
    
    let instant=Instant::now();

    let mut tree=DinoTree::new_seq(axgeom::XAXISS,(),bots,|b|{
        aabb_from_point_isize(b.pos,[5,5])  
    });


    let a=instant_to_sec(instant.elapsed());
    

    colfind::query_seq_mut(&mut tree,|a, b| {
        a.inner.num+=2;
        b.inner.num+=2;
    });

    tree.apply(bots,|a,b|{
        b.num=a.inner.num;
    });

    let b=instant_to_sec(instant.elapsed());

    return (a,(b-a));
}

fn test3(bots:&mut [Bot],rebal_height:usize,query_height:usize)->(f64,f64){
    
    let instant=Instant::now();

    let height=compute_tree_height_heuristic(bots.len()); 
        
    let (mut tree,_)=dinotree::advanced::new_adv(axgeom::XAXISS,(),bots,|b|{
        aabb_from_point_isize(b.pos,[5,5])  
    },height,SplitterEmpty,rebal_height);


    let a=instant_to_sec(instant.elapsed());
    

    let _ =colfind::query_adv_mut(&mut tree,Bo{},SplitterEmpty,query_height);

    tree.apply(bots,|a,b|{
        b.num=a.inner.num;
    });

    let b=instant_to_sec(instant.elapsed());

    return (a,(b-a));
}


pub fn handle(fb:&FigureBuilder){

    let num_bots=20_000;
    let grow=0.2;

    let s=dists::spiral::Spiral::new([400.0,400.0],17.0,grow);
    

    let mut bots:Vec<Bot>=s.clone().take(num_bots).map(|pos|{
        let pos=[pos[0] as isize,pos[1] as isize];
        Bot{num:0,pos}
    }).collect();


    let height=compute_tree_height_heuristic(num_bots);
    
    let mut rebals=Vec::new();
    for rebal_height in (0..height).flat_map(|a|std::iter::repeat(a).take(16)){
        let (a,b)=test3(&mut bots,rebal_height,4);
        rebals.push((rebal_height,a));    
    }

    let mut queries=Vec::new();
    for query_height in (0..height).flat_map(|a|std::iter::repeat(a).take(16)){
        let (a,b)=test3(&mut bots,4,query_height);
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

    let mut fg= fb.new("parallel_height_heuristic");
    
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

    fg.show();

}