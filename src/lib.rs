#![feature(iterator_step_by)]

extern crate axgeom;
extern crate compt;
extern crate rayon;
extern crate pdqselect;
extern crate ordered_float;
extern crate rand;
extern crate smallvec;


///Contains rebalancing code.
mod base_kdtree;
///Provides low level functionality to construct a dyntree.
mod tree_alloc;
///Contains query code
mod colfind;
///Contains code to construct the dyntree.
///Main property is that the nodes and the bots are all copied into one
///segment of memory. 
mod dyntree;
///A collection of 1d functions that operate on lists of 2d objects.
mod oned;

///Provides functionality to draw the dividers of a dinotree.
pub mod graphics;
///Contains the different median finding strategies.
pub mod median;
///Contains conveniance structs.
pub mod support;
///Contains code to query multiple non intersecting rectangles.
pub mod multirect;
///Contains tree level by level timing collection code. 
pub mod treetimer;
///Contains misc tools
pub mod tools;


pub use base_kdtree::TreeCache;
use compt::LevelDesc;
use axgeom::Rect;
use treetimer::*;


///Returns the level at which a parallel divide and conqur algorithm will switch to sequential
pub trait DepthLevel{
    ///Switch to sequential at this height.
    fn switch_to_sequential(a:LevelDesc)->bool;
}

///The underlying number type used for the bounding boxes,
///and for the dividers. 
pub trait NumTrait:Ord+Copy+Send+Sync+std::fmt::Debug+Default{}


//TODO move this to a closure passed to ColFind i think.
pub trait ColFindAdd:Send+Sync{
    fn identity()->Self;
    fn add(&mut self,&Self);
}

pub trait InnerRect:Send+Sync{
  type Num:NumTrait;
  fn get(&self)->&Rect<Self::Num>;
}

///The interface through which the tree interacts with the objects being inserted into it.
pub trait SweepTrait:Send+Sync{

    type InnerRect:InnerRect<Num=Self::Num>;
    ///The part of the object that is allowed to be mutated
    ///during the querying of the tree. It is important that
    ///the bounding boxes not be mutated during querying of the tree
    ///as that would break the invariants of the tree. (it might need to be moved
    ///to a different node)
    type Inner:ColFindAdd;

    ///The number trait used to compare rectangles to
    ///find colliding pairs.
    type Num:NumTrait;


    ///Destructure into the bounding box and mutable parts.
    fn get_mut<'a>(&'a mut self)->(&'a Self::InnerRect,&'a mut Self::Inner);

    ///Destructue into the bounding box and inner part.
    fn get<'a>(&'a self)->(&'a Self::InnerRect,&'a Self::Inner);
    
    /*
    ///Destructure into the bounding box and mutable parts.
    fn get_mut<'a>(&'a mut self)->(&'a Rect<Self::Num>,&'a mut Self::Inner);

    ///Destructue into the bounding box and inner part.
    fn get<'a>(&'a self)->(&'a Rect<Self::Num>,&'a Self::Inner);
    */
}

///The interface through which users can use the tree for what it is for, querying.
pub trait DynTreeTrait{
   type T:SweepTrait<Num=Self::Num>;
   type Num:NumTrait;

   ///Finds all objects strictly within the specified rectangle.
   fn for_all_in_rect<F:FnMut(ColSingle<Self::T>)>(&mut self,rect:&axgeom::Rect<Self::Num>,fu:&mut F);

   ///Find all objects who's bounding boxes intersect in parallel.
   fn for_every_col_pair<H:DepthLevel,F:Fn(ColPair<Self::T>)+Sync,K:TreeTimerTrait>
        (&mut self,clos:F)->K::Bag;

   ///Find all objects who's bounding boxes intersect sequentially. 
   fn for_every_col_pair_seq<F:FnMut(ColPair<Self::T>),K:TreeTimerTrait>
        (&mut self,clos:F)->K::Bag;
}



use axgeom::AxisTrait;
use dyntree::DynTree;
use median::MedianStrat;
use support::DefaultDepthLevel;
use oned::sup::BleekBF;
use oned::sup::BleekSF;
use tools::par;




//Note this doesnt check all invariants.
//e.g. doesnt check that every bot is in the tree only once.
pub fn assert_invariant<A:AxisTrait,T:SweepTrait>(d:&DinoTree<A,T>){
    
    let level=d.0.get_level_desc();
    let ll=compt::LevelIter::new(d.0.get_iter(),level);
    use compt::CTreeIterator;
    for (level,node) in ll.dfs_preorder_iter(){
       
       //println!("level={:?}",level.get_depth());
       if level.get_depth()%2==0{
          oned::is_sorted::<A::Next,_>(&node.range);


          let kk=node.container_box;
          for a in node.range.iter(){
             let (p1,p2)=(
                  a.get().0.get().get_range2::<A>().left(),
                  a.get().0.get().get_range2::<A>().right());
              assert!(kk.left()<=p1);
              assert!(p2<=kk.right());
          }
       }else{
          oned::is_sorted::<A,_>(&node.range);
          
          let kk=node.container_box;
          for a in node.range.iter(){
             let (p1,p2)=(
                  a.get().0.get().get_range2::<A::Next>().left(),
                  a.get().0.get().get_range2::<A::Next>().right());
              assert!(kk.left()<=p1);
              assert!(p2<=kk.right());
          }
       }
    }       
    
}

