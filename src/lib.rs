extern crate axgeom;
extern crate compt;
extern crate rayon;
extern crate pdqselect;
extern crate ordered_float;
//TODO make this only be included in test
extern crate rand;

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

///The interface through which the tree interacts with the objects being inserted into it.
pub trait SweepTrait:Send{
    ///The part of the object that is allowed to be mutated
    ///during the querying of the tree. It is important that
    ///the bounding boxes not be mutated during querying of the tree
    ///as that would break the invariants of the tree. (it might need to be moved
    ///to a different node)
    type Inner:Send;

    ///The number trait used to compare rectangles to
    ///find colliding pairs.
    type Num:NumTrait;

    ///Destructure into the bounding box and mutable parts.
    fn get_mut<'a>(&'a mut self)->(&'a Rect<Self::Num>,&'a mut Self::Inner);

    ///Destructue into the bounding box and inner part.
    fn get<'a>(&'a self)->(&'a Rect<Self::Num>,&'a Self::Inner);
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



///The struct that this crate revolves around.
pub struct DinoTree<'a,A:AxisTrait,T:SweepTrait+'a>(
  DynTree<'a,A,T>
  );

impl<'a,A:AxisTrait,T:SweepTrait+'a> DinoTree<'a,A,T>{
   pub fn new<JJ:par::Joiner,H:DepthLevel,Z:MedianStrat<Num=T::Num>,K:TreeTimerTrait>(
        rest:&'a mut [T],tc:&mut TreeCache<A,T::Num>,medianstrat:&Z) -> (DinoTree<'a,A,T>,K::Bag) {
      let k=DynTree::new::<JJ,H,Z,K>(rest,tc,medianstrat);
      (DinoTree(k.0),k.1)
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
        colfind::for_every_col_pair_seq::<A,T,DefaultDepthLevel,_,K>(&mut self.0,&mut bb)
    }
    fn for_every_col_pair<H:DepthLevel,F:Fn(ColPair<Self::T>)+Sync,K:TreeTimerTrait>
        (&mut self,clos:F)->K::Bag{
        let bb=BleekBF::new(&clos);                            
        colfind::for_every_col_pair::<A,T,H,_,K>(&mut self.0,&bb)
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


#[test]
fn test_dinotree_drop(){
    use test_support::*;
    use support::BBox;
    use support::Numisize;
    use test_support::make_rect;
    use test_support::create_rect_from_point;
    use median::strict::MedianStrict;
    //use test::black_box;
    struct Bot<'a>{
        id:usize,
        drop_counter:&'a mut isize 
    }

    impl<'a> Drop for Bot<'a>{
       fn drop(&mut self){
          *self.drop_counter-=1;
       }
    }

    let mut drop_counter:Vec<isize>=(0..5000).map(|a|1).collect();
    {
      let mut bots:Vec<BBox<Numisize,Bot>>=Vec::new();


      let world=make_rect((-1000,1000),(-100,100));

      let spawn_world=make_rect((-990,990),(-90,90));

      let mut p=PointGenerator::new(&spawn_world,&[1,2,3,4,5]);

      for (id,dc) in (0..5000).zip(drop_counter.iter_mut()){
          
          let rect=create_rect_from_point(p.random_point());
          let j=BBox::new(Bot{id,drop_counter:dc},rect);
          bots.push(j);
      }


      let height=12;
      use axgeom::XAXIS_S;
      use axgeom::YAXIS_S;
      let mut treecache:TreeCache<XAXIS_S,_>=TreeCache::new(height);

      {
        let k=MedianStrict::<Numisize>::new();
        let (mut dyntree,_bag)=DinoTree::new::<par::Parallel,DefaultDepthLevel,_,treetimer::TreeTimerEmpty>
                        (&mut bots,&mut treecache,&k);
        
        
        let clos=|cc:ColPair<BBox<Numisize,Bot>>|{
            //let a=cc.a;
            //let b=cc.b;
            //src.push(test_support::create_unordered(&a.1,&b.1));
            //a.1.col.push(b.1.id);
            //b.1.col.push(a.1.id);
            //black_box(cc);
        };

        let _v=dyntree.for_every_col_pair_seq::<_,treetimer::TreeTimer2>(clos);
      }     

    }  

    println!("{:?}",drop_counter);
    assert!(drop_counter.iter().fold(true,|acc,&x|acc&(x==0)));
}

