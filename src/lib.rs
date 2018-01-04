#![feature(offset_to)]

extern crate axgeom;
extern crate compt;
extern crate rayon;
extern crate pdqselect;
extern crate ordered_float;

mod base_kdtree;
mod tree_alloc;
mod colfind;
pub mod dyntree;
pub mod graphics;
pub mod median;
pub mod oned;
pub mod tools;
pub mod support;
pub mod multirect;

//The TreeCache object is updated during the base kd tree construction.
//So TreeCache and KdTree are tied together.
//On the otherhand, we dont expose KdTree since it is only used
//intermediately in order to construct a DynTree.
pub use base_kdtree::TreeCache;

use compt::LevelDesc;
use axgeom::Rect;

pub trait Bleek{
    type T:SweepTrait;
    fn collide(&mut self,cc:ColPair<Self::T>);
}

pub trait BleekSync:Sync+Copy+Clone{
    type T:SweepTrait+Send;
    fn collide(&self,cc:ColPair<Self::T>);
}




///Returns the level at which a parallel divide and conqur algorithm will switch to sequential
pub trait DepthLevel{
    ///Switch to sequential at this height.
    ///This is highly system dependant of what a "good" level it would be to switch over.
    fn switch_to_sequential(a:LevelDesc)->bool;
}


///A default depth level from which to switch to sequential.
pub struct DefaultDepthLevel;

impl DepthLevel for DefaultDepthLevel{
    fn switch_to_sequential(a:LevelDesc)->bool{
        a.get_depth()>4
    }
}


//The underlying numbers used for the bounding boxes,
//and for the dividers. 
pub trait NumTrait:Ord+Copy+Send+Sync+std::fmt::Debug+Default{}

///Provides a way to destructure an object into a
///reference to a read only bounding box, and a mutable inner struct.
pub trait SweepTrait:Send{
    ///The part of the struct that is allowed to be mutated
    ///during the querying of the tree. It is important that
    ///the bounding boxes not be mutated during querying of the tree
    ///as that would break the invariants of the tree.
    type Inner:Send;

    ///The number trait used to compare rectangles to
    ///find colliding pairs.
    type Num:NumTrait;

    ///Destructure into the bounding box and mutable parts.
    fn get_mut<'a>(&'a mut self)->(&'a Rect<Self::Num>,&'a mut Self::Inner);

    ///Just get the bounding box.
    fn get<'a>(&'a self)->(&'a Rect<Self::Num>,&'a Self::Inner);
}


///This is the functionality that the collision systems in this crate provide.
///Trait that hides the Axis trait specialization
pub trait DynTreeTrait{
   type T:SweepTrait<Num=Self::Num>;
   type Num:NumTrait;
   fn for_all_in_rect<F:FnMut(ColSingle<Self::T>)>(&mut self,rect:&axgeom::Rect<Self::Num>,fu:&mut F);

   fn for_every_col_pair_seq<H:DepthLevel,F:Bleek<T=Self::T>>
        (&mut self,clos:&mut F,timer:&mut TreeTimer);
   
   fn for_every_col_pair<H:DepthLevel,F:BleekSync<T=Self::T>>
        (&mut self,clos:&F,timer:&mut TreeTimer);
}




///This contains the destructured SweepTrait for a colliding pair.
///The rect is read only while T::Inner is allowed to be mutated.
pub struct ColPair<'a,T:SweepTrait+'a>{
    pub a:(&'a Rect<T::Num>,&'a mut T::Inner),
    pub b:(&'a Rect<T::Num>,&'a mut T::Inner)
}

///Similar to ColPair, but for only one SweepTrait
pub struct ColSingle<'a,T:SweepTrait+'a>(pub &'a Rect<T::Num>,pub &'a mut T::Inner);



//internally,index 0 represents the bottom of the tree. or the heighest depth.
//the last index is the depth 0.
//this reverse ordering is used so that smaller and smaller vecs
//can be allocated and added back together for children nodes.
//TODO no need to reverse!!!
///This is used to measure the real time taken to process each level of the tree.
pub struct TreeTimer{
    height:usize,
    a:Vec<f64>
}




pub struct Bag{
    a:Vec<f64>
}
pub struct TreeTimer2{
    a:Vec<f64>,
    index:usize,
    timer:tools::Timer2
}
impl TreeTimer2{
    
    pub fn new(height:usize)->TreeTimer2{
        let v=(0..height).map(|_|0.0).collect();
        let timer=tools::Timer2::new();
        TreeTimer2{a:v,index:0,timer}
    }

    pub fn leaf_finish(self)->Bag{
       let TreeTimer2{mut a,index,timer}=self;
        a[index]+=timer.elapsed();
        Bag{a:a}
    }

    pub fn next(self)->(TreeTimer2,TreeTimer2){
        let TreeTimer2{mut a,index,timer}=self;
        a[index]+=timer.elapsed();
    
        let b=(0..a.len()).map(|_|0.0).collect();
        (
            TreeTimer2{a:a,index:index+1,timer:tools::Timer2::new()},
            TreeTimer2{a:b,index:index+1,timer:tools::Timer2::new()}
        )
    }

    pub fn combine(mut a:Bag,b:Bag)->Bag{
        for (i,j) in a.a.iter_mut().zip(b.a.iter()){
            *i+=j;
        }
        a
    }
}


impl TreeTimer{
    
    fn create_timer()->tools::Timer2{
        tools::Timer2::new()
    }
    fn add_to_depth(&mut self,depth:usize,time:f64){
        let height=self.height;
        let k=&mut self.a[height-1-depth];
        *k+=time;
    }
    fn combine_one_less(&mut self,v:TreeTimer){
        assert!(self.a.len()==1+v.a.len());

        let a=self.a.split_last_mut().unwrap().1;
        let b=&v.a;
        for (i,j) in a.iter_mut().zip(b.iter()){
            *i+=j;
        }
    }
    fn clone_one_less_depth(&mut self)->TreeTimer{
        let mut v=Vec::new();
        let ln=self.a.len()-1;
        v.resize(ln,0.0);

        TreeTimer{a:v,height:self.height}
    }
    pub fn new(height:usize)->TreeTimer{
        let mut a=Vec::new();
        a.resize(height,0.0);
        TreeTimer{a:a,height:height}
    }
    
    ///Returns the time each level of the tree took to compute.
    pub fn into_time_and_bots(mut self)->Vec<f64>{
        self.a.reverse();
        self.a
    }
    
}





#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
