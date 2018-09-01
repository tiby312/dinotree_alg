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
    num:usize,
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
        self.radius-=0.5;
        Some(k)
    }
}



fn test1(bots:&mut [Bot])->usize{
    let mut counter=datanum::Counter::new();

    let mut tree=DynTree::new_seq(axgeom::XAXISS,(),bots,|b|{
        datanum::from_rect(&mut counter,aabb_from_point_isize(b.pos,[5,5]))  
    });

    colfind::query_seq_mut(&mut tree,|a, b| {
        a.inner.num+=2;
        b.inner.num+=2;
    });

    tree.apply_orig_order(bots,|a,b|{
        b.num=a.inner.num;
    });

    counter.into_inner()
}
fn test2(bots:&mut [Bot])->usize{
    let mut counter=datanum::Counter::new();

    let mut bb:Vec<BBoxDemo<datanum::DataNum,Bot>>=bots.iter().map(|b|{   
        let rect=datanum::from_rect(&mut counter,aabb_from_point_isize(b.pos,[5,5]));  
        BBoxDemo::new(rect,*b)
    }).collect();
    


    colfind::query_sweep_mut(axgeom::XAXISS,&mut bb,|a, b| {
        a.inner.num+=2;
        b.inner.num+=2;
    });
    

    //println!("Number of comparisons tree={}",counter.into_inner());

    for (i,j) in bb.into_iter().zip(bots.iter_mut()){
        //let b=BBoxDemo::new(datanum::into_rect(*i.get()),i.inner);    
        //bots.push(b);
        *j=i.inner;
    }
    counter.into_inner()
}

fn test3(bots:&mut [Bot])->usize{
    let mut counter=datanum::Counter::new();

    let mut bb:Vec<BBoxDemo<datanum::DataNum,Bot>>=bots.iter().map(|b|{   
        let rect=datanum::from_rect(&mut counter,aabb_from_point_isize(b.pos,[5,5]));  
        BBoxDemo::new(rect,*b)
    }).collect();
    


    colfind::query_naive_mut(&mut bb,|a, b| {
        a.inner.num+=2;
        b.inner.num+=2;
    });
    

    //println!("Number of comparisons tree={}",counter.into_inner());

    for (i,j) in bb.into_iter().zip(bots.iter_mut()){
        //let b=BBoxDemo::new(datanum::into_rect(*i.get()),i.inner);    
        //bots.push(b);
        *j=i.inner;
    }
    counter.into_inner()
}



pub struct DataColFind3d{
    num_bots:usize,
    //wtr:csv::Writer<std::io::Stdout>
}


impl DataColFind3d{
    pub fn new(_dim:[f64;2])->DataColFind3d{    
        //let wtr = csv::Writer::from_writer(std::io::stdout());
        DataColFind3d{num_bots:0}
    }
}



impl DemoSys for DataColFind3d{
    fn step(&mut self,_cursor:[f64;2],_c:&piston_window::Context,_g:&mut piston_window::G2d)->bool{

        let cc=ClosenessCounter{radius:12.0};


        let mut rects=Vec::new();


        for s in cc{
            let circular_grow=s.get_circular_grow();
                
            for num_bots in (0..8000usize).step_by(200){
                let s2=s.clone();

                //let mut bots:Vec<Bot>=(0..num_bots).map(|a|Bot{num:0}).collect();
                let mut bots:Vec<Bot>=s2.take(num_bots).map(|pos|{
                    let pos=[pos[0] as isize,pos[1] as isize];
                    Bot{num:0,pos}
                }).collect();

                let z1=test1(&mut bots);
                let z2=test2(&mut bots);
                let z3=if num_bots<3000{
                    Some(test3(&mut bots))
                }else{
                    None
                };
                #[derive(Debug, Serialize)]
                struct Record {
                    num_bots: usize,
                    circular_grow: f64,
                    z1: usize,
                    z2: usize,
                    z3: Option<usize>
                }
                let r=Record{num_bots,circular_grow,z1,z2,z3};
                rects.push(r);   
            }
        }

        {
            use gnuplot::*;
            let x=rects.iter().map(|a|a.num_bots as f64);
            let y=rects.iter().map(|a|a.circular_grow as f64);
            let z1=rects.iter().map(|a|a.z1 as f64);
            let z2=rects.iter().map(|a|a.z2 as f64);

            
            let (x2,y2,z3)={

                let ii=rects.iter().filter(|a|a.z3.is_some());
                let x=ii.clone().map(|a|a.num_bots as f64);
                let y=ii.clone().map(|a|a.circular_grow as f64);
                let z3=ii.clone().map(|a|a.z3.unwrap());

                (x,y,z3)
            };
            

            let mut fg = Figure::new();

            fg.axes3d().set_view(110.0,30.0)
                .set_title("Comparison of Sweep and Prune versus Dinotree", &[])
                .set_x_label("Number of Objects", &[])
                .set_y_label("Spareness of Objects", &[])
                .set_z_label("Number of Comparisons", &[Rotate(90.0),TextOffset(-3.0,0.0)])
                .points(x.clone(), y.clone(), z1.clone(), &[Caption("Dinotree"),PointSymbol('O'), Color("violet"), PointSize(1.0)])
                .points(x.clone(), y.clone(), z2.clone(), &[Caption("Sweep and Prune"),PointSymbol('o'), Color("red"), PointSize(1.0)])
                .points(x2.clone(), y2.clone(), z3.clone(), &[Caption("Naive"),PointSymbol('o'), Color("green"), PointSize(0.3)]);


            fg.show();

            return true;
        }

        return true;
    }
}

