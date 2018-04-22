#![feature(test)]

mod support;
extern crate axgeom;
extern crate num;
extern crate rand;
extern crate dinotree;
extern crate ordered_float;
extern crate test;
use test::*;
use support::*;
use dinotree::*;
use dinotree::support::*;
#[bench]
fn rebal_seq(b: &mut Bencher) {

    let mut bots=create_bots_isize(|id|Bot{id,col:Vec::new()},&[0,1000,0,1000],10000,[2,20]);
    
    b.iter(|| {

        let tree = DinoTree::new_seq(&mut bots,  StartAxis::Xaxis);
        black_box(tree);
        
    });
}
#[bench]
fn rebal_par(b: &mut Bencher) {
    let mut bots=create_bots_isize(|id|Bot{id,col:Vec::new()},&[0,1000,0,1000],10000,[2,20]);
   

    b.iter(|| {

        let tree = DinoTree::new(&mut bots,  StartAxis::Yaxis);
        black_box(tree);
        
    });
}

