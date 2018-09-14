use inner_prelude::*;


#[derive(Copy,Clone)]
pub struct Bot{
    pos:[isize;2],
    num:usize
}




#[derive(Debug)]
struct Record {
    num_bots_per_node: usize,
    num_comparison: usize
}

#[derive(Debug)]
struct BenchRecord {
    num_bots_per_node: usize,
    bench: f64
}



pub fn handle(fb:&FigureBuilder){

    let mut theory_records=Vec::new();
    let mut bench_records:Vec<BenchRecord>=Vec::new();
    
    let num_bots=10_000;
    let s=dists::spiral::Spiral::new([400.0,400.0],12.0,2.0);


    let mut bots:Vec<Bot>=s.take(num_bots).map(|pos|{
        let pos=[pos[0] as isize,pos[1] as isize];
        Bot{num:0,pos}
    }).collect();
    

    struct Heur{
        num_bots_per_node:usize,
    }

    impl TreeHeightHeur for Heur{
        fn compute_tree_height_heuristic(&self,num_bots:usize)->usize{
            compute_tree_height_heuristic_debug(num_bots,self.num_bots_per_node)
        }
    }

    for i in (1..200){
        let heur=Heur{num_bots_per_node:i};
    
        let c1={
            let mut counter=datanum::Counter::new();


            let mut tree=DynTree::with_debug_seq(axgeom::XAXISS,(),&bots,|b|{
                datanum::from_rect(&mut counter,aabb_from_point_isize(b.pos,[5,5]))  
            },heur).0;

            colfind::query_seq_mut(&mut tree,|a, b| {
                a.inner.num+=2;
                b.inner.num+=2;            
            });
            
            tree.apply_orig_order(&mut bots,|a,b|{
                *b=a.inner;
            });

            counter.into_inner()
        };

        theory_records.push(Record{num_bots_per_node:i,num_comparison:c1});
    }

    for i in (1..200){
        let heur=Heur{num_bots_per_node:i};
    
        let c1={
            
            let instant=Instant::now();
        
            let mut tree=DynTree::with_debug_seq(axgeom::XAXISS,(),&bots,|b|{
                aabb_from_point_isize(b.pos,[5,5]) 
            },heur).0;

            colfind::query_seq_mut(&mut tree,|a, b| {
                a.inner.num+=2;
                b.inner.num+=2;            
            });
            
            tree.apply_orig_order(&mut bots,|a,b|{
                *b=a.inner;
            });
            instant_to_sec(instant.elapsed())

        };

        bench_records.push(BenchRecord{num_bots_per_node:i,bench:c1});
    }

    {
        let rects=&mut theory_records;
        use gnuplot::*;
        let x=rects.iter().map(|a|a.num_bots_per_node);
        let y=rects.iter().map(|a|a.num_comparison);

        let mut fg = fb.new("colfind_height_heuristic");

        fg.axes2d()
            .set_pos_grid(2,1,0)
            .set_title("Number of Comparisons with 10,000 objects in a dinotree with different numbers of objects per node", &[])
            .lines(x, y,  &[Color("blue"), LineWidth(2.0)])
            .set_x_label("Number of Objects Per Node", &[])
            .set_y_label("Number of Comparisons", &[]);

     
    
    
        let x=bench_records.iter().map(|a|a.num_bots_per_node);
        let y=bench_records.iter().map(|a|a.bench);


        fg.axes2d()
            .set_pos_grid(2,1,1)
            .set_title("Bench times with 10,000 objects in a dinotree with different numbers of objects per node", &[])
            .lines(x, y,  &[Color("blue"), LineWidth(2.0)])
            .set_x_label("Number of Objects Per Node", &[])
            .set_y_label("Time in seconds", &[]);

        fg.show();

    }
 
}

