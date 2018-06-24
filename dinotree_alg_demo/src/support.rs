use axgeom::*;
use rand;
use rand::{SeedableRng, StdRng};
use rand::distributions::{IndependentSample, Range};
use dinotree::*;

use ordered_float::*;
use dinotree::support::*;

pub mod prelude{
    pub use compt::*;
    pub use DemoSys;
    pub(crate) use piston_window;
    pub use ordered_float::NotNaN;
    pub use piston_window::*;
    pub use dinotree_inner::DynTree;
    pub use dinotree_inner::HasAabb;
    pub(crate) use axgeom;
    //pub use dinotree::*;
    pub(crate) use support;
    pub(crate) use support::*;
    
    pub use dinotree::support::*;

}


macro_rules! f64n {
    ( $x:expr  ) => {
        {
            NotNaN::new($x).unwrap()
        }
    };
}

pub type f64N=NotNaN<f64>;

pub struct Conv;
impl Conv{

    pub fn point_to_inner(a:[f64N;2])->[f64;2]{
        //TODO safe to use transmute?
        [a[0].into_inner(),a[1].into_inner()]
    }
    pub fn rect_to_inner(rect:Rect<f64N>)->Rect<f64>{
        let ((a,b),(c,d))=rect.get();
        Rect::new(a.into_inner(),b.into_inner(),c.into_inner(),d.into_inner())   
    }

    pub fn from_rect(rect:Rect<f64>)->Rect<f64N>{
        let ((a,b),(c,d))=rect.get();
        Rect::new(f64n!(a),f64n!(b),f64n!(c),f64n!(d))
    }
    pub fn from_point(a:[f64;2])->[f64N;2]{
        [f64n!(a[0]),f64n!(a[1])]
    }
}

pub fn aabb_from_pointf64(p:[f64;2],r:[f64;2])->Rect<f64>{
    Rect::new(p[0]-r[0],p[0]+r[0],p[1]-r[1],p[1]+r[1])
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
    let mut rng =  SeedableRng::from_seed(arr);


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
