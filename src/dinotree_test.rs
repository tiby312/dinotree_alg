use super::*;
use test_support::*;
use axgeom::XAXISS;
use axgeom::YAXISS;

use support::BBox;

use test::*;

#[cfg(test)]
mod bap {
    use super::*;
    use cgmath::{ Point2, Vector2};
    use collision::dbvt::TreeValue;
    use collision::*;
    use collision::dbvt::DynamicBoundingVolumeTree;
    use collision::algorithm::broad_phase::DbvtBroadPhase;

    #[derive(Debug, Clone)]
    struct Value {
        pub id: u32,
        pub aabb: Aabb2<f32>,
        fat_aabb: Aabb2<f32>,
    }

    impl Value {
        pub fn new(id: u32, aabb: Aabb2<f32>) -> Self {
            Self {
                id,
                fat_aabb: aabb.add_margin(Vector2::new(3., 3.)),
                aabb,
            }
        }
    }

    impl TreeValue for Value {
        type Bound = Aabb2<f32>;

        fn bound(&self) -> &Aabb2<f32> {
            &self.aabb
        }

        fn get_bound_with_margin(&self) -> Aabb2<f32> {
            self.fat_aabb.clone()
        }
    }

    fn aabb2(minx: f32, miny: f32, maxx: f32, maxy: f32) -> Aabb2<f32> {
        Aabb2::new(Point2::new(minx, miny), Point2::new(maxx, maxy))
    }

    #[bench]
    #[ignore]
    fn colfind_3rd_part(b: &mut Bencher) {
        //let mut rng = rand::thread_rng();
        let mut tree = DynamicBoundingVolumeTree::<Value>::new();

        let mut p = PointGenerator::new(
            &test_support::make_rect((0, 1000), (0, 1000)),
            &[100, 42, 6],
        );

        for id in 0..10000 {
            let ppp = p.random_point();
            let offset_x = ppp.0 as f32;
            let offset_y = ppp.1 as f32;

            tree.insert(Value::new(
                id,
                aabb2(offset_x + 2., offset_y + 2., offset_x + 4., offset_y + 4.),
            ));
            tree.tick();
        }

        let db = DbvtBroadPhase::new();

        let aa: Vec<bool> = (0..10000).map(|_| true).collect();
        b.iter(|| {
            let cols = db.find_collider_pairs(&tree, &aa);
            black_box(cols);
        });
    }
}

#[bench]
fn colfind(b: &mut Bencher) {
    use test_support::*;
    let mut p = PointGenerator::new(
        &test_support::make_rect((0, 1000), (0, 1000)),
        &[100, 42, 6],
    );

    let mut bots = Vec::new();
    for id in 0..10000 {
        let ppp = p.random_point();
        let k = test_support::create_rect_from_point(ppp);
        bots.push(BBox::new(
            Bot {
                id,
                col: Vec::new(),
            },
            k,
        ));
    }

    //let height = compute_tree_height(bots.len());

    let mut tree = DinoTree::new(&mut bots, true);

    let mut fu = |a: ColSingle<BBox<isize, Bot>>, b: ColSingle<BBox<isize, Bot>>| {
        a.inner.col.push(b.inner.id);
        b.inner.col.push(a.inner.id);
    };

    b.iter(|| {
        black_box(tree.intersect_every_pair_seq(&mut fu));
    });
}

#[bench]
fn colfind_par(b: &mut Bencher) {
    use test_support::*;
    let mut p = PointGenerator::new(
        &test_support::make_rect((0, 1000), (0, 1000)),
        &[100, 42, 6],
    );

    let mut bots = Vec::new();
    for id in 0..10000 {
        let ppp = p.random_point();
        let k = test_support::create_rect_from_point(ppp);
        bots.push(BBox::new(
            Bot {
                id,
                col: Vec::new(),
            },
            k,
        ));
    }

    //let height = compute_tree_height(bots.len());

    let mut tree = DinoTree::new(&mut bots, true);

    b.iter(|| {

        black_box(tree.intersect_every_pair(|a, b| {
            a.inner.col.push(b.inner.id);
            b.inner.col.push(a.inner.id);
        }));
    });
}


#[bench]
fn colfind_par_point(b: &mut Bencher) {
    use test_support::*;
    let mut p = PointGenerator::new(
        &test_support::make_rect((0, 200), (0, 200)),
        &[100, 42, 6],
    );

    let mut bots = Vec::new();
    for id in 0..2000 {
        let ppp = p.random_point();
        //let k = test_support::create_rect_from_point(ppp);
        bots.push(BBox::new(
            Bot {
                id,
                col: Vec::new(),
            },
            AABBox::<isize>::new((ppp.0,ppp.0),(ppp.1,ppp.1)),
        ));
    }

    
    let mut tree = DinoTree::new(&mut bots, true);

    b.iter(|| {

        let k=tree.intersect_every_pair_debug(|a, b| {
            a.inner.col.push(b.inner.id);
            b.inner.col.push(a.inner.id);
        });
        //println!("{:?}",k.into_vec());
        black_box(k);
    });

    //assert!(false);
}



