use inner_prelude::*;


#[derive(Copy,Clone)]
pub struct Bot{
    pos:[isize;2],
    num:usize
}




pub fn handle_bench_inner(bots:&mut [Bot],height:usize)->f64{

    struct Heur{
        //num_bots_per_node:usize,
        height:usize
    }

    impl TreeHeightHeur for Heur{
        //Always return the height specified. This is meant to be used only for this test.
        fn compute_tree_height_heuristic(&self,num_bots:usize)->usize{
            self.height
            //compute_tree_height_heuristic_debug(num_bots,self.num_bots_per_node)
        }
    }

    let heur=Heur{height};

    let c1={
        
        let instant=Instant::now();
    
        let mut tree=DynTree::with_debug_seq(axgeom::XAXISS,(),bots,|b|{
            aabb_from_point_isize(b.pos,[5,5]) 
        },heur).0;

        colfind::query_seq_mut(&mut tree,|a, b| {
            a.inner.num+=2;
            b.inner.num+=2;            
        });
        
        tree.apply_orig_order(bots,|a,b|{
            *b=a.inner;
        });
        instant_to_sec(instant.elapsed())

    };

    c1
}


pub fn handle_theory_inner(bots:&mut [Bot],height:usize)->usize{
    
    struct Heur{
        //num_bots_per_node:usize,
        height:usize
    }

    impl TreeHeightHeur for Heur{
        //Always return the height specified. This is meant to be used only for this test.
        fn compute_tree_height_heuristic(&self,num_bots:usize)->usize{
            self.height
            //compute_tree_height_heuristic_debug(num_bots,self.num_bots_per_node)
        }
    }

    let heur=Heur{height};
    
    let c1={
        let mut counter=datanum::Counter::new();


        let mut tree=DynTree::with_debug_seq(axgeom::XAXISS,(),bots,|b|{
            datanum::from_rect(&mut counter,aabb_from_point_isize(b.pos,[5,5]))  
        },heur).0;

        colfind::query_seq_mut(&mut tree,|a, b| {
            a.inner.num+=2;
            b.inner.num+=2;            
        });
        
        tree.apply_orig_order(bots,|a,b|{
            *b=a.inner;
        });

        counter.into_inner()
    };

    c1
}

pub fn create_bots(num_bots:usize)->Vec<Bot>{
    let s=dists::spiral::Spiral::new([400.0,400.0],12.0,2.0);


    let bots:Vec<Bot>=s.take(num_bots).map(|pos|{
        let pos=[pos[0] as isize,pos[1] as isize];
        Bot{num:0,pos}
    }).collect();
    
    bots
}


pub fn handle(fb:&FigureBuilder){
    //handle2d(fb);
    //handle3d(fb);
    handle_lowest(fb);
}

fn handle3d(fb:&FigureBuilder){

    struct BenchRecord{
        height:usize,
        num_bots:usize,
        bench:f64
    }
    let mut benches:Vec<BenchRecord>=Vec::new();
    for num_bots in (5_000usize..20_000).step_by(100){
        let max_height=(num_bots as f64).log2() as usize;

        let mut bots=create_bots(num_bots);
        for height in (3..max_height-2){
            let bench=handle_bench_inner(&mut bots,height);
            benches.push(BenchRecord{height,num_bots,bench});
        }
    }

    let x=benches.iter().map(|a|a.height);
    let y=benches.iter().map(|a|a.num_bots);
    let z=benches.iter().map(|a|a.bench);

    let mut fg = fb.new("colfind_height_heuristic_3d");

    fg.axes3d().set_view(80.0,360.0-15.0)
        .set_title("Dinotree Colfind query bench times", &[])
        .set_x_label("Tree Height", &[])
        .set_y_label("Number of Bots", &[])
        .set_z_label("Time Taken in seconds", &[Rotate(90.0),TextOffset(-3.0,0.0)])
        .points(x.clone(), y.clone(), z.clone(), &[Caption("Dinotree"),PointSymbol('O'), Color("violet"), PointSize(0.4)]);



    fg.show();
}

fn handle_lowest(fb:&FigureBuilder){

    struct BenchRecord{
        height:usize,
        num_bots:usize,
    }
    let mut benches:Vec<BenchRecord>=Vec::new();

    let its=(1usize..20_000).step_by(1000);
    for num_bots in its.clone(){
        let mut minimum=None;//(100000.0,0);
        let max_height=(num_bots as f64).log2() as usize;

        let mut bots=create_bots(num_bots);
        for height in (1..max_height){
            let bench=handle_bench_inner(&mut bots,height);
            match minimum{
                Some((a,b))=>{
                    if bench<a{
                        minimum=Some((bench,height));
                        //(bench,height)
                    }
                },
                None=>{
                    minimum=Some((bench,height));
                }
            }
        }
        match minimum{
            Some((min,height))=>{
                benches.push(BenchRecord{height,num_bots});
            },
            None=>{}
        }
    }

    let x=benches.iter().map(|a|a.num_bots);
    let y=benches.iter().map(|a|a.height);

    let mut fg = fb.new("colfind_height_heuristic_3d");



    let heur={
        let mut vec=Vec::new();
        for num_bots in its{
            let height=compute_tree_height_heuristic_debug(num_bots,220);
            vec.push((num_bots,height));
        }
        vec
    };

    let heurx=heur.iter().map(|a|a.0);
    let heury=heur.iter().map(|a|a.1);

    fg.axes2d()
        .set_title("Dinotree Colfind query bench times", &[])
        .set_x_label("Num bots", &[])
        .set_y_label("Best Tree Height", &[])
        .points(x, y,  &[Caption("Dinotree"),PointSymbol('O'), Color("violet"), PointSize(1.0)])
        .lines(heurx,heury,&[Caption("Heuristic")]);


    fg.show();
}

fn handle2d(fb:&FigureBuilder){



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



    for height in (2..10){
        let num_comparison=handle_theory_inner(&mut bots,height);
        theory_records.push(Record{height,num_comparison});
    }

    for height in (2..10).flat_map(|a|std::iter::repeat(a).take(5)){
        let bench=handle_bench_inner(&mut bots,height);
        bench_records.push(BenchRecord{height,bench});
    }



    let rects=&mut theory_records;
    use gnuplot::*;
    let x=rects.iter().map(|a|a.height);
    let y=rects.iter().map(|a|a.num_comparison);

    let mut fg = fb.new("colfind_height_heuristic");

    fg.axes2d()
        .set_pos_grid(2,1,0)
        .set_title("Number of Comparisons with 10,000 objects in a dinotree with different numbers of objects per node", &[])
        .lines(x, y,  &[Color("blue"), LineWidth(2.0)])
        .set_x_label("Tree Height", &[])
        .set_y_label("Number of Comparisons", &[]);

 


    let x=bench_records.iter().map(|a|a.height);
    let y=bench_records.iter().map(|a|a.bench);


    fg.axes2d()
        .set_pos_grid(2,1,1)
        .set_title("Bench times with 10,000 objects in a dinotree with different numbers of objects per node (seq,colfind)", &[])
        .lines(x,y,&[Color("blue"), LineWidth(2.0)])
        .set_x_label("Tree Height", &[])
        .set_y_label("Time in seconds", &[]);
    fg.show();


}

