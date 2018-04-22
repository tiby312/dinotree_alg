
#![feature(test)]
#![feature(iterator_step_by)]


extern crate dinotree;
extern crate rand;
extern crate axgeom;
extern crate test;
extern crate num;
extern crate rayon;
extern crate ordered_float;
mod support;

use axgeom::XAXISS;
use axgeom::YAXISS;
use dinotree::support::BBox;
use test::*;
use dinotree::*;
use support::*;
use support as test_support;




#[test]
fn test_send_sync_dinotree(){

    let mut bots1:Vec<BBox<isize,Bot>>=Vec::new();
    let mut bots2:Vec<BBox<isize,Bot>>=Vec::new();


    let (t1,t2)=rayon::join(||{DinoTree::new(&mut bots1, StartAxis::Xaxis)},||{DinoTree::new(&mut bots2, StartAxis::Yaxis)});

    let (p1,p2)=(&t1,&t2);

    rayon::join(||{p1},||{p2});

}

#[test]
fn test_dinotree_drop() {
    struct Bot<'a> {
        _id: usize,
        drop_counter: &'a mut isize,
    }

    impl<'a> Drop for Bot<'a> {
        fn drop(&mut self) {
            *self.drop_counter -= 1;
        }
    }

    let mut drop_counter: Vec<isize> = (0..5000).map(|_| 1).collect();
    {
        let mut ii=drop_counter.iter_mut();
        let mut bots=create_bots_isize(|id|{
            let v=ii.next().unwrap();
            Bot{_id:id,drop_counter:v}
        },&[0,1000,0,1000],5000,[2,20]);
  

        {
            let mut dyntree:DinoTree<BBox<isize,Bot>> = DinoTree::new(&mut bots,  StartAxis::Xaxis);

            dyntree.intersect_every_pair_seq(|_,_|{});
        }
    }

    println!("{:?}", drop_counter);
    assert!(drop_counter.iter().fold(true, |acc, &x| acc & (x == 0)));
}

#[test]
fn test_dinotree_move_back() {

    
    let mut bots=create_bots_isize(|id|Bot{id,col:Vec::new()},&[-990,990,-90,90],5000,[2,20]);
    /*
    let mut bots: Vec<BBox<isize, Bot>> = Vec::new();

    

    let spawn_world = make_rect((-990, 990), (-90, 90));

    let mut p = PointGenerator::new(&spawn_world, &[1, 2, 3, 4, 5]);

    for id in 0..5000{
        let rect = create_rect_from_point(p.random_point());
        let j = BBox::new(
            Bot {
                id,
                col:Vec::new()
            },
            rect,
        );
        bots.push(j);
    }
    */
    let bots_control=bots.clone();

    {
        let mut dyntree = DinoTree::new(&mut bots,  StartAxis::Yaxis);

        //let clos = |a: ColSingle<BBox<isize, Bot>>, b: ColSingle<BBox<isize, Bot>>| {};

        dyntree.intersect_every_pair_seq(|_,_|{});
    }

    for (a,b) in bots.iter().zip(bots_control.iter()){
        assert!(a.val.id==b.val.id);
        assert!(a.rect.get()==b.rect.get());
    }
}


