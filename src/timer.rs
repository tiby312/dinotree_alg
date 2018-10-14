use super::*;
use std::time::Instant;
use dinotree_inner::Splitter;

fn into_secs(elapsed:std::time::Duration)->f64{
    let sec = (elapsed.as_secs() as f64) + (elapsed.subsec_nanos() as f64 / 1000_000_000.0);
    sec
}
pub struct LevelTimer{
    levels:Vec<f64>,
    time:Option<Instant>,
}

impl LevelTimer{
    pub fn new()->LevelTimer{
        
        LevelTimer{levels:Vec::new(),time:None}
    }
    pub fn into_inner(self)->Vec<f64>{
        self.levels
    }
    fn node_end_common(&mut self){

        let time=self.time.unwrap();

        let elapsed=time.elapsed();
        self.levels.push(into_secs(elapsed));
        self.time=None;
    }
}
impl Splitter for LevelTimer{
    fn div(mut self)->(Self,Self){
        self.node_end_common();

        let length=self.levels.len();

        (self,LevelTimer{levels:std::iter::repeat(0.0).take(length).collect(),time:None})
    }
    fn add(mut self,a:Self)->Self{
        //for (a,b) in self.levels.iter_mut().zip(a.levels.iter()){
        //    *a+=*b;
        //}
        //self

        let (smaller,mut larger)=if self.levels.len()<a.levels.len(){
            (self,a)
        }else{
            (a,self)
        };


        for (a,b) in larger.levels.iter_mut().zip(smaller.levels.iter()){
            *a+=*b;
        }
        larger
    }
    fn node_start(&mut self){
        assert!(self.time.is_none());
        self.time=Some(Instant::now());
    }
    fn node_end(&mut self){
        self.node_end_common();
    } 
}