//!
//! Some misc tests. Most tests can be found in the dinotree_alg_data project.
//!

extern crate dinotree;
extern crate dinotree_alg;
extern crate axgeom;
extern crate dists;
use dinotree::*;

//TODO write better code


#[test]
fn query_test(){
use axgeom;
    use dinotree_sample::SampleBuilder;
    use dinotree::DinoTreeNoCopyBuilder;
    use dinotree_alg::colfind::{query_naive_mut,QueryBuilder};

    let mut bots=SampleBuilder::new().with_num(10_000).build();
    let mut tree=DinoTreeNoCopyBuilder::new(axgeom::XAXISS,&mut bots).build_seq();
    QueryBuilder::new(tree.as_ref_mut()).query_seq(|a,b|a.collide(b));


    let mut bots2=SampleBuilder::new().with_num(10_000).build();
    query_naive_mut(&mut bots2,|a,b|a.collide(b));

    for (a,b) in bots.iter().zip(bots2.iter()){
        assert_eq!(a.acc,b.acc);
    }


}


#[derive(Copy,Clone,Debug)]
struct Bot{
    id:usize,
    pos:[isize;2],
    col:usize
}

#[test]
fn test_send_sync_dinotree(){
    let mut bots1:Vec<Bot>=Vec::new();
    let mut bots2:Vec<Bot>=Vec::new();

    let (t1,t2)=rayon::join(||{DinoTreeBuilder::new(axgeom::XAXISS,&mut bots1,|_|axgeom::Rect::new(0,0,0,0))}.build_seq(),||{DinoTreeBuilder::new(axgeom::YAXISS,&mut bots2,|_|axgeom::Rect::new(0,0,0,0)).build_seq()});

    let (p1,p2)=(&t1,&t2);

    rayon::join(||{p1},||{p2});
}

#[test]
fn test_zero_sized_type() {
    #[derive(Copy,Clone)]
    struct Bot;

  
    {
        let s=dists::spiral::Spiral::new([400.0,400.0],17.0,0.7);
    
        let mut bots:Vec<_>=s.take(500).map(|_|{
            Bot
        }).collect();

        let mut tree = DinoTreeBuilder::new(axgeom::XAXISS,&mut bots,|_|axgeom::Rect::new(0,0,0,0)).build_seq();

        let mut num=0;
        dinotree_alg::colfind::QueryBuilder::new(tree.as_ref_mut()).query_seq(|_, _| {
               num+=1;
        });

        //black_box(num);
    }
}

#[test]
fn test_one_bot() {

    #[derive(Copy,Clone)]
    struct Bot;
    

    let mut bots:Vec<Bot> = Vec::new();
    bots.push(Bot);
    
    let mut tree = DinoTreeBuilder::new(axgeom::XAXISS,&mut bots,|_|axgeom::Rect::new(0,0,0,0)).build_seq();

    let mut num=0;
    dinotree_alg::colfind::QueryBuilder::new(tree.as_ref_mut()).query_seq(|_, _| {
           num+=1;
    });

    //black_box(num);
}


#[test]
fn recursive_dinotree(){

    #[derive(Copy,Clone)]
    struct Bot;
    

    let mut bots:Vec<Bot> = Vec::new();
    bots.push(Bot);
    bots.push(Bot);
    bots.push(Bot);

    let mut bots2=bots.clone();

    let mut tree = DinoTreeBuilder::new(axgeom::XAXISS,&mut bots,|_|axgeom::Rect::new(0,0,0,0)).build_seq();

    let mut vec:Vec<DinoTree<axgeom::XAXISS,BBox<isize,Bot>>>=Vec::new();
    dinotree_alg::colfind::QueryBuilder::new(tree.as_ref_mut()).query_seq(|_, _| {
           vec.push(DinoTreeBuilder::new(axgeom::XAXISS,&mut bots2,|_|axgeom::Rect::new(0,0,0,0)).build_seq());
    });

    //black_box(vec);
}


#[test]
fn test_empty() {
    #[derive(Copy,Clone)]
    struct Bot;
    

    let mut bots:Vec<Bot> = Vec::new();
    
    let mut tree = DinoTreeBuilder::new(axgeom::XAXISS,&mut bots,|_|axgeom::Rect::new(0,0,0,0)).build_seq();

    let mut num=0;
    dinotree_alg::colfind::QueryBuilder::new(tree.as_ref_mut()).query_seq(|_, _| {
           num+=1;
    });

    //black_box(num);

}