#[test]
fn test_dinotree_adv() {

    let mut bots=create_bots_isize(|id|Bot{id,col:Vec::new()},&[-990,990,-90,90],5000,[2,20]);
    /*
    let mut bots: Vec<BBox<isize, Bot>> = Vec::new();

    

    let spawn_world = make_rect((-990, 990), (-90, 90));

    let mut p = PointGenerator::new(&spawn_world, &[1, 2, 3, 4, 5]);

    for id in 0..5000 {
        let rect = create_rect_from_point(p.random_point());
        let j = BBox::new(
            Bot {
                id,
                col:Vec::new()
            },
            rect,
        );
        bots.push(j);
    }
    */
    let bots_control=bots.clone();


    struct Blag{
        a:Vec<Vec<(usize,usize)>>
    }
    impl Blag{
        fn new()->Blag{
            let mut a=Vec::new();
            a.push(Vec::new());
            Blag{a}
        }
        fn append(&mut self,mut bb:Blag){
            self.a.append(&mut bb.a);
        }
    }

    let mut pairs=Blag::new();
    {
        let mut dyntree = DinoTree::new(&mut bots,  StartAxis::Yaxis);

        let clos = |aa:&mut Blag,a: ColSingle<BBox<isize, Bot>>, b: ColSingle<BBox<isize, Bot>>| {
            //expensive collide code here
            aa.a.first_mut().unwrap().push((a.inner.id,b.inner.id))
        };

        let div=|aa:Blag|->(Blag,Blag){
            (aa,Blag::new())
        } ;

        let add=|mut aa:Blag,bb:Blag|->Blag{
            aa.append(bb);
            aa
        };


        pairs=dyntree.intersect_every_pair_adv(pairs,clos,div,add);
    }

    for (a,b) in bots.iter().zip(bots_control.iter()){
        assert!(a.val.id==b.val.id);
        assert!(a.rect.get()==b.rect.get());
    }


    for i in &pairs.a{
        println!("pairs={:?}",i.len());
        
    }

    assert_eq!(pairs.a.len(),32);
    
}

#[test]
fn test_corners_touch() {
    
    //# # # #
    // # # #
    //# # # #
    let mut bots = Vec::new();
    let mut id_counter = 0..;
    let mut a = false;
    for y in (-100..200).step_by(20) {
        if a {
            for x in (-1000..2000).step_by(20).step_by(2) {
                let id = id_counter.next().unwrap();
                let rect =AABBox::new((x-10,x+10),(y-10,y+10));
                bots.push(BBox::new(
                    Bot {
                        id,
                        col: Vec::new(),
                    },
                    rect,
                ));
            }
        } else {
            for x in (-1000..2000).step_by(20).skip(1).step_by(2) {
                let id = id_counter.next().unwrap();
                //let rect = create_rect_from_point((x, y));
                let rect =AABBox::new((x-10,x+10),(y-10,y+10));
                
                bots.push(BBox::new(
                    Bot {
                        id,
                        col: Vec::new(),
                    },
                    rect,
                ));
            }
        }
        a = !a;
    }

    test_bot_layout(bots);
    //assert!(false);
}




#[test]
fn test_panic_in_callback() {


    std::panic::set_hook(Box::new(|_| {
        //println!("Custom panic hook");
    }));

    struct Bot{
        was_hit:usize
    }

    static mut DROP_COUNTER: isize = 0;

    impl Drop for Bot{
        fn drop(&mut self) {
            //We know that the bots are dropped sequentially.
            unsafe{
                DROP_COUNTER+=1;
            }
        }
    }

    {
        let mut bots=create_bots_isize(|id|Bot{was_hit:0},&[-990,990,-90,90],5000,[2,20]);
        /*
        let mut bots: Vec<BBox<isize, Bot>> = Vec::new();

        

        let spawn_world = make_rect((-990, 990), (-90, 90));

        let mut p = PointGenerator::new(&spawn_world, &[1, 2, 3, 4, 5]);

        for _id in 0..5000 {
            let rect = create_rect_from_point(p.random_point());
            let j = BBox::new(
                Bot{was_hit:0},
                rect,
            );
            bots.push(j);
        }
        */

        {
            struct Point(*mut [BBox<isize,Bot>]);
            unsafe impl Send for Point{};

            let bots=Point((&mut bots as &mut [BBox<isize,Bot>]) as *mut [BBox<isize,Bot>]);
            //Test the max size of slice +1
            let k=move ||{
                let bb=unsafe{&mut *bots.0};
                let mut dyntree = DinoTree::new(bb,  StartAxis::Yaxis);

                let mut counter=0;
                

                dyntree.intersect_every_pair_seq(|a, b| {
                    
                    if counter==1000{
                        panic!("panic inside of callback!");
                    }
                    counter+=1;
                    
                    a.inner.was_hit+=1;
                    b.inner.was_hit+=1;
                });
            
                
            };

            let t=std::thread::spawn(k);

            match t.join(){
                Ok(x)=>{
                    panic!("test fail {:?}",x);
                },
                Err(_e)=>{
                    //expected
                }
            }
        }

        let total=bots.iter().fold(0,|a,b|{a+b.val.was_hit});
        assert_eq!(total,2000);
    }
    assert_eq!(unsafe{DROP_COUNTER},5000);
    let _ = std::panic::take_hook();

}

