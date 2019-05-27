extern crate dinotree;
extern crate dinotree_alg;
use dinotree::*;
use dinotree::copy::*;

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

    {
        let mut tree = DinoTreeBuilder::new(axgeom::YAXISS,&mut bots,|a|a.rect).build_seq();

        let mut rects = dinotree_alg::multirect::multi_rect_mut(&mut tree);
        let r1 = axgeom::Rect::new(10, 25, 40, 70);
        let r2 = axgeom::Rect::new(40, 70, 10, 25);

        let mut bb1: Vec<&mut Bot> = Vec::new();
        let mut bb2: Vec<&mut Bot> = Vec::new();

        {
            let _ = rects.for_all_in_rect_mut(r1, |cc| {
                bb1.push(&mut cc.inner);
            });
        }

        {
            let _ = rects.for_all_in_rect_mut(r2, |cc| {
                bb2.push(&mut cc.inner);
            });
        }

        assert_eq!(bb1[0].id,0);
        assert_eq!(bb1[1].id,1);
        println!("These bots are in:{:?}", r1);
        for b in bb1.iter() {
            println!("\tbots={:?}", (&b.id));
        }


        assert_eq!(bb2[0].id,4);
        assert_eq!(bb2[1].id,3);
        println!("These bots are in:{:?}", r2);
        for b in bb2.iter() {
            println!("\tbots={:?}", (&b.id));
        }
    }
}
