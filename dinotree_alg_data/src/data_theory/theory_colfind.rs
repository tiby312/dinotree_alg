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


#[derive(Copy,Clone)]
pub struct Bot{
    pos:[isize;2],
    num:usize
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




impl DemoSys for DataColFind{
    fn step(&mut self,_cursor:[f64;2],c:&piston_window::Context,g:&mut piston_window::G2d)->bool{

        let s=SpiralGenerator::new([400.0,400.0],12.0,2.0);

        if self.num_bots>2000{
            return true;
        }


        let mut bots:Vec<Bot>=s.take(self.num_bots).map(|pos|{
            let pos=[pos[0] as isize,pos[1] as isize];
            Bot{num:0,pos}
        }).collect();
        

        let c1={
            let mut counter=datanum::Counter::new();


            let mut tree=DynTree::new_seq(axgeom::XAXISS,(),&bots,|b|{
                datanum::from_rect(&mut counter,aabb_from_point_isize(b.pos,[5,5]))  
            });

            for bot in tree.iter_every_bot(){
                let a=datanum::into_rect(*bot.get());
                draw_rect_isize([0.0,0.0,0.0,0.3],&a,c,g);
            }


            colfind::query_seq_mut(&mut tree,|a, b| {
                a.inner.num+=2;
                b.inner.num+=2;
                let a=datanum::into_rect(*a.get());
                let b=datanum::into_rect(*b.get());
                draw_rect_isize([1.0,0.0,0.0,0.2],&a,c,g);
                draw_rect_isize([1.0,0.0,0.0,0.2],&b,c,g);
        
            });
            
            tree.apply_orig_order(&mut bots,|a,b|{
                *b=a.inner;
            });

            counter.into_inner()
        };
       
        let c2={
            
            let mut counter=datanum::Counter::new();
            
            if self.num_bots<400{
                let mut bb:Vec<BBoxDemo<datanum::DataNum,Bot>>=bots.iter().map(|b|{
                    let rect=aabb_from_point_isize(b.pos,[5,5]);
                    BBoxDemo::new(datanum::from_rect(&mut counter,rect),*b)
                }).collect();

                colfind::query_naive_mut(&mut bb,|a,b|{
                    a.inner.num-=1;
                    b.inner.num-=1;
                });


                for (a,b) in bb.iter().zip(bots.iter_mut()){
                    *b=a.inner;
                }
            }
            
            counter.into_inner()
        };
        let c3={
            let mut counter=datanum::Counter::new();
            let mut bb:Vec<BBoxDemo<datanum::DataNum,Bot>>=bots.iter().map(|b|{
                let rect=aabb_from_point_isize(b.pos,[5,5]);
                BBoxDemo::new(datanum::from_rect(&mut counter,rect),*b)
            }).collect();

            colfind::query_sweep_mut(axgeom::XAXISS,&mut bb,|a,b|{
                a.inner.num-=1;
                b.inner.num-=1;
            });

            //println!("Number of comparisions naive={}",counter.into_inner());   
            for (a,b) in bb.iter().zip(bots.iter_mut()){
                *b=a.inner;
            }
             
            counter.into_inner()
        };

        #[derive(Debug, Serialize)]
        struct Record {
            num_bots: usize,
            num_comparison_alg: usize,
            num_comparison_naive: usize,
            num_comparison_sweep:usize
        }

        self.wtr.serialize(Record{num_bots:self.num_bots,num_comparison_alg:c1,num_comparison_naive:c2,num_comparison_sweep:c3});
        //println!("num_bots={:?} test/naive={:?} ratio:{:.2}",self.num_bots,(c1,c2),c1 as f64/c2 as f64);




        self.num_bots+=1;
        false
     }
}

