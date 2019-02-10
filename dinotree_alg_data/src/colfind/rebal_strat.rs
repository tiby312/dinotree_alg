use crate::inner_prelude::*;


#[derive(Copy,Clone)]
pub struct Bot{
    _num:usize,
    pos:[isize;2]
}




fn test1(bots:&mut [Bot])->f64{
    
    let instant=Instant::now();

    
    let tree=dinotree::DinoTreeBuilder::new(axgeom::XAXISS,bots,|b|{
        aabb_from_point_isize(b.pos,[5,5])  
    }).with_bin_strat(BinStrat::Checked).build_par();
    

    black_box(tree);

    instant_to_sec(instant.elapsed())
}


fn test2(bots:&mut [Bot])->f64{
    
    let instant=Instant::now();

   
    let tree=dinotree::DinoTreeBuilder::new(axgeom::XAXISS,bots,|b|{
        aabb_from_point_isize(b.pos,[5,5])  
    }).with_bin_strat(BinStrat::NotChecked).build_par();
    
    
    black_box(tree);

    instant_to_sec(instant.elapsed())
}



fn test3(bots:&mut [Bot])->f64{
    
    let instant=Instant::now();

   
    let tree=dinotree::DinoTreeBuilder::new(axgeom::XAXISS,bots,|b|{
        aabb_from_point_isize(b.pos,[5,5])  
    }).with_bin_strat(BinStrat::Checked).build_seq();
    
    
    black_box(tree);

    instant_to_sec(instant.elapsed())
}



fn test4(bots:&mut [Bot])->f64{
    
    let instant=Instant::now();

   
    let tree=dinotree::DinoTreeBuilder::new(axgeom::XAXISS,bots,|b|{
        aabb_from_point_isize(b.pos,[5,5])  
    }).with_bin_strat(BinStrat::NotChecked).build_seq();
    
    

    black_box(tree);

    instant_to_sec(instant.elapsed())
}




pub fn handle(fb:&mut FigureBuilder){ 
    handle_num_bots(fb,1.0);
}


#[derive(Debug)]
struct Record {
    num_bots:usize,
    arr:[f64;4]    
}
impl Record{
    fn draw(records:&[Record],fg:&mut Figure){
        const NAMES:&[&str]=&["RebalStrat1","RebalStrat2","RebalStrat1 Seq","RebalStrat2 Seq"];
        {
            let k=fg.axes2d()
                .set_title(&"Rebal vs Query Comparisons with a spiral grow of 1".to_string(), &[])
                .set_legend(Graph(1.0),Graph(1.0),&[LegendOption::Horizontal],&[])
                .set_x_label("Number of Elements", &[])
                .set_y_label("Number of Comparisons", &[]);

            let x=records.iter().map(|a|a.num_bots);
            for index in 0..4{
                let y=records.iter().map(|a|a.arr[index]);
                k.lines(x.clone(),y,&[Caption(NAMES[index]),Color(COLS[index]),LineWidth(2.0)]);
            }
        }
    }
}

fn handle_num_bots(fb:&mut FigureBuilder,grow:f64){
    
    let s=dists::spiral::Spiral::new([400.0,400.0],17.0,grow);
    let mut rects=Vec::new();

    for num_bots in (0..1_000_000).step_by(20000){

        let mut bots:Vec<Bot>=s.clone().take(num_bots).map(|pos|{
            let pos=[pos[0] as isize,pos[1] as isize];
            Bot{_num:0,pos}
        }).collect();

        let arr=[test1(&mut bots),
                test2(&mut bots),
                test3(&mut bots),
                test4(&mut bots)];

        let r=Record{num_bots,arr};
        rects.push(r);      
    }

    let mut fg= fb.build(&format!("colfind_rebal_vs_query_num_bots_grow_of_{}",grow));
    
    Record::draw(&rects,&mut fg);
    
    fb.finish(fg);
}