#[bench]
fn colfind_par_dense(b: &mut Bencher) {
    use test_support::*;
    let mut p = PointGenerator::new(
        &test_support::make_rect((0, 200), (0, 200)),
        &[100, 42, 6],
    );

    let mut bots = Vec::new();
    for id in 0..2000 {
        let ppp = p.random_point();
        let k = test_support::create_rect_from_point(ppp);
        bots.push(BBox::new(
            Bot {
                id,
                col: Vec::new(),
            },
            k,
        ));
    }

    
    let mut tree = DinoTree::new(&mut bots, true);

    b.iter(|| {

        let k=tree.intersect_every_pair_debug(|a, b| {
            a.inner.col.push(b.inner.id);
            b.inner.col.push(a.inner.id);
        });
        //println!("{:?}",k.into_vec());
        black_box(k);
    });

}


#[bench]
fn rebal_seq(b: &mut Bencher) {
    use test_support::*;
    let mut p = PointGenerator::new(
        &test_support::make_rect((0, 1000), (0, 1000)),
        &[100, 42, 6],
    );

    let mut bots = Vec::new();
    for id in 0..10000 {
        let ppp = p.random_point();
        let k = test_support::create_rect_from_point(ppp);
        bots.push(BBox::new(
            Bot {
                id,
                col: Vec::new(),
            },
            k,
        ));
    }

    
    b.iter(|| {

        let tree = DinoTree::new_seq(&mut bots, true);
        black_box(tree);
        
    });
}
#[bench]
fn rebal_par(b: &mut Bencher) {
    use test_support::*;
    let mut p = PointGenerator::new(
        &test_support::make_rect((0, 1000), (0, 1000)),
        &[100, 42, 6],
    );

    let mut bots = Vec::new();
    for id in 0..10000 {
        let ppp = p.random_point();
        let k = test_support::create_rect_from_point(ppp);
        bots.push(BBox::new(
            Bot {
                id,
                col: Vec::new(),
            },
            k,
        ));
    }

    b.iter(|| {

        let tree = DinoTree::new(&mut bots, true);
        black_box(tree);
        
    });
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
        let mut bots: Vec<BBox<isize, Bot>> = Vec::new();

        
        let spawn_world = make_rect((-990, 990), (-90, 90));

        let mut p = PointGenerator::new(&spawn_world, &[1, 2, 3, 4, 5]);

        for (id, dc) in (0..5000).zip(drop_counter.iter_mut()) {
            let rect = create_rect_from_point(p.random_point());
            let j = BBox::new(
                Bot {
                    _id:id,
                    drop_counter: dc,
                },
                rect,
            );
            bots.push(j);
        }

        {
            let mut dyntree:DinoTree<BBox<isize,Bot>> = DinoTree::new(&mut bots, false);

            dyntree.intersect_every_pair_seq(|_,_|{});
        }
    }

    println!("{:?}", drop_counter);
    assert!(drop_counter.iter().fold(true, |acc, &x| acc & (x == 0)));
}

