use super::*;
use test_support::*;
use axgeom::XAXIS_S;
use axgeom::YAXIS_S;

use support::Numisize;
use support::BBox;
//use median::strict::MedianStrict;
//use treetimer::*;





#[test]
fn test_dinotree_drop(){

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


      {
        let (mut dyntree,_bag)=DinoTree::new::<treetimer::TreeTimerEmpty>(&mut bots);
        

        let clos=|cc:ColPair<BBox<Numisize,Bot>>|{

        };

        let _v=dyntree.for_every_col_pair_seq::<_,treetimer::TreeTimer2>(clos);
      }     

    }  

    println!("{:?}",drop_counter);
    assert!(drop_counter.iter().fold(true,|acc,&x|acc&(x==0)));
}




#[test]
fn test_corners_touch(){
    
    let world=make_rect((-1010,1010),(-110,110));
    let spawn_world=make_rect((-1000,1000),(-100,100));

    //# # # #
    // # # #
    //# # # #
    let mut bots=Vec::new();
    let mut id_counter=0..;
    let mut a=false;
    for y in (-100..200).step_by(20){
        if a{
          for x in (-1000..2000).step_by(20).step_by(2){
            let id=id_counter.next().unwrap();
            let rect=create_rect_from_point((Numisize(x),Numisize(y)));
            bots.push(BBox::new(Bot{id,col:Vec::new()},rect));
          }   
        }else{
          
          for x in (-1000..2000).step_by(20).skip(1).step_by(2){
            let id=id_counter.next().unwrap();
            let rect=create_rect_from_point((Numisize(x),Numisize(y)));
            bots.push(BBox::new(Bot{id,col:Vec::new()},rect));
          }
        }
        a=!a;
    }

    
    test_bot_layout(bots); 
    //assert!(false);
}


#[test]
fn test_1_apart(){
    
    let world=make_rect((-1010,1010),(-110,110));
    let spawn_world=make_rect((-1000,1000),(-100,100));

    let mut bots=Vec::new();
    let mut id_counter=0..;
    for x in (-1000..2000).step_by(21){
      for y in (-100..200).step_by(21){
          let id=id_counter.next().unwrap();
          let rect=create_rect_from_point((Numisize(x),Numisize(y)));
          bots.push(BBox::new(Bot{id,col:Vec::new()},rect));
      }   
    }

    test_bot_layout(bots); 
}


#[test]
fn test_mesh(){
    //in this test, tesselate a bunch of bots such that
    //all of their edges are touching.
    let world=make_rect((-1010,1010),(-110,110));
    let spawn_world=make_rect((-1000,1000),(-100,100));

    let mut bots=Vec::new();
    let mut id_counter=0..;
    for x in (-1000..2000).step_by(20){
      for y in (-100..200).step_by(20){
          let id=id_counter.next().unwrap();
          let rect=create_rect_from_point((Numisize(x),Numisize(y)));
          bots.push(BBox::new(Bot{id,col:Vec::new()},rect));
      }   
    }

    test_bot_layout(bots); 
}

#[test]
fn test_russian_doll(){
    //In this test, test larger and larger rectangles overlapping each other.
    
    let world=make_rect((-1010,1010),(-110,110));

    let spawn_world=make_rect((-1000,1000),(-100,100));

    let mut bots=Vec::new();
    let mut id_counter=0..;

    for x in (-1000..2000).step_by(20){
      for y in (-100..200).step_by(20){
          if x>y{
            let id=id_counter.next().unwrap();
            
            let rect=AABBox(make_rect((-1000,-100),(x,y)));
            
            bots.push(BBox::new(Bot{id,col:Vec::new()},rect));
          }
      }   
    }

    test_bot_layout(bots);
}


