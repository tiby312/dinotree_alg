#![feature(iterator_step_by)]


extern crate axgeom;
extern crate compt;
extern crate rayon;
extern crate pdqselect;
extern crate ordered_float;
#[cfg(test)]
extern crate rand;
extern crate smallvec;


mod inner_prelude{
  pub use AABBox;
  pub use axgeom::Axis;
  pub use compt::LevelIter;
  pub use compt::LevelDesc;
  pub use axgeom::Range;
  pub use *;
  pub use oned::Sweeper;
  pub use super::median::MedianStrat;
  pub use compt::CTreeIterator;
  pub use par;
  pub use axgeom::AxisTrait;
  pub use std::marker::PhantomData;
  pub use treetimer::*;
  pub use NumTrait;
  pub use *;
}

/// Conveniently include commonly used symbols in this crate.
/// Use like this:
/// ```
/// extern crate dinotree;
/// use dinotree::prelude::*;
/// fn main(){
///    //...
/// }
/// ```
pub mod prelude{
  pub use daxis;
  pub use AABBox;
  pub use ColPair;
  pub use ColSingle;
  pub use DinoTree2;
  pub use Rects;
  pub use TreeCache2;
  pub use DepthLevel;
  pub use NumTrait;
  pub use RectsTreeTrait;
  pub use SweepTrait;
  pub use median::*;
  pub use median::relax::*;
  pub use median::strict::*;
  pub use par;
  pub use treetimer;
}


///Provides functionality to draw the dividers of a dinotree.
pub mod graphics;
///Contains the different median finding strategies.
pub mod median;

///Contains convenience structs.
pub mod support;

///Contains tree level by level timing collection code. 
pub mod treetimer;

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

///Contains misc tools
mod tools;

use base_kdtree::TreeCache;
use compt::LevelDesc;
use axgeom::Rect;
use treetimer::*;
use axgeom::XAXIS_S;
use axgeom::YAXIS_S;
use base_kdtree::DivNode;
use colfind::ColMulti;
use colfind::ColSeq;
use colfind::ColSing;

///Returns the level at which a parallel divide and conqur algorithm will switch to sequential
pub trait DepthLevel{
    ///Switch to sequential at this height.
    fn switch_to_sequential(a:LevelDesc)->bool;
}

///The underlying number type used for the bounding boxes,
///and for the dividers. 
pub trait NumTrait:Ord+Copy+Send+Sync+std::fmt::Debug+Default{}


#[derive(Copy,Clone)]
pub struct AABBox<N:NumTrait>(axgeom::Rect<N>);
impl<N:NumTrait> AABBox<N>{

  ///For both axises, the first value must be less than the second.
  pub fn new(xaxis:(N,N),yaxis:(N,N))->AABBox<N>{
    AABBox(axgeom::Rect::new(xaxis.0,xaxis.1,yaxis.0,yaxis.1))
  }
  pub fn get(&self)->((N,N),(N,N)){
    let a=self.0.get_range2::<XAXIS_S>();
    let b=self.0.get_range2::<YAXIS_S>();
    ((a.start,a.end),(b.start,b.end))
  }
}


pub mod daxis{
  pub use axgeom::Axis as DAxis;
  pub use axgeom::XAXIS;
  pub use axgeom::YAXIS;
}

impl<N:NumTrait> std::fmt::Debug for AABBox<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let (xx,yy)=self.get();
        write!(f, "AABBox {{ xaxis: {:?}, yaxis: {:?} }}", xx, yy)
    }
}

///The interface through which the tree interacts with the objects being inserted into it.
pub trait SweepTrait:Send+Sync{
    ///The part of the object that is allowed to be mutated
    ///during the querying of the tree. It is important that
    ///the bounding boxes not be mutated during querying of the tree
    ///as that would break the invariants of the tree. (it might need to be moved
    ///to a different node)
    type Inner:Send+Sync;

    ///The number trait used to compare rectangles to
    ///find colliding pairs.
    type Num:NumTrait;


