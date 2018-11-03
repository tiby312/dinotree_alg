


/*


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
            let mut dyntree = DinoTree::new_seq(&mut bots,  StartAxis::Xaxis);

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
            //println!("diff={:?}",diff);

            //println!("first={:?}",(&bots[diff[0].0],&bots[diff[0].1]));
            //let bots_copy = bots.clone();


            let mut dyntree = DinoTree::new(&mut bots, StartAxis::Xaxis);

            for i in diff.iter(){
                let id1=i.0;
                let id2=i.1;
                println!("------------------");
                println!("{:?}",dyntree.find_element(|bla|{bla.val.id==id1}));
                println!("{:?}",dyntree.find_element(|bla|{bla.val.id==id2}));

                println!("------------------");
            }
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
            }*/
        }

        assert!(diff.len() == 0);
    }
}

*/



/*

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
*/




pub mod russian_doll{
    /*

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

    */
}

pub mod mesh{
/*
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
*/
}
pub mod one_apart{
    /*
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
    */

}


pub mod lattice{
    /*
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
    */
}




pub mod spiral{
    use super::*;


    #[derive(Copy,Clone)]
    pub struct Spiral{
        point:[f64;2],
        rad:f64,
        start:f64,
        rate:f64,
        width:f64
    }

    impl Spiral{
        pub fn new(point:[f64;2],circular_grow:f64,outward_grow:f64)->Spiral{
            Spiral{point,rad:0.0,start:1.0,rate:outward_grow,width:circular_grow}
        }
        pub fn get_circular_grow(&self)->f64{
            self.width
        }
        pub fn get_outward_grow(&self)->f64{
            self.rate
        }
    }
    
    impl std::iter::FusedIterator for Spiral{}

    impl Iterator for Spiral{
        type Item=[f64;2];
        fn next(&mut self)->Option<[f64;2]>{
            
            let length=self.start+self.rate*self.rad;

            let x=self.point[0]+self.rad.cos()*length;
            let y=self.point[1]+self.rad.sin()*length;

            self.rad+=self.width/length;

            Some([x,y])

        }
    }

}

