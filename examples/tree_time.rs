#![feature(test)]

extern crate dinotree;
extern crate dinotree_alg;
extern crate axgeom;
extern crate test;
extern crate dists;
use test::*;

use dinotree_alg::colfind;


fn print_times(st:&str,times:&[f64]){
    println!("{}",st);
    for (i,_) in times.iter().enumerate(){
        print!("\t{}",i);
    }
    println!();
    for t in times.iter(){
        print!("\t{:.4}",t);
    }

    println!("\ttotal={:.4}",times.iter().fold(0.0,|a,b|{a+b}));
    println!();
}

#[derive(Copy,Clone)]
struct Bot{
    pos:[isize;2],
    col:usize
}
impl Bot{
    fn create_rect(&self)->axgeom::Rect<isize>{
        let d=5;
        let x=self.pos[0];
        let y=self.pos[1];
        axgeom::Rect::new(x-d,x+d,y-d,y+d)
    }
}


fn test_sequential(num:usize,grow:f64){
    let s=dists::spiral::Spiral::new([400.0,400.0],17.0,grow);
    
    let mut bots:Vec<_>=s.take(num).map(|pos|{
        let x=pos[0] as isize;
        let y=pos[1] as isize;
        Bot{pos:[x,y],col:0}
    }).collect();

    {
        let treetimes=dinotree::advanced::LevelTimer::new();
        let height=dinotree::advanced::compute_tree_height_heuristic(bots.len());
        let (mut tree,treetimes) = dinotree::advanced::new_adv_seq(axgeom::XAXISS,(),&mut bots,|a|a.create_rect(),height,treetimes);

        let treetimes2=dinotree::advanced::LevelTimer::new();
        
        let treetimes2=colfind::query_seq_adv_mut(&mut tree,|a, b| {
            a.inner.col+=1;
            b.inner.col+=1;
            
        },treetimes2);

        let treetimes=treetimes.into_inner();
        let treetimes2=treetimes2.into_inner();
        print_times("dinotree creation times (seconds):",&treetimes);
    
        print_times("dinotree query times (seconds):",&treetimes2);
        
    };
    black_box(bots);
    
}

fn main() {
    let num=50_000;
    println!("Colliding {} bots! (run example with --release)",num);

    println!("Sequential ------------------------------------------------------------------------------------------------------------------");
   
    let t1=2.0;
    println!("Test with bots with grow {:?}.",t1);
    test_sequential(num,t1);

    let t1=0.1;
    println!("Test with bots with grow {:?}.",t1);
    test_sequential(num,t1);

    println!("------------------------------------------------------------------------------------------------------------------");
    println!("Notice that the more spread out the bots, the lower levels of the tree take up more of the time since more bots live in them.");
    println!("See dinotree_alg_data project for more comparisons of the practical running time of algorithms as a whole");
}
