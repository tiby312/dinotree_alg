extern crate dinotree;
extern crate dinotree_alg;
use dinotree::*;


#[derive(Copy,Clone,Debug)]
struct Bot {
    id: usize,
    rect:axgeom::Rect<isize>
}

fn make_bot(id: usize, x: (isize, isize), y: (isize, isize)) -> Bot {
    let rect=axgeom::Rect::new(x.0,x.1,y.0,y.1);
    Bot{id,rect}
}


fn main() {
    let mut bots: Vec<Bot> = Vec::new();

    bots.push(make_bot(0, (10, 20), (50, 60)));
    bots.push(make_bot(1, (15, 25), (50, 60)));
    bots.push(make_bot(2, (16, 30), (50, 60)));

    bots.push(make_bot(3, (50, 60), (10, 20)));
    bots.push(make_bot(4, (50, 60), (15, 25)));
    bots.push(make_bot(5, (50, 60), (16, 30)));


    let mut pairs=Vec::new();
    {
        let mut dinotree = DinoTree::new(axgeom::XAXISS,(),&mut bots,|a|a.rect);

        dinotree_alg::colfind::query_seq_mut(&mut dinotree,|a, b| {
            pairs.push((a.inner.id,b.inner.id))
        });
    }

    println!("These bots are colliding:");
    for b in pairs.iter() {
        println!("pairs={:?}", b);
    }
}
