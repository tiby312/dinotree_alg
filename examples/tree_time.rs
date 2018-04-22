extern crate dinotree;
extern crate axgeom;
extern crate num;
extern crate rand;
extern crate ordered_float;
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
    println!();
}

fn test(num:usize,radius:[isize;2]){

    let mut bots=create_bots_isize(|id|Bot{id,col:Vec::new()},&[0,800,0,800],num,radius);

    {
        let (mut dyntree,rebal_times) = DinoTree::new_debug(&mut bots,  StartAxis::Xaxis);

        let col_times=dyntree.intersect_every_pair_debug(|a, b| {
           a.inner.col.push(b.inner.id);
           b.inner.col.push(a.inner.id);
            
        });

        print_times("dinotree creation times (microseconds):",&rebal_times);
    
        print_times("dinotree query times (microseconds):",&col_times);
    }

}
fn main() {
    let num=50000;
    println!("------------------------------------------------------------------------------------------------------------------");
    println!("Colliding {} bots! (run example with --release)",num);

    let t1=[2,20];
    println!("Test with bots with radius in the range {:?}.",t1);
    test(num,t1);

    let t1=[1,2];
    println!("Test with bots with radius in the range {:?}.",t1);
    test(num,t1);

    println!("Notice that with smaller bots, the lower levels of the tree take up more of the time since more bots live in them");
    println!("------------------------------------------------------------------------------------------------------------------");
}
