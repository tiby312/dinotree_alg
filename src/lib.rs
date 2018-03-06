#![feature(iterator_step_by)]


extern crate axgeom;
extern crate compt;
extern crate rayon;
extern crate pdqselect;
extern crate ordered_float;
#[cfg(test)]
extern crate rand;
extern crate smallvec;
extern crate dinotree_inner;

mod inner_prelude{
  pub use dinotree_inner::prelude::*;
  pub use AABBox;
  pub use axgeom::Axis;
  pub use compt::LevelIter;
  pub use compt::LevelDesc;
  pub use axgeom::Range;
  pub use *;
  pub use oned::Sweeper;
  pub use compt::CTreeIterator;
  pub use par;
  pub use axgeom::AxisTrait;
  pub use std::marker::PhantomData;
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
  pub use dinotree_inner::prelude::*;
  pub use ColPair;
  pub use ColSingle;
  pub use DinoTree;
  pub use Rects;
  //pub use TreeCache2;
  pub use RectsTreeTrait;
}


///Provides functionality to draw the dividers of a dinotree.
pub mod graphics;

///Contains convenience structs.
pub mod support;

///Contains query code
mod colfind;

///A collection of 1d functions that operate on lists of 2d objects.
mod oned;

///Contains misc tools
mod tools;


pub use dinotree_inner::prelude::*;
//use inner_prelude::*;
//use dinotree_inner::TreeCache;
use compt::LevelDesc;
use axgeom::Rect;

use axgeom::XAXIS_S;
use axgeom::YAXIS_S;
use dinotree_inner::DivNode;
use colfind::ColMulti;
use colfind::ColSeq;
use colfind::ColSing;

///This contains the destructured SweepTrait for a colliding pair.
///The rect is read only while T::Inner is allowed to be mutated.
pub struct ColPair<'a,T:SweepTrait+'a>{
    pub a:(&'a AABBox<T::Num>,&'a mut T::Inner),
    pub b:(&'a AABBox<T::Num>,&'a mut T::Inner)
}

///Similar to ColPair, but for only one SweepTrait
pub struct ColSingle<'a,T:SweepTrait+'a>(pub &'a AABBox<T::Num>,pub &'a mut T::Inner);



use dinotree_inner::DynTree;
//use dinotree_inner::DefaultDepthLevel;





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


//pub use ba::TreeCache2;
pub use ba::DinoTree;
pub(crate) use ba::DynTreeEnum;
pub use ba::compute_tree_height;
mod ba{
  use super::*;
  use DynTree;
  //use dinotree_inner::TreeCache;
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
          >{
          a:&'a F,
          p:PhantomData<T>
      }

      impl
      <
          'a,
          T:SweepTrait<Inner=I>,
          I:Send+Sync,
          F:Fn(ColPair<T>)+Send+Sync,
          > ColMultiStruct<'a,T,I,F>{
          pub fn new(a:&'a F)->ColMultiStruct<'a,T,I,F>{
              ColMultiStruct{a,p:PhantomData}
          }
      }


