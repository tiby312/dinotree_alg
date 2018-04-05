extern crate dinotree;
use dinotree::*;
use dinotree::support::BBox;

#[derive(Debug)]
struct Bot {
    id: usize,
    touching: Vec<usize>,
}

fn make_bot(id: usize, x: (isize, isize), y: (isize, isize)) -> BBox<isize, Bot> {
    let rect = AABBox::new(
        (x.0, x.1),
        (y.0, y.1),
    );
    BBox::new(
        Bot {
            id,
            touching: Vec::new(),
        },
        rect,
    )
}

fn main() {
    let mut bots: Vec<BBox<isize, Bot>> = Vec::new();

    bots.push(make_bot(0, (10, 20), (50, 60)));
    bots.push(make_bot(1, (15, 25), (50, 60)));
    bots.push(make_bot(2, (16, 30), (50, 60)));

    bots.push(make_bot(3, (50, 60), (10, 20)));
    bots.push(make_bot(4, (50, 60), (15, 25)));
    bots.push(make_bot(5, (50, 60), (16, 30)));

    {
        let mut dyntree = DinoTree::new(&mut bots, false);

        let clos = |a: ColSingle<BBox<isize, Bot>>, b: ColSingle<BBox<isize, Bot>>| {
            a.inner.touching.push(b.inner.id);
            b.inner.touching.push(a.inner.id);
        };

        dyntree.intersect_every_pair_seq(clos);
    }

    println!("These bots are colliding:");
    for b in bots.iter() {
        println!("bots={:?}", (&b.val.id, &b.val.touching));
    }
}
