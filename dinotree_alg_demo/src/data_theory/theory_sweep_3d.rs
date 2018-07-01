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

pub struct ClosenessCounter{
    radius:f64
}

impl Iterator for ClosenessCounter{
    type Item=SpiralGenerator;
    fn next(&mut self)->Option<SpiralGenerator>
    {
        if self.radius<=0.0{
            return None;
        }
        //SpiralGenerator::new([400.0,400.0],12.0,2.0)
        let k=SpiralGenerator::new([400.0,400.0],self.radius,2.0);
        self.radius-=0.1;
        Some(k)
    }
}


pub fn test(s:SpiralGenerator,num_bots:usize)->usize{

    let mut bots:Vec<BBox<isize,Bot>>=s.take(num_bots).map(|pos|{
            let pos=[pos[0] as isize,pos[1] as isize];
            BBox::new(aabb_from_point_isize(pos,[5,5]),Bot{num:0})
        }
    ).collect();

    for bot in bots.iter(){
        //draw_rect_isize([0.0,0.0,0.0,0.3],bot.get(),c,g);
    }  

    let c1={
        let mut counter=datanum::Counter::new();

        
        let mut bb:Vec<BBox<datanum::DataNum,Bot>>=bots.drain(..).map(|b|{     
            BBox::new(datanum::from_rect(&mut counter,*b.get()),b.inner)
        }).collect();
        


        colfind::query_sweep_mut(axgeom::XAXISS,&mut bb,|a, b| {
            a.inner.num+=2;
            b.inner.num+=2;
            let a=datanum::into_rect(*a.get());
            let b=datanum::into_rect(*b.get());
            //draw_rect_isize([1.0,0.0,0.0,0.2],&a,c,g);
            //draw_rect_isize([1.0,0.0,0.0,0.2],&b,c,g);
    
        });
        

        //println!("Number of comparisons tree={}",counter.into_inner());

        for b in bb.into_iter(){
            let b=BBox::new(datanum::into_rect(*b.get()),b.inner);    
            bots.push(b);
        }
        /*
        for b in tree.into_iter_orig_order(){
            let b=BBox::new(datanum::into_rect(*b.get()),b.inner);
            bots.push(b);
        } 
        */
        counter.into_inner()
    };
    c1
}

pub struct DataColFind3d{
    num_bots:usize,
    wtr:csv::Writer<std::io::Stdout>
}


impl DataColFind3d{
    pub fn new(dim:[f64;2])->DataColFind3d{    
        let mut wtr = csv::Writer::from_writer(std::io::stdout());
        DataColFind3d{num_bots:0,wtr}
    }
}


impl DemoSys for DataColFind3d{
    fn step(&mut self,cursor:[f64;2],c:&piston_window::Context,g:&mut piston_window::G2d){

        let mut cc=ClosenessCounter{radius:12.0};
        for s in cc{
            let circular_grow=s.get_circular_grow();
                
            for num_bots in (0..100usize).step_by(5){
                let s2=s.clone();
                let z=test(s2,num_bots);

                #[derive(Debug, Serialize)]
                struct Record {
                    num_bots: usize,
                    circular_grow: f64,
                    z: usize
                }
                let num_bots=num_bots;
                self.wtr.serialize(Record{num_bots,circular_grow,z});
            }
        }
        panic!("Finish");
    }
}

