
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