    ///Destructure into the bounding box and mutable parts.
    fn get_mut<'a>(&'a mut self)->(&'a AABBox<Self::Num>,&'a mut Self::Inner);

    ///Destructue into the bounding box and inner part.
    fn get<'a>(&'a self)->(&'a AABBox<Self::Num>,&'a Self::Inner);
}


///This contains the destructured SweepTrait for a colliding pair.
///The rect is read only while T::Inner is allowed to be mutated.
pub struct ColPair<'a,T:SweepTrait+'a>{
    pub a:(&'a AABBox<T::Num>,&'a mut T::Inner),
    pub b:(&'a AABBox<T::Num>,&'a mut T::Inner)
}

///Similar to ColPair, but for only one SweepTrait
pub struct ColSingle<'a,T:SweepTrait+'a>(pub &'a AABBox<T::Num>,pub &'a mut T::Inner);



use axgeom::AxisTrait;
use dyntree::DynTree;
use median::MedianStrat;
use support::DefaultDepthLevel;





pub trait RectsTreeTrait{
    type T:SweepTrait;
    type Num:NumTrait;
    fn for_all_in_rect<F:FnMut(ColSingle<Self::T>)>(&mut self,rect:&AABBox<Self::Num>,fu:F);
}

///A construct to allow querying non-intersecting rectangles to retrive mutable references to what is inside them.
pub struct Rects<'a,C:RectsTreeTrait+'a>{
    tree:&'a mut C,
    rects:Vec<AABBox<C::Num>>
}


impl<'a,C:RectsTreeTrait+'a> Rects<'a,C>{

    ///Iterate over all bots in a rectangle.
    ///It is safe to call this function multiple times with rectangles that 
    ///do not intersect. Because the rectangles do not intersect, all bots retrieved
    ///from inside either rectangle are guarenteed to be disjoint. 
    ///If a rectangle is passed that does intersect one from a previous call, this function will panic.
    ///
    ///Note the lifetime of the mutable reference in the passed function.
    ///The user is allowed to move this reference out and hold on to it for 
    ///the lifetime of this struct.
    pub fn for_all_in_rect<F:FnMut(ColSingle<'a,C::T>)>(&mut self,rect:&AABBox<C::Num>,mut func:F){
    

        
        for k in self.rects.iter(){
            if rect.0.intersects_rect(&k.0){
                panic!("Rects cannot intersect! {:?}",(k,rect));
            }
        }

        self.rects.push(AABBox(rect.0));

        {
            let wrapper=|c:ColSingle<C::T>|{
                let (a,b)=(c.0 as *const AABBox<<C::T as SweepTrait>::Num>,c.1 as *mut <C::T as SweepTrait>::Inner);
                //Unsafely extend the lifetime to accocomate the
                //lifetime of RectsTrait.
                let (a,b)=unsafe{(&*a,&mut *b)};
                
                let cn=ColSingle(a,b);
                func(cn);
            };
            self.tree.for_all_in_rect(rect,wrapper);
        }
        
    }
}


pub mod par{
    use rayon;
    pub trait Joiner{

        fn join<A:FnOnce() -> RA + Send,RA:Send,B:FnOnce() -> RB + Send,RB:Send>(oper_a: A, oper_b: B) -> (RA, RB);
        fn is_parallel()->bool;
    }

    pub struct Parallel;
    impl Joiner for Parallel{
        fn is_parallel()->bool{
            return true;
        }

        fn join<A:FnOnce() -> RA + Send,RA:Send,B:FnOnce() -> RB + Send,RB:Send>(oper_a: A, oper_b: B) -> (RA, RB)   {
          rayon::join(oper_a, oper_b)
        }
    }
    pub struct Sequential;
    impl Joiner for Sequential{
        fn is_parallel()->bool{
            return false;
        }
        fn join<A:FnOnce() -> RA + Send,RA:Send,B:FnOnce() -> RB + Send,RB:Send>(oper_a: A, oper_b: B) -> (RA, RB)   {
            let a = oper_a();
            let b = oper_b();
            (a, b)
        }
    }
}