#[test]
fn test_dinotree(){
    use test_support::*;
    use axgeom::XAXIS_S;
    use axgeom::YAXIS_S;
    
  use support::Numisize;
    use support::BBox;
    use median::strict::MedianStrict;

    fn make_rect(a:(isize,isize),b:(isize,isize))->axgeom::Rect<Numisize>{
        axgeom::Rect::new(
          Numisize(a.0),
          Numisize(a.1),
          Numisize(b.0),
          Numisize(b.1),
        )
    }

    fn create_rect_from_point(a:(Numisize,Numisize))->axgeom::Rect<Numisize>{
        let r:isize=10;
        let x=a.0;
        let y=a.1;
        make_rect((x.0-r,x.0+r),(y.0-r,y.0+r))
    }

    let world=make_rect((-1000,1000),(-100,100));


    let spawn_world=make_rect((-990,990),(-90,90));

    let mut p=PointGenerator::new(&spawn_world,&[1,2,3,4,5]);

    
    let mut bots:Vec<BBox<Numisize,Bot>>={
        (0..10000).map(|id|{
            let rect=create_rect_from_point(p.random_point());
            BBox::new(Bot{id,col:Vec::new()},rect)
        }).collect()  
    };
    
    
    let mut control_result={
        let mut src:Vec<(usize,usize)>=Vec::new();
        
        let control_bots=bots.clone();
        for (i, el1) in control_bots.iter().enumerate() {
            for el2 in control_bots[i + 1..].iter() {
              
                let a=el1;
                let b=el2;
                let ax=a.get().0.get_range2::<XAXIS_S>();     
                let ay=a.get().0.get_range2::<YAXIS_S>();     
                let bx=b.get().0.get_range2::<XAXIS_S>();     
                let by=b.get().0.get_range2::<YAXIS_S>();     
              
                if ax.intersects(bx) && ay.intersects(by){
                    src.push(test_support::create_unordered(&a.val,&b.val));
                }
            }
        }
        src
    };
    

    let mut test_result={
        let mut src:Vec<(usize,usize)>=Vec::new();
        
        let height=12;
        use axgeom::XAXIS_S;
        use axgeom::YAXIS_S;
        let mut treecache:TreeCache<XAXIS_S,_>=TreeCache::new(height);

        {
          let k=MedianStrict::<Numisize>::new();
          let (mut dyntree,_bag)=DinoTree::new::<par::Parallel,DefaultDepthLevel,_,treetimer::TreeTimerEmpty>
                          (&mut bots,&mut treecache,&k);
          
          let clos=|cc:ColPair<BBox<Numisize,Bot>>|{
              let a=cc.a;
              let b=cc.b;
              src.push(test_support::create_unordered(&a.1,&b.1));
              //a.1.col.push(b.1.id);
              //b.1.col.push(a.1.id);
          };

          let _v=dyntree.for_every_col_pair_seq::<_,treetimer::TreeTimer2>(clos);
        }       

        //println!("{:?}",bots);
        src
    };

    control_result.sort_by(&test_support::compair_bot_pair);
    test_result.sort_by(&test_support::compair_bot_pair);
   
    {      
      use std::collections::HashSet;
      println!("control vs test len={:?}",(control_result.len(),test_result.len()));
      
      let mut control_hash=HashSet::new();
      for k in control_result.iter(){
          control_hash.insert(k);
      }

      let mut test_hash=HashSet::new();
      for k in test_result.iter(){
          test_hash.insert(k);
      }

      let diff=control_hash.symmetric_difference(&test_hash).collect::<Vec<_>>();
      println!("diff={:?}",diff.len());
      assert!(diff.len()==0);
    }


}



