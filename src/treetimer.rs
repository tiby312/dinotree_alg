 use super::*;
//TODO use this
pub trait TreeTimerTrait:Sized+Send{
    type Bag:Send+Sized;
    fn combine(a:Self::Bag,b:Self::Bag)->Self::Bag;
    fn new(height:usize)->Self;
    fn leaf_finish(self)->Self::Bag;
    fn start(&mut self);
    fn next(self)->(Self,Self);
}


pub struct TreeTimerEmpty;
pub struct BagEmpty;
impl TreeTimerTrait for TreeTimerEmpty{
    type Bag=BagEmpty;
    fn combine(mut a:BagEmpty,b:BagEmpty)->BagEmpty{
        BagEmpty
    }

    fn new(height:usize)->TreeTimerEmpty{
        TreeTimerEmpty
    }

    fn leaf_finish(self)->BagEmpty{
        BagEmpty
    }

    fn start(&mut self){

    }
    fn next(self)->(Self,Self){
        (TreeTimerEmpty,TreeTimerEmpty)
    }

}
pub struct Bag{
    a:Vec<f64>
}
impl Bag{
    pub fn into_vec(self)->Vec<f64>{
        self.a
    }
}

pub struct TreeTimer2{
    a:Vec<f64>,
    index:usize,
    timer:Option<tools::Timer2>
}



impl TreeTimerTrait for TreeTimer2{
    type Bag=Bag;
    fn combine(mut a:Bag,b:Bag)->Bag{
        for (i,j) in a.a.iter_mut().zip(b.a.iter()){
            *i+=j;
        }
        a
    }
    fn new(height:usize)->TreeTimer2{
        let v=(0..height).map(|_|0.0).collect();
        
        TreeTimer2{a:v,index:0,timer:None}
    }

    fn leaf_finish(self)->Bag{
       let TreeTimer2{mut a,index,timer}=self;
        a[index]+=timer.unwrap().elapsed();
        Bag{a:a}
    }

    fn start(&mut self){
        self.timer=Some(tools::Timer2::new())
    }

    fn next(self)->(TreeTimer2,TreeTimer2){
        let TreeTimer2{mut a,index,timer}=self;
        a[index]+=timer.unwrap().elapsed();

        let b=(0..a.len()).map(|_|0.0).collect();
        (
            TreeTimer2{a:a,index:index+1,timer:None},
            TreeTimer2{a:b,index:index+1,timer:
                None}
        )
    }

  
}