pub use ba::TreeCache2;
pub use ba::DinoTree2;
mod ba{
  use super::*;
  use DynTree;
  use base_kdtree::TreeCache;
  use RectsTreeTrait;

  mod closure_struct{
      use super::*;
      use ColPair;
      use std::marker::PhantomData;
      use ColSeq;
      use ColSingle;
      use ColSing;
      use ColMulti;

      pub struct ColSeqStruct<T:SweepTrait,F:FnMut(ColPair<T>)>{
          d:F,
          p:PhantomData<T>
      }
      impl<T:SweepTrait,F:FnMut(ColPair<T>)> ColSeqStruct<T,F>{
          pub fn new(a:F)->ColSeqStruct<T,F>{
              ColSeqStruct{d:a,p:PhantomData}
          }
      }
      impl<T:SweepTrait,F:FnMut(ColPair<T>)> ColSeq for ColSeqStruct<T,F>{
          type T=T;
          fn collide(&mut self,a:ColPair<Self::T>){
              (self.d)(a);
          }
      }
      
      pub struct ColSingStruct<T:SweepTrait,F:FnMut(ColSingle<T>)>{
          d:F,
          p:PhantomData<T>
      }
      impl<T:SweepTrait,F:FnMut(ColSingle<T>)> ColSingStruct<T,F>{
          pub fn new(a:F)->ColSingStruct<T,F>{
              ColSingStruct{d:a,p:PhantomData}
          }
      }
      impl<T:SweepTrait,F:FnMut(ColSingle<T>)> ColSing for ColSingStruct<T,F>{
          type T=T;
          fn collide(&mut self,a:ColSingle<Self::T>){
              (self.d)(a);
          }
      }
      
      
      pub struct ColMultiStruct<'a,
          T:SweepTrait<Inner=I>,
          I:Send+Sync,
          F:Fn(ColPair<T>)+Send+Sync+'a,
          F2:Fn(&T)->T+Send+Sync+'a,
          F3:Fn(&mut I,&mut I)+Send+Sync+'a
          >{
          a:&'a F,
          b:&'a F2,
          c:&'a F3,
          p:PhantomData<T>
      }

      impl
      <
          'a,
          T:SweepTrait<Inner=I>,
          I:Send+Sync,
          F:Fn(ColPair<T>)+Send+Sync,
          F2:Fn(&T)->T+Send+Sync+'a,
          F3:Fn(&mut I,&mut I)+Send+Sync
          > ColMultiStruct<'a,T,I,F,F2,F3>{
          pub fn new(a:&'a F,b:&'a F2,c:&'a F3)->ColMultiStruct<'a,T,I,F,F2,F3>{
              ColMultiStruct{a,b,c,p:PhantomData}
          }
      }


      impl
      <
          'a,
          T:SweepTrait<Inner=I>,
          I:Send+Sync,
          F:Fn(ColPair<T>)+Send+Sync,
          F2:Fn(&T)->T+Send+Sync+'a,
          F3:Fn(&mut I,&mut I)+Send+Sync
          >Copy for ColMultiStruct<'a,T,I,F,F2,F3>{
          
      }