#[test]
fn test_zero_sized_type() {
    struct Bot;

    static mut DROP_COUNTER: isize = 0;

    impl Drop for Bot{
        fn drop(&mut self) {
            //We know that the bots are dropped sequentially.
            unsafe{
                DROP_COUNTER+=1;
            }
        }
    }

    {
        let mut bots=create_bots_isize(|id|Bot,&[-990,990,-90,90],5000,[2,20]);
        /*
        let mut bots: Vec<BBox<isize, Bot>> = Vec::new();

        

        let spawn_world = make_rect((-990, 990), (-90, 90));

        let mut p = PointGenerator::new(&spawn_world, &[1, 2, 3, 4, 5]);

        for _id in 0..5000 {
            let rect = create_rect_from_point(p.random_point());
            let j = BBox::new(
                Bot,
                rect,
            );
            bots.push(j);
        }
    */
        {
            let mut dyntree = DinoTree::new(&mut bots,  StartAxis::Xaxis);

            dyntree.intersect_every_pair_seq(|_,_|{});
        }
    }
    
    assert_eq!(unsafe{DROP_COUNTER},5000);

}


#[test]
fn test_rect(){
    struct Bot{id:usize};

    fn from_point(a:isize,b:isize)->AABBox<isize>{
        AABBox::new((a,a),(b,b))
    }

    let mut bots=Vec::new();
    bots.push(BBox::new(Bot{id:0},from_point(0,0)));
    bots.push(BBox::new(Bot{id:1},from_point(10,0)));
    bots.push(BBox::new(Bot{id:2},from_point(0,10)));
    bots.push(BBox::new(Bot{id:3},from_point(10,10)));


    let mut res=Vec::new();
    {
        let mut dyntree = DinoTree::new(&mut bots,  StartAxis::Xaxis);

        let clos = |a: ColSingle<BBox<isize, Bot>>| {

            res.push(a.inner.id);
        };

        let mut r=dyntree.rects();
        let rect=AABBox::new((0,10),(0,10));
        r.for_all_in_rect(&rect,clos);
    }
    assert!(res.len()==4);
}

#[should_panic]
#[test]
fn test_rect_panic(){

    let mut bots=Vec::new();

    let mut res=Vec::new();
    {
        let mut dyntree = DinoTree::new(&mut bots,  StartAxis::Yaxis);

        let mut r=dyntree.rects();
        let rect=AABBox::new((0,10),(0,10));
        r.for_all_in_rect(&rect,|a: ColSingle<BBox<isize, Bot>>|res.push(a.inner.id)).unwrap();
        let rect=AABBox::new((10,20),(0,10));

        r.for_all_in_rect(&rect,|a: ColSingle<BBox<isize, Bot>>|res.push(a.inner.id)).unwrap();
    }

}


#[test]
fn test_rect_intersect(){
    struct Bot{id:usize};

    fn from_point(a:isize,b:isize)->AABBox<isize>{
        AABBox::new((a,a),(b,b))
    }

    let mut bots=Vec::new();
    bots.push(BBox::new(Bot{id:0},from_point(0,0)));
    bots.push(BBox::new(Bot{id:1},from_point(10,0)));
    bots.push(BBox::new(Bot{id:2},from_point(0,10)));
    bots.push(BBox::new(Bot{id:3},from_point(10,10)));

    let rect=AABBox::new((10,20),(10,20));
    bots.push(BBox::new(Bot{id:3},rect));

    let mut res=Vec::new();
    {
        let mut dyntree = DinoTree::new(&mut bots,  StartAxis::Xaxis);


        let rect=AABBox::new((0,10),(0,10));
        dyntree.for_all_intersect_rect(&rect,|a|res.push(a.inner.id));
    }
    assert!(res.len()==5);
}