#[test]
fn test_dinotree_move_back() {

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
    let bots_control=bots.clone();

    {
        let mut dyntree = DinoTree::new(&mut bots, false);

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
        let mut dyntree = DinoTree::new(&mut bots, false);

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
                let rect = create_rect_from_point((x, y));
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
                let rect = create_rect_from_point((x, y));
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

        //Test the max size of slice +1
        let k=move ||{

            let mut dyntree = DinoTree::new(&mut bots, false);

            let mut counter=0;
            

            dyntree.intersect_every_pair_seq(|_, _| {
                if counter==1000{
                    panic!("panic inside of callback!");
                }
                counter+=1;

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

        {
            let mut dyntree = DinoTree::new(&mut bots, false);

            dyntree.intersect_every_pair_seq(|_,_|{});
        }
    }
    
    assert_eq!(unsafe{DROP_COUNTER},5000);

}

#[test]
fn test_k_nearest(){
    fn from_point(a:isize,b:isize)->AABBox<isize>{
        AABBox::new((a,a),(b,b))
    }

    let mut bots=Vec::new();
    bots.push(BBox::new(Bot::new(4),from_point(15,15)));
    bots.push(BBox::new(Bot::new(1),from_point(10,10)));
    bots.push(BBox::new(Bot::new(2),from_point(20,20)));
    bots.push(BBox::new(Bot::new(3),from_point(30,30)));
    bots.push(BBox::new(Bot::new(0),from_point(0,0)));

    let mut res=Vec::new();

    let min_rect=|point:(isize,isize),aabb:&AABBox<isize>|{
        let (px,py)=(point.0,point.1);
        //let (px,py)=(px.0,py.0);

        let ((a,b),(c,d))=aabb.get();

        let xx=num::clamp(px,a,b);
        let yy=num::clamp(py,c,d);

        (xx-px)*(xx-px) + (yy-px)*(yy-py)
    
    };

    let min_oned=|p1:isize,p2:isize|{
        //let (p1,p2)=(p1.0,p2.0);
        (p2-p1)*(p2-p1)
    };

    {
        let mut dyntree = DinoTree::new(&mut bots, false);

        dyntree.k_nearest((40,40),3,|a|res.push(a.inner.id),&min_rect,&min_oned);
        assert!(res.len()==3);
        assert!(res[0]==3);
        assert!(res[1]==2);
        assert!(res[2]==4);

        res.clear();
        dyntree.k_nearest((-40,-40),3,|a|res.push(a.inner.id),min_rect,min_oned);
        assert!(res.len()==3);
        println!("res={:?}",res);
        assert!(res[0]==0);
        assert!(res[1]==1);
        assert!(res[2]==4);
    }


}



#[test]
fn test_rect(){
    fn from_point(a:isize,b:isize)->AABBox<isize>{
        AABBox::new((a,a),(b,b))
    }

    let mut bots=Vec::new();
    bots.push(BBox::new(Bot::new(0),from_point(0,0)));
    bots.push(BBox::new(Bot::new(1),from_point(10,0)));
    bots.push(BBox::new(Bot::new(2),from_point(0,10)));
    bots.push(BBox::new(Bot::new(3),from_point(10,10)));


    let mut res=Vec::new();
    {
        let mut dyntree = DinoTree::new(&mut bots, false);

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
        let mut dyntree = DinoTree::new(&mut bots, false);

        let mut r=dyntree.rects();
        let rect=AABBox::new((0,10),(0,10));
        r.for_all_in_rect(&rect,|a: ColSingle<BBox<isize, Bot>>|res.push(a.inner.id));
        let rect=AABBox::new((10,20),(0,10));

        r.for_all_in_rect(&rect,|a: ColSingle<BBox<isize, Bot>>|res.push(a.inner.id));
    }

}


#[test]
fn test_rect_intersect(){
    fn from_point(a:isize,b:isize)->AABBox<isize>{
        AABBox::new((a,a),(b,b))
    }

    let mut bots=Vec::new();
    bots.push(BBox::new(Bot::new(0),from_point(0,0)));
    bots.push(BBox::new(Bot::new(1),from_point(10,0)));
    bots.push(BBox::new(Bot::new(2),from_point(0,10)));
    bots.push(BBox::new(Bot::new(3),from_point(10,10)));

    let rect=AABBox::new((10,20),(10,20));
    bots.push(BBox::new(Bot::new(3),rect));

    let mut res=Vec::new();
    {
        let mut dyntree = DinoTree::new(&mut bots, false);


        let rect=AABBox::new((0,10),(0,10));
        dyntree.for_all_intersect_rect(&rect,|a|res.push(a.inner.id));
    }
    assert!(res.len()==5);
}

#[test]
fn test_intersect_with(){
 
    fn from_rect(a:isize,b:isize,c:isize,d:isize)->AABBox<isize>{
        AABBox::new((a,b),(c,d)) 
    }

    let mut bots=Vec::new();
    bots.push(BBox::new(Bot::new(0),from_rect(0,10,0,10)));
    bots.push(BBox::new(Bot::new(1),from_rect(5,10,0,10)));

    let mut bots2=Vec::new();
    bots2.push(BBox::new(Bot::new(2),from_rect(-10,4,0,10)));
    bots2.push(BBox::new(Bot::new(3),from_rect(-10,3,0,10)));


    let mut res=Vec::new();
    {
        let mut dyntree = DinoTree::new(&mut bots, false);

        dyntree.intersect_with_seq::<BBox<isize,Bot>,_>(&mut bots2,|a,b|res.push((a.inner.id,b.inner.id)));
    }

    assert!(res.len()==2);
    res.contains(&(0,2));
    res.contains(&(0,3));
}


#[test]
fn test_bounding_boxes_as_points() {
    

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

    test_bot_layout(bots);

}



#[test]
fn test_one_bot() {
    
    let mut bots:Vec<BBox<isize,Bot>> = Vec::new();
    let rect=create_rect_from_point((0,0));
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
            let rect = create_rect_from_point((x, y));
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
            let rect = create_rect_from_point((x, y));
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

                let rect = AABBox(make_rect((-1000, -100), (x, y)));

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
            let mut dyntree = DinoTree::new(&mut bots, false);

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
fn test_dinotree() {
    

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

    test_bot_layout(bots);

}
