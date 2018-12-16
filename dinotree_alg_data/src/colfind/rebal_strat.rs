use crate::inner_prelude::*;



#[derive(Copy,Clone)]
pub struct Bot{
    num:usize,
    pos:[isize;2]
}




fn test1(bots:&mut [Bot])->f64{
    
    let instant=Instant::now();

    
    let mut tree=dinotree::DinoTreeBuilder::new(axgeom::XAXISS,(),bots,|b|{
        aabb_from_point_isize(b.pos,[5,5])  
    }).with_rebal_strat(RebalStrat::First).build_par();
    

    black_box(tree);

    let a=instant_to_sec(instant.elapsed());
    a
}


fn test2(bots:&mut [Bot])->f64{
    
    let instant=Instant::now();

   
    let mut tree=dinotree::DinoTreeBuilder::new(axgeom::XAXISS,(),bots,|b|{
        aabb_from_point_isize(b.pos,[5,5])  
    }).with_rebal_strat(RebalStrat::Second).build_par();
    
    
    black_box(tree);

    let a=instant_to_sec(instant.elapsed());
    a
}



fn test3(bots:&mut [Bot])->f64{
    
    let instant=Instant::now();

   
    let mut tree=dinotree::DinoTreeBuilder::new(axgeom::XAXISS,(),bots,|b|{
        aabb_from_point_isize(b.pos,[5,5])  
    }).with_rebal_strat(RebalStrat::First).build_seq();
    
    
    black_box(tree);

    let a=instant_to_sec(instant.elapsed());
    a
}



fn test4(bots:&mut [Bot])->f64{
    
    let instant=Instant::now();

   
    let mut tree=dinotree::DinoTreeBuilder::new(axgeom::XAXISS,(),bots,|b|{
        aabb_from_point_isize(b.pos,[5,5])  
    }).with_rebal_strat(RebalStrat::Second).build_seq();
    
    

    black_box(tree);

    let a=instant_to_sec(instant.elapsed());
    a
}



fn test5(bots:&mut [Bot])->f64{
    
    let instant=Instant::now();

    let mut tree=dinotree::DinoTreeBuilder::new(axgeom::XAXISS,(),bots,|b|{
        aabb_from_point_isize(b.pos,[5,5])  
    }).with_rebal_strat(RebalStrat::Third).build_par();
    
    

    black_box(tree);

    let a=instant_to_sec(instant.elapsed());
    a
}





fn test6(bots:&mut [Bot])->f64{
    
    let instant=Instant::now();

    let mut tree=dinotree::DinoTreeBuilder::new(axgeom::XAXISS,(),bots,|b|{
        aabb_from_point_isize(b.pos,[5,5])  
    }).with_rebal_strat(RebalStrat::Third).build_seq();
    
    

    black_box(tree);

    let a=instant_to_sec(instant.elapsed());
    a
}






pub fn handle(fb:&mut FigureBuilder){ 
    handle_num_bots(fb,1.0);
}


#[derive(Debug)]
struct Record {
    num_bots:usize,
    arr:[f64;6]    
}
impl Record{
    fn draw(records:&[Record],fg:&mut Figure){
        const NAMES:[&'static str;6]=["RebalStrat1","RebalStrat2","RebalStrat1 Seq","RebalStrat2 Seq","RebalStrat3","RebalStrat3 Seq"];
        {
            let k=fg.axes2d()
                .set_title(&format!("Rebal vs Query Comparisons with a spiral grow of 1"), &[])
                .set_legend(Graph(1.0),Graph(1.0),&[LegendOption::Horizontal],&[])
                .set_x_label("Number of Elements", &[])
                .set_y_label("Number of Comparisons", &[]);

            let x=records.iter().map(|a|a.num_bots);
            for index in 0..6{
                let y=records.iter().map(|a|a.arr[index]);
                k.lines(x.clone(),y,&[Caption(NAMES[index]),Color(COLS[index]),LineWidth(2.0)]);
            }
        }
    }
}

fn handle_num_bots(fb:&mut FigureBuilder,grow:f64){
    
    let s=dists::spiral::Spiral::new([400.0,400.0],17.0,grow);
    let mut rects=Vec::new();

    for num_bots in (0..2000_000).step_by(20000){

        let mut bots:Vec<Bot>=s.clone().take(num_bots).map(|pos|{
            let pos=[pos[0] as isize,pos[1] as isize];
            Bot{num:0,pos}
        }).collect();

        let a=test1(&mut bots);
        let b=test2(&mut bots);
        let c=test3(&mut bots);
        let d=test4(&mut bots);
        let e=test5(&mut bots);
        let f=test6(&mut bots);

        let r=Record{num_bots,arr:[a,b,c,d,e,f]};
        rects.push(r);      
    }

    let mut fg= fb.new(&format!("colfind_rebal_vs_query_num_bots_grow_of_{}",grow));
    
    Record::draw(&rects,&mut fg);
    
    fb.finish(fg);
}
