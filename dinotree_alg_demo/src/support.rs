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
    pub use support::Bot;
    pub use dinotree_inner::DynTree;
    pub use dinotree_inner::HasAabb;
    pub(crate) use axgeom;
    //pub use dinotree::*;
    pub(crate) use support;
    pub(crate) use support::*;
    
    pub(crate) use num;
    pub use dinotree::support::*;

}

pub type f64N=NotNaN<f64>;

macro_rules! f64n {
    ( $x:expr  ) => {
        {
            NotNaN::new($x).unwrap()
        }
    };
}





pub struct Bot{
    pub id:usize,
    pub pos:[f64N;2],
    pub vel:[f64N;2],
    pub acc:[f64N;2],
    pub radius:[f64N;2],
}

impl Bot{

    pub fn wrap_position(&mut self,dim:[f64N;2]){
        let mut a=[self.pos[0],self.pos[1]];
        
        let start=[f64n!(0.0);2];

        if a[0]>dim[0]{
            a[0]=start[0]
        }
        if a[0]<start[0]{
            a[0]=dim[0];
        }
        if a[1]>dim[1]{
            a[1]=start[1];
        }
        if a[1]<start[1]{
            a[1]=dim[1];
        }
        self.pos=[a[0],a[1]]
    }

    pub fn update(&mut self){
        self.vel[0]+=self.acc[0];
        self.vel[1]+=self.acc[1];
        self.pos[0]+=self.vel[0];
        self.pos[1]+=self.vel[1];
        self.acc[0]=f64n!(0.0);
        self.acc[1]=f64n!(0.0);
    }
}


/*
///A generic container that implements the kdtree trait.
#[derive(Debug,Clone,Copy)]
pub struct BBox<Nu:NumTrait,T>{
    pub rect:Rect<Nu>,
    pub inner:T
}
/*
impl<Nu:NumTrait,T> BBox<Nu,T>{
    fn update<A:AxisTrait,N>(_tree:mut DynTree<A,N,BBox<Nu,T>>,func:Fn(&BBox<Nu,T>)->)
}
*/

impl<Nu:NumTrait,T> HasAabb for BBox<Nu,T>{
    type Num=Nu;
    