      impl
      <
          'a,
          T:SweepTrait<Inner=I>,
          I:Send+Sync,
          F:Fn(ColPair<T>)+Send+Sync,
          F2:Fn(&T)->T+Send+Sync+'a,
          F3:Fn(&mut I,&mut I)+Send+Sync
          >Clone for ColMultiStruct<'a,T,I,F,F2,F3>{
          fn clone(&self)->Self{
              *self
          }
      }

      impl
      <
          'a,
          T:SweepTrait<Inner=I>,
          I:Send+Sync,
          F:Fn(ColPair<T>)+Send+Sync,
          F2:Fn(&T)->T+Send+Sync+'a,
          F3:Fn(&mut I,&mut I)+Send+Sync
          >ColMulti for ColMultiStruct<'a,T,I,F,F2,F3>{

          type T=T;
          fn identity(&self,src:&T)->T{
              (self.b)(src)
          }
          fn add(&self,a:&mut I,b:&mut I){
              (self.c)(a,b);
          }
          fn collide(&self,a:ColPair<T>){
              (self.a)(a);
          }

      } 
  }


  enum TreeCacheEnum<T:NumTrait>{
    Xa(TreeCache<XAXIS_S,T>),
    Ya(TreeCache<YAXIS_S,T>)
  }

  pub struct TreeCache2<T:NumTrait>(TreeCacheEnum<T>);

  impl<T:NumTrait> TreeCache2<T>{
    
    ///It's the user's responsibility to pick a "good" height.
    ///The distribution and number of bots matter.
    ///Ideally you want every node to have around 10 elements in it.
    ///Here's a good heuristic
    ///log2(num_bots/num_bots_per_node)
    pub fn new(axis:axgeom::Axis,height:usize)->TreeCache2<T>{
      let a=if axis==axgeom::XAXIS{
        TreeCacheEnum::Xa(TreeCache::<XAXIS_S,T>::new(height))
      }else{
        TreeCacheEnum::Ya(TreeCache::<YAXIS_S,T>::new(height))
      };
      TreeCache2(a)
    }

    pub fn new_tree<'a,TT:SweepTrait<Num=T>,JJ:par::Joiner,H:DepthLevel,Z:MedianStrat<Num=T>,K:TreeTimerTrait>(
          &mut self,rest:&'a mut [TT],medianstrat:&Z)->(DinoTree2<'a,TT>,K::Bag){

        let d=match &mut self.0{
          &mut TreeCacheEnum::Xa(ref mut a)=>{
            let k=DynTree::<XAXIS_S,TT>::new::<JJ,H,Z,K>(rest,a,medianstrat);
            (DynTreeEnum::Xa(k.0),k.1)
          },
          &mut TreeCacheEnum::Ya(ref mut a)=>{
            let k=DynTree::<YAXIS_S,TT>::new::<JJ,H,Z,K>(rest,a,medianstrat);
            (DynTreeEnum::Ya(k.0),k.1)
          }
        };

        //TODO remove this
        //assert_invariant(&d);

        (DinoTree2(d.0),d.1)
     }
    
    pub(crate) fn get_tree(&self)->&compt::GenTree<DivNode<T>>{
      match &self.0{
        &TreeCacheEnum::Xa(ref a)=>{
          //unsafe{std::mem::transmute(a.get_tree())}
          a.get_tree()
        },
        &TreeCacheEnum::Ya(ref a)=>{
          //unsafe{std::mem::transmute(a.get_tree())}
          a.get_tree()
        }
       }

    }

    pub(crate) fn get_num_nodes(&self)->usize{
        match &self.0{
        &TreeCacheEnum::Xa(ref a)=>{
          a.get_num_nodes()
        },
        &TreeCacheEnum::Ya(ref a)=>{
          a.get_num_nodes()
        }
       }
    }

    pub(crate) fn get_height(&self)->usize{
        match &self.0{
        &TreeCacheEnum::Xa(ref a)=>{
          a.get_height()
        },
        &TreeCacheEnum::Ya(ref a)=>{
          a.get_height()
        }
       }
    }  

    pub(crate) fn get_axis(&self)->axgeom::Axis{
       match &self.0{
        &TreeCacheEnum::Xa(_)=>{
          axgeom::XAXIS
        },
        &TreeCacheEnum::Ya(_)=>{
          axgeom::YAXIS
        }
       }
    }
  }


  enum DynTreeEnum<'a,T:SweepTrait+'a>{
    Xa(DynTree<'a,XAXIS_S,T>),
    Ya(DynTree<'a,YAXIS_S,T>)
  }

  pub struct DinoTree2<'a,T:SweepTrait+'a>(DynTreeEnum<'a,T>);

  impl <'a,T:SweepTrait+'a> RectsTreeTrait for DinoTree2<'a,T>{
      type T=T;
      type Num=T::Num;

      fn for_all_in_rect<F:FnMut(ColSingle<T>)>(&mut self,rect:&AABBox<T::Num>,fu:F){
        DinoTree2::for_all_in_rect(self,rect,fu);
      }
  }
  impl<'a,T:SweepTrait+'a> DinoTree2<'a,T>{
      

      pub fn rects<'b>(&'b mut self)->Rects<'b,Self>{
          Rects{tree:self,rects:Vec::new()}
      }

     
      fn for_all_in_rect<F:FnMut(ColSingle<T>)>(&mut self,rect:&AABBox<T::Num>,fu:F){
        let mut fu=self::closure_struct::ColSingStruct::new(fu);
        match &mut self.0{
          &mut DynTreeEnum::Xa(ref mut a)=>{
            colfind::for_all_in_rect(a,&rect.0,fu);
          },
          &mut DynTreeEnum::Ya(ref mut a)=>{
            colfind::for_all_in_rect(a,&rect.0,fu);
          }
        }
      }
      
      ///Not implemented!
      ///Finds the k nearest bots to a point.
      pub fn kth_nearest<F:FnMut(ColSingle<T>)>(&mut self,mut clos:F,point:(T::Num,T::Num),num:usize){
        unimplemented!();
      }

      pub fn for_every_col_pair_seq<F:FnMut(ColPair<T>),K:TreeTimerTrait>
          (&mut self,mut clos:F)->K::Bag{     
          let mut clos=self::closure_struct::ColSeqStruct::new(clos);

          match &mut self.0{
            &mut DynTreeEnum::Xa(ref mut a)=>{
              colfind::for_every_col_pair_seq::<_,T,DefaultDepthLevel,_,K>(a,clos)
            },
            &mut DynTreeEnum::Ya(ref mut a)=>{
              colfind::for_every_col_pair_seq::<_,T,DefaultDepthLevel,_,K>(a,clos)
            }
          }
      }

      //It is the user responsibility to not change the bounding box
      //That is returned by SweepTrat in the identity() function.
      pub fn for_every_col_pair<
        F:Fn(ColPair<T>)+Send+Sync,
        F2:Fn(&T)->T+Send+Sync,
        F3:Fn(&mut T::Inner,&mut T::Inner)+Send+Sync,
        D:DepthLevel,
        K:TreeTimerTrait>(&mut self,a:F,b:F2,c:F3)->K::Bag{
          
          let mut clos=self::closure_struct::ColMultiStruct::new(&a,&b,&c);
          
          match &mut self.0{
            &mut DynTreeEnum::Xa(ref mut a)=>{
              colfind::for_every_col_pair::<_,T,D,_,K>(a,clos)
            },
            &mut DynTreeEnum::Ya(ref mut a)=>{
              colfind::for_every_col_pair::<_,T,D,_,K>(a,clos)
            }
          }

      }
      
  }


}




///The struct that this crate revolves around.
struct DinoTree<'a,A:AxisTrait,T:SweepTrait+'a>(
  DynTree<'a,A,T>
  );

impl<'a,A:AxisTrait,T:SweepTrait+'a> DinoTree<'a,A,T>{
   fn new<JJ:par::Joiner,H:DepthLevel,Z:MedianStrat<Num=T::Num>,K:TreeTimerTrait>(
        rest:&'a mut [T],tc:&mut TreeCache<A,T::Num>,medianstrat:&Z) -> (DinoTree<'a,A,T>,K::Bag) {
      let k=DynTree::new::<JJ,H,Z,K>(rest,tc,medianstrat);
      
      let d=DinoTree(k.0);

      //TODO remove this
      //assert_invariant(&d);

      (d,k.1)

  }
}



//Pub so benches can access
#[cfg(test)]
mod test_support;

#[cfg(test)]
mod dinotree_test;
