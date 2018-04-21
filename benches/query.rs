

#[test]
fn naive(){

}

#[test]
fn all_same_x_value_median(){
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
    */
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

    let mut tree = DinoTree::new(&mut bots, StartAxis::Xaxis);

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


    let mut tree = DinoTree::new(&mut bots,  StartAxis::Yaxis);

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

    
    let mut tree = DinoTree::new(&mut bots,  StartAxis::Xaxis);

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

    
    let mut tree = DinoTree::new(&mut bots,  StartAxis::Yaxis);

    b.iter(|| {

        let k=tree.intersect_every_pair_debug(|a, b| {
            a.inner.col.push(b.inner.id);
            b.inner.col.push(a.inner.id);
        });
        //println!("{:?}",k.into_vec());
        black_box(k);
    });

}