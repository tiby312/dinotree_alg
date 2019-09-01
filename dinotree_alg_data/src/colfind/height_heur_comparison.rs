use crate::inner_prelude::*;

#[derive(Copy,Clone)]
pub struct Bot{
    pos:Vec2<isize>,
    num:usize
}




pub fn handle_bench_inner(bots:&mut [Bot],height:usize)->f64{

    
    let instant=Instant::now();

    let func=|b:&Bot|{axgeom::Rect::from_point(b.pos,vec2same(5))};
    let mut tree=DinoTreeBuilder::new(axgeom::XAXISS,bots,func).with_height(height).build_seq();

    colfind::QueryBuilder::new(&mut tree).query_seq(|mut a,mut b| {
        a.inner_mut().num+=2;
        b.inner_mut().num+=2;            
    });

    instant_to_sec(instant.elapsed())

}


pub fn handle_theory_inner(bots:&mut [Bot],height:usize)->usize{
    
    
    let mut counter=datanum::Counter::new();



    let func=|b:&Bot|{datanum::from_rect(&mut counter,axgeom::Rect::from_point(b.pos,vec2same(5)))};
    let mut tree=DinoTreeBuilder::new(axgeom::XAXISS,bots,func).with_height(height).build_seq();

    colfind::QueryBuilder::new(&mut tree).query_seq(|mut a,mut b| {
        a.inner_mut().num+=2;
        b.inner_mut().num+=2;            
    });
    

    counter.into_inner()

}

pub fn create_bots(num_bots:usize)->Vec<Bot>{
    let s=dists::spiral::Spiral::new([400.0,400.0],12.0,2.0);

    s.take(num_bots).map(|pos|{
        Bot{num:0,pos:pos.inner_as()}
    }).collect()    
}


pub fn handle(fb:&mut FigureBuilder){
    handle2d(fb);
    //handle3d(fb);
    handle_lowest(fb);
}


fn handle_lowest(fb:&mut FigureBuilder){

    struct BenchRecord{
        height:usize,
        num_bots:usize,
    }

    struct TheoryRecord{
        height:usize,
        num_bots:usize
    }

    let mut benches:Vec<BenchRecord>=Vec::new();
    let mut theories:Vec<TheoryRecord>=Vec::new();

    let its=(1usize..80_000).step_by(2000);
    for num_bots in its.clone(){
        let mut minimum=None;//(100000.0,0);
        let mut minimum_theory=None;
        let max_height=(num_bots as f64).log2() as usize;

        let mut bots=create_bots(num_bots);
        for height in 1..max_height{
            let theory=handle_theory_inner(&mut bots,height);
            let bench=handle_bench_inner(&mut bots,height);
            match minimum{
                Some((a,_b))=>{
                    if bench<a{
                        minimum=Some((bench,height));
                    }
                },
                None=>{
                    minimum=Some((bench,height));
                }
            }
            match minimum_theory{
                Some((a,_b))=>{
                    if theory<a{
                        minimum_theory=Some((theory,height));
                    }
                },
                None=>{
                    minimum_theory=Some((theory,height));
                }
            }
        }

        if let Some((_,height))=minimum_theory{
            theories.push(TheoryRecord{height,num_bots});            
            let (_,height)=minimum.unwrap();
            benches.push(BenchRecord{height,num_bots});
        }


    }


    {
        let mut fg = fb.build("height_heuristic_vs_optimal");

        let xx=theories.iter().map(|a|a.num_bots);
        let yy=theories.iter().map(|a|a.height);
        
        let x=benches.iter().map(|a|a.num_bots);
        let y=benches.iter().map(|a|a.height);


        let heur={
            let mut vec=Vec::new();
            for num_bots in its.clone(){
                let height=compute_tree_height_heuristic(num_bots);
                vec.push((num_bots,height));
            }
            vec
        };

        let heurx=heur.iter().map(|a|a.0);
        let heury=heur.iter().map(|a|a.1);

        fg.axes2d()
            .set_pos_grid(2,1,0)
            .set_legend(Graph(1.0),Graph(0.0),&[LegendOption::Placement(AlignRight,AlignBottom)],&[])
            .set_title("Dinotree Colfind Query: Optimal Height vs Heuristic Height with abspiral(x,2.0)", &[])
            .set_x_label("Num bots", &[])
            .set_y_label("Best Tree Height", &[])
            .points(xx, yy,  &[Caption("Optimal"),PointSymbol('O'), Color("red"), PointSize(1.0)])
            .points(heurx.clone(),heury.clone(),&[Caption("Heuristic"),PointSymbol('x'), Color("blue"), PointSize(2.0)]);

        fg.axes2d()
            .set_pos_grid(2,1,1)
            .set_legend(Graph(1.0),Graph(0.0),&[LegendOption::Placement(AlignRight,AlignBottom)],&[])
            .set_title("Dinotree Colfind Query: Optimal Height vs Heuristic Height with abspiral(x,2.0)", &[])
            .set_x_label("Num bots", &[])
            .set_y_label("Best Tree Height", &[])
            .points(x, y,  &[Caption("Optimal"),PointSymbol('O'), Color("red"), PointSize(1.0)])
            .points(heurx,heury,&[Caption("Heuristic"),PointSymbol('x'), Color("blue"), PointSize(2.0)]);


        fb.finish(fg);
        
    }
}

fn handle2d(fb:&mut FigureBuilder){



    #[derive(Debug)]
    struct Record {
        height: usize,
        num_comparison: usize
    }

    #[derive(Debug)]
    struct BenchRecord {
        height: usize,
        bench: f64
    }

    let mut theory_records=Vec::new();
    let mut bench_records:Vec<BenchRecord>=Vec::new();
    

    let mut bots=create_bots(10_000);



    for height in 2..13{
        let num_comparison=handle_theory_inner(&mut bots,height);
        theory_records.push(Record{height,num_comparison});
    }

    for height in (2..13).flat_map(|a|std::iter::repeat(a).take(20)){
        let bench=handle_bench_inner(&mut bots,height);
        bench_records.push(BenchRecord{height,bench});
    }



    let rects=&mut theory_records;
    use gnuplot::*;
    let x=rects.iter().map(|a|a.height);
    let y=rects.iter().map(|a|a.num_comparison);

    let mut fg = fb.build("height_heuristic");

    fg.axes2d()
        .set_pos_grid(2,1,0)
        .set_title("Number of Comparisons with different numbers of objects per node with abspiral(10000,2.0)", &[])
        .lines(x, y,  &[Color("blue"), LineWidth(2.0)])
        .set_x_label("Tree Height", &[])
        .set_y_label("Number of Comparisons", &[]);

 


    let x=bench_records.iter().map(|a|a.height);
    let y=bench_records.iter().map(|a|a.bench);


    fg.axes2d()
        .set_pos_grid(2,1,1)
        .set_title("Bench times with different numbers of objects per node (seq,colfind) with abspiral(10000,2.0)", &[])
        .points(x,y,&[Color("blue"), LineWidth(2.0)])
        .set_x_label("Tree Height", &[])
        .set_y_label("Time in seconds", &[]);
    
    fb.finish(fg);


}

