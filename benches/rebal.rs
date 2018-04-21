


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

        let tree = DinoTree::new_seq(&mut bots,  StartAxis::Xaxis);
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

        let tree = DinoTree::new(&mut bots,  StartAxis::Yaxis);
        black_box(tree);
        
    });
}