fn test_bot_layout(mut bots:Vec<BBox<Numisize,Bot>>){
  let height=12;
  //TODO also test yaxis?
  let mut treecache:TreeCache2<Numisize>=TreeCache2::new(axgeom::XAXIS,height);

  let mut control_result={
      let mut src:Vec<(usize,usize)>=Vec::new();
      
      let control_bots=bots.clone();
      for (i, el1) in control_bots.iter().enumerate() {
          for el2 in control_bots[i + 1..].iter() {
            
              let a=el1;
              let b=el2;
              let ax=(a.get().0).0.get_range2::<XAXIS_S>();     
              let ay=(a.get().0).0.get_range2::<YAXIS_S>();     
              let bx=(b.get().0).0.get_range2::<XAXIS_S>();     
              let by=(b.get().0).0.get_range2::<YAXIS_S>();     
            
              if ax.intersects(bx) && ay.intersects(by){
                  src.push(test_support::create_unordered(&a.val,&b.val));
              }
          }
      }
      src
  };
  

  let mut test_result={
      let mut src:Vec<(usize,usize)>=Vec::new();
      
      //use axgeom::XAXIS_S;
      //use axgeom::YAXIS_S;

      {
        let k=MedianStrict::<Numisize>::new();
        let (mut dyntree,_bag)=treecache.new_tree::<_,par::Parallel,DefaultDepthLevel,_,treetimer::TreeTimerEmpty>
                        (&mut bots,&k);
        
        let clos=|cc:ColPair<BBox<Numisize,Bot>>|{
            let a=cc.a;
            let b=cc.b;
            src.push(test_support::create_unordered(&a.1,&b.1));
        };

        let _v=dyntree.for_every_col_pair_seq::<_,treetimer::TreeTimer2>(clos);
      }       

      src
  };

  control_result.sort_by(&test_support::compair_bot_pair);
  test_result.sort_by(&test_support::compair_bot_pair);
 
  println!("control length={} test length={}",control_result.len(),test_result.len());
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
    
    if diff.len()!=0{
        let bots_copy=bots.clone();
        let k=MedianStrict::<Numisize>::new();
        let (mut dyntree,_bag)=treecache.new_tree::<_,par::Parallel,DefaultDepthLevel,_,treetimer::TreeTimerEmpty>
                          (&mut bots,&k);
         

        //use compt::CTreeIterator;
        /*
        for i in diff.iter(){
            let level=dyntree.0.get_level_desc();
            let first={
              let dd=dyntree.0.get_iter_mut();
              let ll=compt::LevelIter::new(dd,level);
              let mut first=None;
              'bla:for (level,n) in ll.dfs_preorder_iter(){
                 for bot in n.range.iter(){
                    if bot.get().1.id==i.0{
                      first=Some(level.get_depth());
                      break 'bla;
                    }
                 }
              }
              first
            };

            let second={
              let dd=dyntree.0.get_iter_mut();
              let ll=compt::LevelIter::new(dd,level);
              
              let mut second=None;
              'bla2:for (level,n) in ll.dfs_preorder_iter(){
                 for bot in n.range.iter(){
                    if bot.get().1.id==i.1{
                      second=Some(level.get_depth());
                      break 'bla2;
                    }
                 }
              }
              second
            };

            println!("debug={:?}",(first,second));
 
            let first_bot=bots_copy.iter().find(|a|a.get().1.id==i.0).unwrap();
            let second_bot=bots_copy.iter().find(|a|a.get().1.id==i.1).unwrap();
            println!("{:?}",(first_bot.get().0,second_bot.get().0));
        }
        */
      

      
    }

    assert!(diff.len()==0);
  }
}

#[test]
fn test_dinotree(){
    
    let world=make_rect((-1000,1000),(-100,100));

    let spawn_world=make_rect((-990,990),(-90,90));

    let mut p=PointGenerator::new(&spawn_world,&[1,2,3,4,5]);


    for _ in 0..1{

      let mut bots:Vec<BBox<Numisize,Bot>>={
          (0..2000).map(|id|{
              let rect=create_rect_from_point(p.random_point());
              BBox::new(Bot{id,col:Vec::new()},rect)
          }).collect()  
      };
      
      test_bot_layout(bots);
    }    
}