#[test]
fn test_intersect_with(){
    struct Bot{id:usize};
    fn from_rect(a:isize,b:isize,c:isize,d:isize)->AABBox<isize>{
        AABBox::new((a,b),(c,d)) 
    }

    let mut bots=Vec::new();
    bots.push(BBox::new(Bot{id:0},from_rect(0,10,0,10)));
    bots.push(BBox::new(Bot{id:1},from_rect(5,10,0,10)));

    let mut bots2=Vec::new();
    bots2.push(BBox::new(Bot{id:2},from_rect(-10,4,0,10)));
    bots2.push(BBox::new(Bot{id:3},from_rect(-10,3,0,10)));


    let mut res=Vec::new();
    {
        let mut dyntree = DinoTree::new(&mut bots,  StartAxis::Yaxis);

        dyntree.intersect_with_seq::<BBox<isize,Bot>,_>(&mut bots2,|a,b|res.push((a.inner.id,b.inner.id)));
    }

    assert!(res.len()==2);
    res.contains(&(0,2));
    res.contains(&(0,3));
}


#[test]
fn test_bounding_boxes_as_points() {
    

    let mut bots=create_bots_isize(|id|Bot{id,col:Vec::new()},&[0,800,0,800],500,[2,3]);
    /*
    let spawn_world = make_rect((-990, 990), (-90, 90));

    let mut p = PointGenerator::new(&spawn_world, &[1, 2, 3, 4, 5]);

    let bots: Vec<BBox<isize, Bot>> = {
        (0..2000)
            .map(|id| {
                let p=p.random_point();
                let rect = AABBox::new((p.0,p.0),(p.1,p.1));
                BBox::new(
                    Bot {
                        id,
                        col: Vec::new(),
                    },
                    rect,
                )
            })
            .collect()
    };
    */

    test_bot_layout(bots);

}



#[test]
fn test_one_bot() {
    
    let mut bots:Vec<BBox<isize,Bot>> = Vec::new();
    //let rect=create_rect_from_point((0,0));
    let rect =AABBox::new((0-10,0+10),(0-10,0+10));
                
    bots.push(BBox::new(Bot{id:0,col:Vec::new()},rect));
    test_bot_layout(bots);
}



#[test]
fn test_empty() {
    
    let bots:Vec<BBox<isize,Bot>> = Vec::new();
    
    test_bot_layout(bots);
}


#[test]
fn test_1_apart() {
    
    let mut bots = Vec::new();
    let mut id_counter = 0..;
    for x in (-1000..2000).step_by(21) {
        for y in (-100..200).step_by(21) {
            let id = id_counter.next().unwrap();
            //let rect = create_rect_from_point((x, y));
            let rect =AABBox::new((x-10,x+10),(y-10,y+10));
                
            bots.push(BBox::new(
                Bot {
                    id,
                    col: Vec::new(),
                },
                rect,
            ));
        }
    }

    test_bot_layout(bots);
}

#[test]
fn test_mesh() {
    //in this test, tesselate a bunch of bots such that
    //all of their edges are touching.
    
    let mut bots = Vec::new();
    let mut id_counter = 0..;
    for x in (-1000..2000).step_by(20) {
        for y in (-100..200).step_by(20) {
            let id = id_counter.next().unwrap();
            //let rect = create_rect_from_point((x, y));
            let rect =AABBox::new((x-10,x+10),(y-10,y+10));
                
            bots.push(BBox::new(
                Bot {
                    id,
                    col: Vec::new(),
                },
                rect,
            ));
        }
    }

    test_bot_layout(bots);
}

#[test]
fn test_russian_doll() {
    //In this test, test larger and larger rectangles overlapping each other.

    
    let mut bots = Vec::new();
    let mut id_counter = 0..;

    for x in (-1000..2000).step_by(20) {
        for y in (-100..200).step_by(20) {
            if x > y {
                let id = id_counter.next().unwrap();

                //let rect = AABBox(make_rect((-1000, -100), (x, y)));
                let rect =AABBox::new((-1000,-100),(y,x));
                
                bots.push(BBox::new(
                    Bot {
                        id,
                        col: Vec::new(),
                    },
                    rect,
                ));
            }
        }
    }

    test_bot_layout(bots);
}


