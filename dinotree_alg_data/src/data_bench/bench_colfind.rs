use support::*;
use dinotree_alg::colfind;
use csv;
use std;

use std::time::Instant;
use std::time::Duration;
use piston_window;
use axgeom;
use dinotree_inner::*;
use DemoSys;
use spiral::SpiralGenerator;

#[derive(Copy,Clone)]
pub struct Bot{
    num:isize,
    pos:[isize;2]
}

pub struct DataColFind{
    num_bots:usize,
    wtr:csv::Writer<std::io::Stdout>
}


impl DataColFind{
    pub fn new(_dim:[f64;2])->DataColFind{    
        let wtr = csv::Writer::from_writer(std::io::stdout());
        DataColFind{num_bots:0,wtr}
    }
} 


fn instant_to_sec(elapsed:Duration)->f64{
     (elapsed.as_secs() as f64) + (elapsed.subsec_nanos() as f64 / 1000_000_000.0)
           
}
impl DemoSys for DataColFind{
    fn step(&mut self,_cursor:[f64;2],c:&piston_window::Context,g:&mut piston_window::G2d)->bool{
       
        let mut s=SpiralGenerator::new([400.0,400.0],12.0,2.0);

  
        let mut bots:Vec<Bot>=s.take(self.num_bots).enumerate().map(|(e,pos)|{
            let pos=[pos[0] as isize,pos[1] as isize];
            Bot{num:0,pos}
        }).collect();
        

        if self.num_bots>20000{
            return true;
        }
        if self.num_bots<2000{
            for bot in bots.iter(){
                let r=aabb_from_point_isize(bot.pos,[5,5]);
                draw_rect_isize([0.0,0.0,0.0,0.3],&r,c,g);
            }  
        }

        let c0={
            let instant=Instant::now();
            
            let mut tree=DynTree::new(axgeom::XAXISS,(),&bots,|b|{   
                aabb_from_point_isize(b.pos,[5,5])
            });

            colfind::query_mut(&mut tree,|a, b| {
                a.inner.num+=1;
                b.inner.num+=1;
        
            });

            tree.apply_orig_order(&mut bots,|a,b|{
                b.num=a.inner.num;
            });

            instant_to_sec(instant.elapsed())
        };

        let c1={
            let instant=Instant::now();

            let mut tree=DynTree::new(axgeom::XAXISS,(),&bots,|b|{   
                aabb_from_point_isize(b.pos,[5,5])
            });

            colfind::query_mut(&mut tree,|a, b| {
                a.inner.num+=1;
                b.inner.num+=1;
            });

            tree.apply_orig_order(&mut bots,|a,b|{
                b.num=a.inner.num;
            });

            instant_to_sec(instant.elapsed())
        };
        
        let c3={
            let mut bb:Vec<BBoxDemo<isize,Bot>>=bots.iter().map(|b|{
                BBoxDemo::new(aabb_from_point_isize(b.pos,[5,5]),*b)
            }).collect();


            let instant=Instant::now();
            
            colfind::query_sweep_mut(axgeom::XAXISS,&mut bb,|a,b|{
                a.inner.num-=2;
                b.inner.num-=2;
            });

            for b in bb.iter(){
                assert_eq!(b.inner.num,0);
            } 

            instant_to_sec(instant.elapsed())
        };

        #[derive(Debug, Serialize)]
        struct Record {
            num_bots: usize,
            bench_alg: f64,
            bench_par:f64,
            bench_sweep:f64
        }

        self.wtr.serialize(Record{num_bots:self.num_bots,bench_alg:c1,bench_par:c0,bench_sweep:c3});
        
        self.num_bots+=200;

        false
     }
}