    ///Destructue into the bounding box and inner part.
    fn get<'a>(&'a self)->&Rect<Nu>{
        &self.rect
    }
}
*/


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
    pub pos:[f64N;2],
    pub vel:[f64N;2],
    pub radius:[f64N;2],
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
        let pos=[self.pos[0].into_inner() as isize,self.pos[1].into_inner() as isize];
        let vel=[self.vel[0].into_inner() as isize,self.vel[1].into_inner() as isize];
        let radius=[self.radius[0].into_inner() as isize,self.radius[1].into_inner() as isize];
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
        let px=f64n!(self.xvaluegen.get(rng) as f64);
        let py=f64n!(self.yvaluegen.get(rng) as f64);
        let rx=f64n!(self.radiusgen.get(rng) as f64);
        let ry=f64n!(self.radiusgen.get(rng) as f64);

        let (velx,vely)={
            let vel_dir=self.velocity_dir.get(rng) as f64;
            let vel_dir=vel_dir.to_radians();
            let (mut xval,mut yval)=(vel_dir.cos(),vel_dir.sin());
            let vel_mag=self.velocity_mag.get(rng) as f64;
            xval*=vel_mag;
            yval*=vel_mag;
            (f64n!(xval),f64n!(yval))
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
/*
pub fn create_bots_f64<X:Send+Sync,F:FnMut(usize,[f64;2])->X>(mut func:F,area:&[isize;4],num_bots:usize,radius:[isize;2])->Vec<BBoxVisible<NotNaN<f64>,X>>{
    
    let arr:&[usize]=&[100,42,6];
    let mut rng =  SeedableRng::from_seed(arr);
    let rng=&mut rng;

    let xvaluegen=UniformRangeGenerator::new(area[0],area[1]);
    let yvaluegen=UniformRangeGenerator::new(area[2],area[3]);
    let radiusgen= UniformRangeGenerator::new(radius[0],radius[1]);


    let mut bots = Vec::with_capacity(num_bots);
    for id in 0..num_bots {;

        let px=NotNaN::new(xvaluegen.get(rng) as f64).unwrap();
        let py=NotNaN::new(yvaluegen.get(rng) as f64).unwrap();
        let rx=NotNaN::new(radiusgen.get(rng) as f64).unwrap();
        let ry=NotNaN::new(radiusgen.get(rng) as f64).unwrap();

        bots.push(BBoxVisible{
            inner:func(id,[px.into_inner(),py.into_inner()]),
            rect:Rect::new(px-rx,px+rx,py-ry,py+ry)
        });
        
            
    }
    bots

}


#[allow(dead_code)]
pub fn create_bots_isize_seed<X:Send+Sync,F:FnMut(usize)->X>(seed:&[usize],mut func:F,area:&[isize;4],num_bots:usize,radius:[isize;2])->Vec<BBoxVisible<isize,X>>{
    let mut rng =  SeedableRng::from_seed(seed);
    let rng=&mut rng;

    let xvaluegen=UniformRangeGenerator::new(area[0],area[1]);
    let yvaluegen=UniformRangeGenerator::new(area[2],area[3]);
    let radiusgen= UniformRangeGenerator::new(radius[0],radius[1]);


    let mut bots = Vec::with_capacity(num_bots);
    for id in 0..num_bots {;

        let px=xvaluegen.get(rng);
        let py=yvaluegen.get(rng);
        let rx=radiusgen.get(rng);
        let ry=radiusgen.get(rng);

        bots.push(BBoxVisible{
            inner:func(id),
            rect:Rect::new(px-rx,px+rx,py-ry,py+ry)
        });
    }
    bots
}
#[allow(dead_code)]
pub fn create_bots_isize<X:Send+Sync,F:FnMut(usize)->X>(func:F,area:&[isize;4],num_bots:usize,radius:[isize;2])->Vec<BBoxVisible<isize,X>>{
    
    let arr:&[usize]=&[100,42,6];
    create_bots_isize_seed(arr,func,area,num_bots,radius)

}*/


pub fn create_aabb_f64(center:[f64N;2],radius:[f64N;2])->Rect<f64N>{
    Rect::new(center[0]-radius[0],center[0]+radius[1],center[1]-radius[1],center[1]+radius[1])    
}
#[allow(dead_code)]
pub fn rectf64_to_notnan(rect:Rect<f64>)->Rect<NotNaN<f64>>{
    let ((a,b),(c,d))=rect.get();

    Rect::new(NotNaN::new(a).unwrap(),NotNaN::new(b).unwrap(),NotNaN::new(c).unwrap(),NotNaN::new(d).unwrap())
}

/*
#[derive(Clone, Debug)]
pub struct Bot {
    pub id: usize,
    pub col: Vec<usize>,
}
*/

/*
pub fn make_rect(a: (isize, isize), b: (isize, isize)) -> axgeom::Rect<isize> {
    axgeom::Rect::new(a.0, a.1, b.0, b.1)
}
*/

/*
pub fn create_rect_from_point_f64(a: (f64, f64)) -> AABBox<NotNaN<f64>> {
    let r = 8.0;
    let x = a.0;
    let y = a.1;

    let x1=NotNaN::new(x).unwrap();
    let x2=NotNaN::new(x+r).unwrap();
    let y1=NotNaN::new(y).unwrap();
    let y2=NotNaN::new(y+r).unwrap();
    AABBox(axgeom::Rect::new(x1,x2,y1,y2))
    //AABBox(make_rect((x , x + r), (y , y + r)))
}
*/
/*
pub fn create_unordered(a: &Bot, b: &Bot) -> (usize, usize) {
    if a.id < b.id {
        (a.id, b.id)
    } else {
        (b.id, a.id)
    }
}
pub fn compair_bot_pair(a: &(usize, usize), b: &(usize, usize)) -> std::cmp::Ordering {
    if a.0 < b.0 {
        std::cmp::Ordering::Less
    } else if a.0 > b.0 {
        std::cmp::Ordering::Greater
    } else {
        if a.1 < b.1 {
            std::cmp::Ordering::Less
        } else if a.1 > b.1 {
            std::cmp::Ordering::Greater
        } else {
            std::cmp::Ordering::Equal
        }
    }
}
*/


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
