use axgeom::*;
use rand;
use rand::{SeedableRng, StdRng};
use rand::distributions::{IndependentSample, Range};


pub mod prelude{
    pub use compt::*;
    pub use DemoSys;
    pub(crate) use piston_window;
    pub use ordered_float::NotNan;
    pub use piston_window::*;
    pub use dinotree_inner::DynTree;
    pub use dinotree_inner::HasAabb;
    pub(crate) use axgeom;
    pub(crate) use support::*;
    
    pub use dinotree::support::*;

}
use ordered_float::NotNan;


pub mod datanum{
    use std;
    use std::cmp::Ordering;
    use std::cell::UnsafeCell;
    use std::marker::PhantomData;
    use axgeom::Rect;

    pub struct Counter(usize);

    pub fn from_rect(counter:&mut Counter,rect:Rect<isize>)->Rect<DataNum>{
        let ((a,b),(c,d))=rect.get();
        Rect::new(counter.new_num(a),counter.new_num(b),counter.new_num(c),counter.new_num(d))
    }

    pub fn into_rect(rect:Rect<DataNum>)->Rect<isize>{
        let ((a,b),(c,d))=rect.get();
        Rect::new(a.into_inner(),b.into_inner(),c.into_inner(),d.into_inner())
    }

    impl Counter{
        pub fn new()->Counter{
            Counter(0)
        }
        pub fn into_inner(self)->usize{
            self.0
        }
        pub fn new_num(&mut self,a:isize)->DataNum{
            DataNum(a,self as *mut Counter)
        }
    }

    #[derive(Copy,Clone)]
    pub struct DataNum(pub isize,*mut Counter);
    
    //unsafe implement send and sync.
    //we will be cause to only use sequential version of the tree algorithms
    unsafe impl Send for DataNum{}
    unsafe impl Sync for DataNum{}

    impl DataNum{
        pub fn new(&self,a:isize)->DataNum{
            DataNum(a,self.1)
        }
        pub fn into_inner(&self)->isize{
            self.0
        }
    }

    impl PartialOrd for DataNum {
        fn partial_cmp(&self, other: &DataNum) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }

    impl PartialEq for DataNum {
        fn eq(&self, other: &DataNum) -> bool {
            self.0.cmp(&other.0)==Ordering::Equal
        }
    }

    impl Eq for DataNum {}
    impl Ord for DataNum{
        fn cmp(&self, other: &DataNum) -> Ordering {

            unsafe{
                let mut p=self.1;
                (*p).0+=1;
            }
            self.0.cmp(&other.0)
        }
    }
}



macro_rules! f64n {
    ( $x:expr  ) => {
        {
            NotNan::new($x).unwrap()
        }
    };
}

pub type F64n=NotNan<f64>;

pub struct Conv;
impl Conv{

    pub fn point_to_inner(a:[F64n;2])->[f64;2]{
        //TODO safe to use transmute?
        [a[0].into_inner(),a[1].into_inner()]
    }
    pub fn rect_to_inner(rect:Rect<F64n>)->Rect<f64>{
        let ((a,b),(c,d))=rect.get();
        Rect::new(a.into_inner(),b.into_inner(),c.into_inner(),d.into_inner())   
    }

    pub fn from_rect(rect:Rect<f64>)->Rect<F64n>{
        let ((a,b),(c,d))=rect.get();
        Rect::new(f64n!(a),f64n!(b),f64n!(c),f64n!(d))
    }
}

pub fn aabb_from_pointf64(p:[f64;2],r:[f64;2])->Rect<f64>{
    Rect::new(p[0]-r[0],p[0]+r[0],p[1]-r[1],p[1]+r[1])
}

pub fn aabb_from_point_isize(p:[isize;2],r:[isize;2])->Rect<isize>{
    Rect::new(p[0]-r[0],p[0]+r[0],p[1]-r[1],p[1]+r[1])
}

use piston_window::*;
pub fn draw_rect_f64n(col:[f32;4],r1:&Rect<F64n>,c:&Context,g:&mut G2d){
    let ((x1,x2),(y1,y2))=r1.get();        
    {
        //let ((x1,x2),(y1,y2))=((x1 as f64,x2 as f64),(y1 as f64,y2 as f64));
        let ((x1,x2),(y1,y2))=((x1.into_inner(),x2.into_inner()),(y1.into_inner(),y2.into_inner()));
           
        let square = [x1,y1,x2-x1,y2-y1];
        rectangle(col, square, c.transform, g);
    }
}
pub fn draw_rect_isize(col:[f32;4],r1:&Rect<isize>,c:&Context,g:&mut G2d){
    let ((x1,x2),(y1,y2))=r1.get();        
    {
        let ((x1,x2),(y1,y2))=((x1 as f64,x2 as f64),(y1 as f64,y2 as f64));
           
        let square = [x1,y1,x2-x1,y2-y1];
        rectangle(col, square, c.transform, g);
    }
}