///The struct that this crate revolves around.
pub struct DinoTree<'a,A:AxisTrait,T:SweepTrait+'a>(
  DynTree<'a,A,T>
  );

impl<'a,A:AxisTrait,T:SweepTrait+'a> DinoTree<'a,A,T>{
   pub fn new<JJ:par::Joiner,H:DepthLevel,Z:MedianStrat<Num=T::Num>,K:TreeTimerTrait>(
        rest:&'a mut [T],tc:&mut TreeCache<A,T::Num>,medianstrat:&Z) -> (DinoTree<'a,A,T>,K::Bag) {
      let k=DynTree::new::<JJ,H,Z,K>(rest,tc,medianstrat);
      
      let d=DinoTree(k.0);

      //TODO remove this
      //assert_invariant(&d);

      (d,k.1)

  }
}


impl<'a,A:AxisTrait,T:SweepTrait+'a> DynTreeTrait for DinoTree<'a,A,T>{
    type T=T;
    type Num=T::Num;
    

    fn for_all_in_rect<F:FnMut(ColSingle<Self::T>)>(&mut self,rect:&axgeom::Rect<Self::Num>,fu:&mut F){
        colfind::for_all_in_rect(&mut self.0,rect,fu);
    }

    fn for_every_col_pair_seq<F:FnMut(ColPair<Self::T>),K:TreeTimerTrait>
        (&mut self,mut clos:F)->K::Bag{
        let mut bb=BleekSF::new(&mut clos);            
        colfind::for_every_col_pair_seq::<_,T,DefaultDepthLevel,_,K>(&mut self.0,&mut bb)
    }
    fn for_every_col_pair<H:DepthLevel,F:Fn(ColPair<Self::T>)+Sync,K:TreeTimerTrait>
        (&mut self,clos:F)->K::Bag{
        let bb=BleekBF::new(&clos);                            
        colfind::for_every_col_pair::<_,T,H,_,K>(&mut self.0,&bb)
    }
}

mod test_support{
  use axgeom;
  use support::Numisize;
  use std;
  use rand;
  use rand::{ SeedableRng, StdRng};
  use rand::distributions::{IndependentSample, Range};
    

  #[derive(Clone,Debug)]
  pub struct Bot{
    pub id:usize,
    pub col:Vec<usize>
  }

  pub fn make_rect(a:(isize,isize),b:(isize,isize))->axgeom::Rect<Numisize>{
    axgeom::Rect::new(
      Numisize(a.0),
      Numisize(a.1),
      Numisize(b.0),
      Numisize(b.1),
     )
  }

  pub fn create_rect_from_point(a:(Numisize,Numisize))->axgeom::Rect<Numisize>{
    let r:isize=10;
    let x=a.0;
    let y=a.1;
    make_rect((x.0-r,x.0+r),(y.0-r,y.0+r))
  }
  pub fn create_unordered(a:&Bot,b:&Bot)->(usize,usize){
    if a.id<b.id{
      (a.id,b.id)
    }else{
      (b.id,a.id)
    }
  }
  pub fn compair_bot_pair(a:&(usize,usize),b:&(usize,usize))->std::cmp::Ordering{
      if a.0<b.0{
          std::cmp::Ordering::Less
      }else if a.0>b.0{
          std::cmp::Ordering::Greater
      }else{
          if a.1<b.1{
              std::cmp::Ordering::Less
          }else if a.1>b.1{
              std::cmp::Ordering::Greater
          }else{
              std::cmp::Ordering::Equal
          }
      }
  }


    pub struct PointGenerator{
        rng:StdRng,
        xdist:Range<isize>,
        ydist:Range<isize>
    }
    impl PointGenerator{
      pub fn new(a:&axgeom::Rect<Numisize>,seed:&[usize])->PointGenerator{
         use rand::distributions::IndependentSample;
    
         let mut rng: StdRng = SeedableRng::from_seed(seed);

         let rr=a.get_range2::<axgeom::XAXIS_S>();
         let xdist=rand::distributions::Range::new(rr.start.0,rr.end.0);
         
         let rr=a.get_range2::<axgeom::YAXIS_S>();
         let ydist=rand::distributions::Range::new(rr.start.0,rr.end.0);

         PointGenerator{rng,xdist,ydist}
      }
      pub fn random_point(&mut self)->(Numisize,Numisize){
          (
            Numisize(self.xdist.ind_sample(&mut self.rng)),
            Numisize(self.ydist.ind_sample(&mut self.rng))
          )
      }
    }
}


///This contains the destructured SweepTrait for a colliding pair.
///The rect is read only while T::Inner is allowed to be mutated.
//TODO change name to stat and dyn.
pub struct ColPair<'a,T:SweepTrait+'a>{
    pub a:(&'a T::InnerRect,&'a mut T::Inner),
    pub b:(&'a T::InnerRect,&'a mut T::Inner)
}

///Similar to ColPair, but for only one SweepTrait
pub struct ColSingle<'a,T:SweepTrait+'a>(pub &'a T::InnerRect,pub &'a mut T::Inner);




#[cfg(test)]
mod dinotree_test;