fn test_bot_layout(mut bots: Vec<BBox<isize, Bot>>) {
    let mut control_result = {
        let mut src: Vec<(usize, usize)> = Vec::new();

        let control_bots = bots.clone();
        for (i, el1) in control_bots.iter().enumerate() {
            for el2 in control_bots[i + 1..].iter() {
                let a = el1;
                let b = el2;
                let ax = (a.get().0).0.get_range2::<XAXISS>();
                let ay = (a.get().0).0.get_range2::<YAXISS>();
                let bx = (b.get().0).0.get_range2::<XAXISS>();
                let by = (b.get().0).0.get_range2::<YAXISS>();

                if ax.intersects(bx) && ay.intersects(by) {
                    src.push(test_support::create_unordered(&a.val, &b.val));
                }
            }
        }
        src
    };

    let mut test_result = {
        let mut src: Vec<(usize, usize)> = Vec::new();

        {
            let mut dyntree = DinoTree::new(&mut bots,  StartAxis::Xaxis);

            let clos = |a: ColSingle<BBox<isize, Bot>>, b: ColSingle<BBox<isize, Bot>>| {
                //let (a,b)=(ca,ca.1);
                //let a=ca[0];
                //let b=ca[1];
                src.push(test_support::create_unordered(&a.inner, &b.inner));
            };

            dyntree.intersect_every_pair_seq(clos);
        }

        src
    };

    control_result.sort_by(&test_support::compair_bot_pair);
    test_result.sort_by(&test_support::compair_bot_pair);

    println!(
        "control length={} test length={}",
        control_result.len(),
        test_result.len()
    );
    {
        use std::collections::HashSet;
        println!(
            "control vs test len={:?}",
            (control_result.len(), test_result.len())
        );

        let mut control_hash = HashSet::new();
        for k in control_result.iter() {
            control_hash.insert(k);
        }

        let mut test_hash = HashSet::new();
        for k in test_result.iter() {
            test_hash.insert(k);
        }

        let diff = control_hash
            .symmetric_difference(&test_hash)
            .collect::<Vec<_>>();

        if diff.len() != 0 {
            //let bots_copy = bots.clone();

            //let mut dyntree = DinoTree::new(&mut bots, false);

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
        */        }

        assert!(diff.len() == 0);
    }
}

#[test]
fn test_fat_bots_dinotree() {
    

    let mut bots=create_bots_isize(|id|Bot{id,col:Vec::new()},&[0,800,0,800],500,[2,50]);
    /*
    let spawn_world = make_rect((-990, 990), (-90, 90));

    let mut p = PointGenerator::new(&spawn_world, &[1, 2, 3, 4, 5]);

    let bots: Vec<BBox<isize, Bot>> = {
        (0..2000)
            .map(|id| {
                let rect = create_rect_from_point(p.random_point());
                BBox::new(
                    Bot {
                        id,
                        col: Vec::new(),
                    },
                    rect,
                )
            })
            .collect()
    };
    */

    test_bot_layout(bots);

}


#[test]
fn test_massive_bots_dinotree() {
    

    let mut bots=create_bots_isize(|id|Bot{id,col:Vec::new()},&[0,800,0,800],500,[100,200]);
    /*
    let spawn_world = make_rect((-990, 990), (-90, 90));

    let mut p = PointGenerator::new(&spawn_world, &[1, 2, 3, 4, 5]);

    let bots: Vec<BBox<isize, Bot>> = {
        (0..2000)
            .map(|id| {
                let rect = create_rect_from_point(p.random_point());
                BBox::new(
                    Bot {
                        id,
                        col: Vec::new(),
                    },
                    rect,
                )
            })
            .collect()
    };
    */

    test_bot_layout(bots);

}
