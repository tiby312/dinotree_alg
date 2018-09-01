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
    records:Vec<Record>
    //wtr:csv::Writer<std::io::Stdout>
}


impl DataColFind{
    pub fn new(_dim:[f64;2])->DataColFind{    
        //let wtr = csv::Writer::from_writer(std::io::stdout());
        DataColFind{num_bots:0,records:Vec::new()}
    }
}


#[derive(Debug, Serialize)]
struct Record {
    num_bots: usize,
    num_comparison_alg: usize,
    num_comparison_naive: Option<usize>,
    num_comparison_sweep:Option<usize>
}



impl DemoSys for DataColFind{
    fn step(&mut self,_cursor:[f64;2],c:&piston_window::Context,g:&mut piston_window::G2d)->bool{

        let s=SpiralGenerator::new([400.0,400.0],12.0,2.0);

        if self.num_bots>7000{
            {
                let rects=&mut self.records;
                use gnuplot::*;
                let x=rects.iter().map(|a|a.num_bots);
                let y1=rects.iter().map(|a|a.num_comparison_alg);
                let y2=rects.iter().take_while(|a|a.num_comparison_naive.is_some()).map(|a|a.num_comparison_naive.unwrap());
                let y3=rects.iter().take_while(|a|a.num_comparison_sweep.is_some()).map(|a|a.num_comparison_sweep.unwrap());

                let mut fg = Figure::new();

                fg.axes2d()
                    .set_title("Comparison of AABB Collision Detection Algorithms", &[])
                    .lines(x.clone(), y2,  &[Caption("Naive"), Color("blue"), LineWidth(2.0)])
                    .lines(x.clone(), y3,  &[Caption("Sweep and Prune"), Color("green"), LineWidth(2.0)])
                    .lines(x.clone(), y1,  &[Caption("Dinotree"), Color("red"), LineWidth(2.0)])
                    .set_x_label("Number of Objects", &[])
                    .set_y_label("Number of Comparisons", &[]);
        
                fg.show();

                return true;
            }
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
            
            
            if self.num_bots<600{
                let mut counter=datanum::Counter::new();
            
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
                Some(counter.into_inner())
            }else{
                None
            }
        };
        let c3={
            if self.num_bots<4000{
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
                 
                Some(counter.into_inner())
            }else{
                None
            }

        };

        self.records.push(Record{num_bots:self.num_bots,num_comparison_alg:c1,num_comparison_naive:c2,num_comparison_sweep:c3});
        //println!("num_bots={:?} test/naive={:?} ratio:{:.2}",self.num_bots,(c1,c2),c1 as f64/c2 as f64);




        self.num_bots+=1;
        false
     }
}

