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


pub fn test(mut s:SpiralGenerator,num_bots:usize)->usize{

    
    let mut bots:Vec<Bot>=(0..num_bots).map(|a|Bot{num:0}).collect();

    let c1={
        let mut counter=datanum::Counter::new();

        let mut tree=DynTree::new_seq(axgeom::XAXISS,(),&bots,|b|{
            let pos=s.next().unwrap();  
            let pos=[pos[0] as isize,pos[1] as isize];
            datanum::from_rect(&mut counter,aabb_from_point_isize(pos,[5,5]))  
        });

        colfind::query_seq_mut(&mut tree,|a, b| {
            a.inner.num+=2;
            b.inner.num+=2;
        });

        tree.apply_orig_order(&mut bots,|a,b|{
            b.num=a.inner.num;
        });

        counter.into_inner()
    };

    //assert!(c1>0);
    
    c1
}

pub struct DataColFind3d{
    num_bots:usize,
    wtr:csv::Writer<std::io::Stdout>
}


impl DataColFind3d{
    pub fn new(_dim:[f64;2])->DataColFind3d{    
        let wtr = csv::Writer::from_writer(std::io::stdout());
        DataColFind3d{num_bots:0,wtr}
    }
}


impl DemoSys for DataColFind3d{
    fn step(&mut self,_cursor:[f64;2],_c:&piston_window::Context,_g:&mut piston_window::G2d)->bool{

        let cc=ClosenessCounter{radius:12.0};


        let mut rects=Vec::new();

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
                let r=Record{num_bots,circular_grow,z};
                rects.push(r);
                //self.wtr.serialize();
                
            }
        }

        {
            use gnuplot::*;
            let x=rects.iter().map(|a|a.num_bots as f64);
            let y=rects.iter().map(|a|a.circular_grow as f64);
            let z=rects.iter().map(|a|a.z as f64);
            //let z = (0..100).map(|z| z as f32 / 10.0);
            //let x = z.clone().map(|z| z.cos());
            //let y = z.clone().map(|z| z.sin());

            let mut fg = Figure::new();

            fg.axes3d().set_view(120.0,30.0)
                .set_title("3D lines", &[])
                .set_x_label("Number of Objects", &[])
                .set_y_label("Number of Intersecting Objects", &[])
                .set_z_label("Number of Comparisons", &[Rotate(90.0),TextOffset(-2.0,0.0)])
                .points(x, y, z, &[PointSymbol('O'), Color("violet"), PointSize(1.0)]);

            fg.show();

            return true;
        }

        return true;
    }
}

