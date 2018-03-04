use axgeom;
use support::Numisize;
use std;
use rand;
use rand::{ SeedableRng, StdRng};
use rand::distributions::{IndependentSample, Range};    
use prelude::*;

/*
//Note this doesnt check all invariants.
//e.g. doesnt check that every bot is in the tree only once.
fn assert_invariant<T:SweepTrait>(d:&DinoTree2<T>){
    
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
*/



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

pub fn create_rect_from_point(a:(Numisize,Numisize))->AABBox<Numisize>{
  let r:isize=10;
  let x=a.0;
  let y=a.1;
  AABBox(make_rect((x.0-r,x.0+r),(y.0-r,y.0+r)))
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