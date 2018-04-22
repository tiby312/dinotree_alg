#![feature(test)]

extern crate dinotree;
extern crate axgeom;
extern crate num;
extern crate rand;
extern crate ordered_float;
extern crate test;
use test::*;
mod support;
use dinotree::*;
use dinotree::support::BBox;

use support::*;

fn print_times(st:&str,times:&[f64]){
    println!("{}",st);
    for (i,_) in times.iter().enumerate(){
        print!("\t{}",i);
    }
    println!();
    for (i,t) in times.iter().enumerate(){
        print!("\t{:.4}",t);
    }

    println!("\ttotal={:.4}",times.iter().fold(0.0,|a,b|{a+b}));
    println!();
}


fn test_sequential(num:usize,radius:[isize;2]){
    let mut bots=create_bots_isize(|id|Bot{id,col:Vec::new()},&[0,800,0,800],num,radius);
    {
        let (mut dyntree,rebal_times) = DinoTree::new_seq_debug(&mut bots,  StartAxis::Xaxis);

        let col_times=dyntree.intersect_every_pair_seq_debug(|a, b| {
           a.inner.col.push(b.inner.id);
           b.inner.col.push(a.inner.id);
            
        });

        print_times("dinotree creation times (seconds):",&rebal_times);
    
        print_times("dinotree query times (seconds):",&col_times);
        
    };
    black_box(bots);
    
}

fn main() {
    let num=50_000;
    println!("Colliding {} bots! (run example with --release)",num);

    println!("Sequential ------------------------------------------------------------------------------------------------------------------");
   
    let t1=[2,20];
    println!("Test with bots with radius in the range {:?}.",t1);
    test_sequential(num,t1);

    let t1=[1,2];
    println!("Test with bots with radius in the range {:?}.",t1);
    test_sequential(num,t1);

    println!("------------------------------------------------------------------------------------------------------------------");


    println!("Notice that with smaller bots, the lower levels of the tree take up more of the time since more bots live in them.");
    println!("Run the benches more more comparisons of the practical running time of algorithms as a whole");
    



}