      impl
      <
          'a,
          T:SweepTrait<Inner=I>,
          I:Send+Sync,
          F:Fn(ColPair<T>)+Send+Sync,
          >Copy for ColMultiStruct<'a,T,I,F>{
          
      }

      impl
      <
          'a,
          T:SweepTrait<Inner=I>,
          I:Send+Sync,
          F:Fn(ColPair<T>)+Send+Sync,
          >Clone for ColMultiStruct<'a,T,I,F>{
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
          >ColMulti for ColMultiStruct<'a,T,I,F>{

          type T=T;
          fn collide(&self,a:ColPair<T>){
              (self.a)(a);
          }

      } 
  }

  /*
  enum TreeCacheEnum<T:NumTrait>{
    Xa(TreeCache<XAXIS_S,T>),
    Ya(TreeCache<YAXIS_S,T>)
  }
  */

  /*
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
          &mut self,rest:&'a mut [TT],medianstrat:&Z)->(DinoTree<'a,TT>,K::Bag){

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

        (DinoTree(d.0),d.1)
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
    /*
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
    */
    /*
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
    */ 

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
  */


  pub(crate) enum DynTreeEnum<'a,T:SweepTrait+'a>{
    Xa(DynTree<'a,XAXIS_S,T>),
    Ya(DynTree<'a,YAXIS_S,T>)
  }

  pub struct DinoTree<'a,T:SweepTrait+'a>(pub(crate) DynTreeEnum<'a,T>);

  impl <'a,T:SweepTrait+'a> RectsTreeTrait for DinoTree<'a,T>{
      type T=T;
      type Num=T::Num;

      fn for_all_in_rect<F:FnMut(ColSingle<T>)>(&mut self,rect:&AABBox<T::Num>,fu:F){
        DinoTree::for_all_in_rect(self,rect,fu);
      }
  }





  pub fn compute_tree_height(num_bots: usize) -> usize {
      
      //we want each node to have space for around 300 bots.
      //there are 2^h nodes.
      //2^h*200>=num_bots.  Solve for h s.t. h is an integer.

      const NUM_PER_NODE: usize = 10;
      if num_bots <= NUM_PER_NODE {
          return 1;
      } else {
          return (num_bots as f32 / NUM_PER_NODE as f32).log2().ceil() as usize;
      }
  }


  impl<'a,T:SweepTrait+'a> DinoTree<'a,T>{
      

    pub fn new<JJ:par::Joiner,K:TreeTimerTrait>(
          rest:&'a mut [T],axis:axgeom::Axis)->(DinoTree<'a,T>,K::Bag){
        let height=self::compute_tree_height(rest.len());
        if axis==daxis::XAXIS{
            let k=DynTree::<XAXIS_S,T>::new::<JJ,DefaultDepthLevel,K>(rest,height);
            (DinoTree(DynTreeEnum::Xa(k.0)),k.1)
          
        }else{
              let k=DynTree::<YAXIS_S,T>::new::<JJ,DefaultDepthLevel,K>(rest,height);
              (DinoTree(DynTreeEnum::Ya(k.0)),k.1)    
          
        }
          
        //TODO remove this
        //assert_invariant(&d);

        //(DinoTree(d.0),d.1)
     }

      pub fn rects<'b>(&'b mut self)->Rects<'b,Self>{
          Rects{tree:self,rects:Vec::new()}
      }

      /*
       pub(crate) fn get_tree(&self)->&DynTree{
        match &self.0{
          &DynTreeEnum::Xa(ref a)=>{
            //unsafe{std::mem::transmute(a.get_tree())}
            a.get_tree()
          },
          &DynTreeEnum::Ya(ref a)=>{
            //unsafe{std::mem::transmute(a.get_tree())}
            a.get_tree()
          }
         }

      }
      */
      /*
    pub(crate) fn get_axis(&self)->axgeom::Axis{
       match &self.0{
        &DynTreeEnum::Xa(_)=>{
          axgeom::XAXIS
        },
        &DynTreeEnum::Xa(_)=>{
          axgeom::YAXIS
        }
       }
    }

*/
      fn for_all_in_rect<F:FnMut(ColSingle<T>)>(&mut self,rect:&AABBox<T::Num>,fu:F){
        let fu=self::closure_struct::ColSingStruct::new(fu);
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
      pub fn kth_nearest<F:FnMut(ColSingle<T>)>(&mut self,_clos:F,_point:(T::Num,T::Num),_num:usize){
        unimplemented!();
      }

      pub fn for_every_col_pair_seq<F:FnMut(ColPair<T>),K:TreeTimerTrait>
          (&mut self,clos:F)->K::Bag{     
          let clos=self::closure_struct::ColSeqStruct::new(clos);

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
        K:TreeTimerTrait>(&mut self,a:F)->K::Bag{
          
          let clos=self::closure_struct::ColMultiStruct::new(&a);
          
          match &mut self.0{
            &mut DynTreeEnum::Xa(ref mut a)=>{
              colfind::for_every_col_pair::<_,T,DefaultDepthLevel,_,K>(a,clos)
            },
            &mut DynTreeEnum::Ya(ref mut a)=>{
              colfind::for_every_col_pair::<_,T,DefaultDepthLevel,_,K>(a,clos)
            }
          }

      }
      
  }


}



/*
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
*/



//Pub so benches can access
#[cfg(test)]
mod test_support;

#[cfg(test)]
mod dinotree_test;
