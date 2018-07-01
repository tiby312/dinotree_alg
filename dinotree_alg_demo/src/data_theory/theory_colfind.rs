use support::prelude::*;
use dinotree::colfind;
use dinotree::rect;
use dinotree_geom;
use csv;
use std;
pub struct Bot{
    num:usize
}
pub struct DataColFind{
    num_bots:usize,
    wtr:csv::Writer<std::io::Stdout>
}


impl DataColFind{
    pub fn new(dim:[f64;2])->DataColFind{    
        let mut wtr = csv::Writer::from_writer(std::io::stdout());
        DataColFind{num_bots:0,wtr}
    }
}




impl DemoSys for DataColFind{
    fn step(&mut self,cursor:[f64;2],c:&piston_window::Context,g:&mut piston_window::G2d){

        let s=SpiralGenerator::new([400.0,400.0],12.0,2.0);

        if self.num_bots>300{
            panic!();
        }

        let mut bots:Vec<BBox<isize,Bot>>=s.take(self.num_bots).map(|pos|{
                let pos=[pos[0] as isize,pos[1] as isize];
                BBox::new(aabb_from_point_isize(pos,[5,5]),Bot{num:0})
            }
        ).collect();

        for bot in bots.iter(){
            draw_rect_isize([0.0,0.0,0.0,0.3],bot.get(),c,g);
        }  

        let c1={
            let mut counter=datanum::Counter::new();


            let mut tree=DynTree::new_seq(axgeom::XAXISS,(),bots.drain(..).map(|b|{     
                BBox::new(datanum::from_rect(&mut counter,*b.get()),b.inner)
            }));


            colfind::query_mut(&mut tree,|a, b| {
                a.inner.num+=2;
                b.inner.num+=2;
                let a=datanum::into_rect(*a.get());
                let b=datanum::into_rect(*b.get());
                draw_rect_isize([1.0,0.0,0.0,0.2],&a,c,g);
                draw_rect_isize([1.0,0.0,0.0,0.2],&b,c,g);
        
            });
            

            //println!("Number of comparisons tree={}",counter.into_inner());


            for b in tree.into_iter_orig_order(){
                let b=BBox::new(datanum::into_rect(*b.get()),b.inner);
                bots.push(b);
            } 
            counter.into_inner()
        };
       
        let c2={
            let mut counter=datanum::Counter::new();
            let mut bb:Vec<BBox<datanum::DataNum,Bot>>=bots.drain(..).map(|b|{
                BBox::new(datanum::from_rect(&mut counter,*b.get()),b.inner)
            }).collect();

            colfind::query_naive_mut(&mut bb,|a,b|{
                a.inner.num-=1;
                b.inner.num-=1;
            });

            //println!("Number of comparisions naive={}",counter.into_inner());   

            for b in bb.drain(..){
                let b=BBox::new(datanum::into_rect(*b.get()),b.inner);
                bots.push(b);
            } 
            counter.into_inner()
        };
        let c3={
            let mut counter=datanum::Counter::new();
            let mut bb:Vec<BBox<datanum::DataNum,Bot>>=bots.drain(..).map(|b|{
                BBox::new(datanum::from_rect(&mut counter,*b.get()),b.inner)
            }).collect();

            colfind::query_sweep_mut(axgeom::XAXISS,&mut bb,|a,b|{
                a.inner.num-=1;
                b.inner.num-=1;
            });

            //println!("Number of comparisions naive={}",counter.into_inner());   

            for b in bb.drain(..){
                assert_eq!(b.inner.num,0);
                let b=BBox::new(datanum::into_rect(*b.get()),b.inner);
                bots.push(b);
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
     }
}