/*
#[test]
fn test_every_col_pair(){

    for _ in 0..10{
        let world=test_support::create_word();
        
          let mut vec:Vec<BBox<NotNaN<f32>,Bot>>=(0..100).map(|a|
        {
            let rect=test_support::get_random_rect(&world);
            let bot=Bot::new(a);
            BBox::new(bot,rect)
        }
            ).collect();
        
        let mut bots_orig=vec.clone();
        /*
        println!("Bots:");
        for b in vec.iter(){
            println!("{:?}",(b.val,b.get()));
        }
        println!();
        */


        let mut src:Vec<(usize,usize)>=Vec::new();
        for (e,i) in vec.iter().enumerate(){
            for j in vec[e+1..].iter(){
                let (a,b):(&BBox<NotNaN<f32>,Bot>,&BBox<NotNaN<f32>,Bot>)=(i,j);

                if a.get().intersects_rect(b.get()){
                    src.push(test_support::create_unordered(&a.val,&b.val));
                }
            }
        }


        let mut tc=TreeCache::new::<par::Parallel,_>(axgeom::XAXIS,5,&mut vec);


        let mut v:Vec<(usize,usize)>=Vec::new();

        

        let mut kd=KdTree::new::<par::Parallel,MedianRelax>(&mut tc,&mut vec);
        //let mut kd=KdTree::new::<DefaultDepthLevel,MedianStrict>(&mut vec,&mut tc,&world);
        {
            let mut func=|cc:ColPair<BBox<NotNaN<f32>,Bot>>|{
                v.push(test_support::create_unordered(cc.a.1,cc.b.1));
            };

            //let mut l=timer::LL::new(tc.get_height());
            kd.for_every_col_pair_sequential(&mut func);
        }
    
        src.sort_by(&test_support::compair_bot_pair);
        v.sort_by(&test_support::compair_bot_pair);
            

        {

            fn at_depth(tree:&mut KdTree<BBox<NotNaN<f32>,Bot>>,id:usize)->Option<usize>{
                
                let level=tree.get_tree().get_level_desc();
                
                let dt=tree.get_tree().create_down();
                use compt::CTreeIterator;
                //use kdtree::colfind::NodeTrait;
                let mut j=compt::LevelIter::new(dt,level);
    
                let mut found=None;
                j.dfs_preorder(|a:(LevelDesc,&Node2<BBox<NotNaN<f32>,Bot>>)|{
                    for bot in a.1.range.iter(){
                        if bot.val.id==id{
                            found=Some(a.0.get_depth());
                        }    
                    }
                });
                return found;
            }
            
            //println!("source:{:?}",src);
        
            //println!("v     :{:?}",v);
            use std::collections::HashSet;
            println!("source/v len={:?}",(src.len(),v.len()));
            
            let mut src_hash=HashSet::new();
            for k in src.iter(){
                src_hash.insert(k);
            }

            let mut v_hash=HashSet::new();
            for k in v.iter(){
                v_hash.insert(k);
            }

            let diff=src_hash.symmetric_difference(&v_hash).collect::<Vec<_>>();
           

            fn find(vec:&[BBox<NotNaN<f32>,Bot>],id:usize)->Option<&BBox<NotNaN<f32>,Bot>>{
                for i in vec.iter(){
                    if i.val.id==id{
                        return Some(i)
                    }
                }
                return None
            }
            for i in diff.iter(){
                let id1=at_depth(&mut kd,i.0).unwrap();
                let id2=at_depth(&mut kd,i.1).unwrap();
                let b1=find(&bots_orig,id1).unwrap();
                let b2=find(&bots_orig,id2).unwrap();
                println!("{:?}\n{:?}\n\n",(id1,b1),(id2,b2));
            }

            assert!(src==v);

        }
    }
*/


///This contains the destructured SweepTrait for a colliding pair.
///The rect is read only while T::Inner is allowed to be mutated.
pub struct ColPair<'a,T:SweepTrait+'a>{
    pub a:(&'a Rect<T::Num>,&'a mut T::Inner),
    pub b:(&'a Rect<T::Num>,&'a mut T::Inner)
}

///Similar to ColPair, but for only one SweepTrait
pub struct ColSingle<'a,T:SweepTrait+'a>(pub &'a Rect<T::Num>,pub &'a mut T::Inner);

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
