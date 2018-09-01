use support::*;
use dinotree_alg::colfind;
use csv;
use std;
use dinotree_inner::*;
use axgeom;
use spiral::SpiralGenerator;
use data_theory::datanum;
use piston_window;
use DemoSys;

use std::time::Instant;

use std::time::Duration;

#[derive(Copy,Clone)]
pub struct Bot{
    pos:[isize;2],
    num:usize
}
pub struct DataColFind{
    records:Vec<Record>
}


impl DataColFind{
    pub fn new(_dim:[f64;2])->DataColFind{    
        //let wtr = csv::Writer::from_writer(std::io::stdout());
        DataColFind{records:Vec::new()}
    }
}


fn instant_to_sec(elapsed:Duration)->f64{
     (elapsed.as_secs() as f64) + (elapsed.subsec_nanos() as f64 / 1000_000_000.0)           
}



#[derive(Debug, Serialize)]
struct Record {
    num_bots_per_node: usize,
    num_comparison: usize
}

#[derive(Debug, Serialize)]
struct BenchRecord {
    num_bots_per_node: usize,
    bench: f64
}



impl DemoSys for DataColFind{
    fn step(&mut self,_cursor:[f64;2],c:&piston_window::Context,g:&mut piston_window::G2d)->bool{

        let num_bots=10_000;
        let s=SpiralGenerator::new([400.0,400.0],12.0,2.0);


        let mut bots:Vec<Bot>=s.take(num_bots).map(|pos|{
            let pos=[pos[0] as isize,pos[1] as isize];
            Bot{num:0,pos}
        }).collect();
        

        struct Heur{
            num_bots_per_node:usize,
        }

        impl TreeHeightHeur for Heur{
            fn compute_tree_height_heuristic(&self,num_bots:usize)->usize{
                compute_tree_height_heuristic(num_bots,self.num_bots_per_node)
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

            self.records.push(Record{num_bots_per_node:i,num_comparison:c1});
        }

        let mut bench_records:Vec<BenchRecord>=Vec::new();
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
            let rects=&mut self.records;
            use gnuplot::*;
            let x=rects.iter().map(|a|a.num_bots_per_node);
            let y=rects.iter().map(|a|a.num_comparison);

            let mut fg = Figure::new();


            fg.axes2d()
                .set_title("Number of Comparisons with 10,000 objects in a dinotree with different numbers of objects per node", &[])
                .lines(x, y,  &[Color("blue"), LineWidth(2.0)])
                .set_x_label("Number of Objects Per Node", &[])
                .set_y_label("Number of Comparisons", &[]);
    
            fg.show();

        }
        {
            use gnuplot::*;
            let x=bench_records.iter().map(|a|a.num_bots_per_node);
            let y=bench_records.iter().map(|a|a.bench);

            let mut fg = Figure::new();


            fg.axes2d()
                .set_title("Bench times with 10,000 objects in a dinotree with different numbers of objects per node", &[])
                .lines(x, y,  &[Color("blue"), LineWidth(2.0)])
                .set_x_label("Number of Objects Per Node", &[])
                .set_y_label("Time in seconds", &[]);
    
            fg.show();

            return true;
        }
     }
}

