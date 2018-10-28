extern crate dinotree;
extern crate dinotree_alg;
use dinotree::*;
extern crate num;


#[derive(Debug)]
struct Bot {
    id: usize,
}

fn make_bot(id: usize, x: (isize, isize), y: (isize, isize)) -> BBox<isize, Bot> {
    let rect = AABBox::new(
        (x.0, x.1),
        (y.0, y.1),
    );
    BBox::new(
        Bot {
            id,
        },
        rect,
    )
}

fn main() {
    let mut bots: Vec<Bot> = Vec::new();


    //    |--------------|
    //             ||
    //                      ||
    //                                ||

    bots.push(make_bot(0, (10, 40), (0, 0)));
    bots.push(make_bot(1, (30, 30), (0, 0)));
    bots.push(make_bot(2, (50, 50), (0, 0)));
    bots.push(make_bot(3, (70, 70), (0, 0)));


    {
        let mut dyntree = DinoTree::new(&mut bots,  StartAxis::Xaxis);

        //Compute distance sqr
        let min_rect=&|point:[isize;2],aabb:&AABBox<isize>|{
            let (px,py)=(point[0],point[1]);

            let ((a,b),(c,d))=aabb.get();

            let xx=num::clamp(px,a,b);
            let yy=num::clamp(py,c,d);

            (xx-px)*(xx-px) + (yy-py)*(yy-py)
        };

        //Compute distance sqr in 1d cases.
        let min_oned=&|p1:isize,p2:isize|{
            (p2-p1)*(p2-p1)
        };

        {
            let mut v=Vec::new();
            dyntree.k_nearest([100,0],1,|a,dis|{v.push((a.inner.id,dis))},min_rect,min_oned);
            assert_eq!(v[0].0,3);
            assert_eq!(v[0].1,(100-70)*(100-70));
        }
        {
            let mut v=Vec::new();
            dyntree.k_nearest([41,0],2,|a,dis|{v.push((a.inner.id,dis))},min_rect,min_oned);
            assert_eq!(v[0].0,0);
            assert_eq!(v[1].0,2);
        }
    }

    
}
