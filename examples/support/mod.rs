use axgeom::*;
use rand;
use rand::{SeedableRng, StdRng};
use rand::distributions::{IndependentSample, Range};
use dinotree::*;

use ordered_float::*;
use dinotree::support::*;

#[allow(dead_code)]
pub fn create_bots_f64<X:Send+Sync,F:FnMut(usize,[f64;2])->X>(mut func:F,area:&[isize;4],num_bots:usize,radius:[isize;2])->Vec<BBox<NotNaN<f64>,X>>{
    
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

        bots.push(BBox{
            inner:func(id,[px.into_inner(),py.into_inner()]),
            rect:Rect::new(px-rx,px+rx,py-ry,py+ry)
        });
    }
    bots

}


#[allow(dead_code)]
pub fn create_bots_isize_seed<X:Send+Sync,F:FnMut(usize)->X>(seed:&[usize],mut func:F,area:&[isize;4],num_bots:usize,radius:[isize;2])->Vec<BBox<isize,X>>{
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

        bots.push(BBox{
            inner:func(id),
            rect:Rect::new(px-rx,px+rx,py-ry,py+ry)
        });
    }
    bots
}
#[allow(dead_code)]
pub fn create_bots_isize<X:Send+Sync,F:FnMut(usize)->X>(func:F,area:&[isize;4],num_bots:usize,radius:[isize;2])->Vec<BBox<isize,X>>{
    
    let arr:&[usize]=&[100,42,6];
    create_bots_isize_seed(arr,func,area,num_bots,radius)

}


#[allow(dead_code)]
pub fn rectf64_to_notnan(rect:Rect<f64>)->Rect<NotNaN<f64>>{
    let ((a,b),(c,d))=rect.get();

    Rect::new(NotNaN::new(a).unwrap(),NotNaN::new(b).unwrap(),NotNaN::new(c).unwrap(),NotNaN::new(d).unwrap())
}


#[derive(Clone, Debug)]
pub struct Bot {
    pub id: usize,
    pub col: Vec<usize>,
}

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


pub struct UniformRangeGenerator{
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