#[derive(Clone)]
pub struct SpiralGenerator{
    point:[f64;2],
    rad:f64,
    start:f64,
    rate:f64,
    width:f64
}

impl SpiralGenerator{
    pub fn new(point:[f64;2],circular_grow:f64,outward_grow:f64)->SpiralGenerator{
        SpiralGenerator{point,rad:0.0,start:1.0,rate:outward_grow,width:circular_grow}
    }
    pub fn get_circular_grow(&self)->f64{
        self.width
    }
}
use std;
impl std::iter::FusedIterator for SpiralGenerator{}

impl Iterator for SpiralGenerator{
    type Item=[f64;2];
    fn next(&mut self)->Option<[f64;2]>{
        
        let length=self.start+self.rate*self.rad;

        let x=self.point[0]+self.rad.cos()*length;
        let y=self.point[1]+self.rad.sin()*length;

        self.rad+=self.width/length;

        Some([x,y])

    }
}


pub struct RangeGenIterf64{
    max:usize,
    counter:usize,
    rng:rand::StdRng,
    xvaluegen:UniformRangeGenerator,
    yvaluegen:UniformRangeGenerator,
    radiusgen:UniformRangeGenerator,
    velocity_dir:UniformRangeGenerator,
    velocity_mag:UniformRangeGenerator
}

pub struct Retf64{
    pub id:usize,
    pub pos:[f64;2],
    pub vel:[f64;2],
    pub radius:[f64;2],
}

pub struct RetInteger{
    pub id:usize,
    pub pos:[isize;2],
    pub vel:[isize;2],
    pub radius:[isize;2],
}
impl Retf64{
    pub fn into_isize(self)->RetInteger{
        let id=self.id;
        let pos=[self.pos[0] as isize,self.pos[1] as isize];
        let vel=[self.vel[0] as isize,self.vel[1] as isize];
        let radius=[self.radius[0] as isize,self.radius[1] as isize];
        RetInteger{id,pos,vel,radius}
    }
}
impl ExactSizeIterator for RangeGenIterf64{}
impl Iterator for RangeGenIterf64{
    type Item=Retf64;
    fn size_hint(&self)->(usize,Option<usize>){
        (self.max,Some(self.max))
    }
    fn next(&mut self)->Option<Self::Item>{  

        if self.counter==self.max{
            return None
        }

        let rng=&mut self.rng;  
        let px=self.xvaluegen.get(rng) as f64;
        let py=self.yvaluegen.get(rng) as f64;
        let rx=self.radiusgen.get(rng) as f64;
        let ry=self.radiusgen.get(rng) as f64;

        let (velx,vely)={
            let vel_dir=self.velocity_dir.get(rng) as f64;
            let vel_dir=vel_dir.to_radians();
            let (mut xval,mut yval)=(vel_dir.cos(),vel_dir.sin());
            let vel_mag=self.velocity_mag.get(rng) as f64;
            xval*=vel_mag;
            yval*=vel_mag;
            (xval,yval)
        };

        let curr=self.counter;
        self.counter+=1;
        let r=Retf64{id:curr,pos:[px,py],vel:[velx,vely],radius:[rx,ry]};
        Some(r)
    }
}
pub fn create_world_generator(num:usize,area:&[isize;4],radius:[isize;2],velocity:[isize;2])->RangeGenIterf64{
    let arr:&[usize]=&[100,42,6];
    let rng =  SeedableRng::from_seed(arr);


    let xvaluegen=UniformRangeGenerator::new(area[0],area[1]);
    let yvaluegen=UniformRangeGenerator::new(area[2],area[3]);
    let radiusgen= UniformRangeGenerator::new(radius[0],radius[1]);


    let velocity_dir=UniformRangeGenerator::new(0,360);
    let velocity_mag= UniformRangeGenerator::new(velocity[0],velocity[1]);

    RangeGenIterf64{max:num,counter:0,rng,xvaluegen,yvaluegen,radiusgen,velocity_dir,velocity_mag}
}



struct UniformRangeGenerator{
    range:Range<isize>
}

impl UniformRangeGenerator{
    pub fn new(a:isize,b:isize)->Self{
        //let rr = a.get_range2::<axgeom::XAXISS>();
        let xdist = rand::distributions::Range::new(a,b);
        UniformRangeGenerator{range:xdist}
    }
    pub fn get(&self,rng:&mut StdRng)->isize{
        self.range.ind_sample(rng)
    }
}
