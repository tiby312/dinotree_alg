 #![feature(test)]

extern crate dinotree;
extern crate rand;
extern crate axgeom;
extern crate test;
extern crate num;
extern crate ordered_float;


mod support;

use dinotree::support::BBox;
use test::*;
use dinotree::*;
use support::*;


#[test]
fn test_k_nearest_not_enough_bots(){
    //TODO test requesting 5 nearest when there only 4 bots.
}


#[test]
fn test_k_nearest(){
    struct Bot{id:usize};

    fn from_point(a:isize,b:isize)->AABBox<isize>{
        AABBox::new((a,a),(b,b))
    }

    let mut bots=Vec::new();
    bots.push(BBox::new(Bot{id:4},from_point(15,15)));
    bots.push(BBox::new(Bot{id:1},from_point(10,10)));
    bots.push(BBox::new(Bot{id:2},from_point(20,20)));
    bots.push(BBox::new(Bot{id:3},from_point(30,30)));
    bots.push(BBox::new(Bot{id:0},from_point(0,0)));

    let mut res=Vec::new();

    let min_rect=|point:[isize;2],aabb:&AABBox<isize>|{
        let (px,py)=(point[0],point[1]);
        //let (px,py)=(px.0,py.0);

        let ((a,b),(c,d))=aabb.get();

        let xx=num::clamp(px,a,b);
        let yy=num::clamp(py,c,d);

        (xx-px)*(xx-px) + (yy-py)*(yy-py)
    };

    let min_oned=|p1:isize,p2:isize|{
        //let (p1,p2)=(p1.0,p2.0);
        (p2-p1)*(p2-p1)
    };

    {
        let mut dyntree = DinoTree::new(&mut bots,  StartAxis::Yaxis);

        dyntree.k_nearest([40,40],3,|a,_dis|res.push(a.inner.id),&min_rect,&min_oned);
        assert!(res.len()==3);
        assert!(res[0]==3);
        assert!(res[1]==2);
        assert!(res[2]==4);

        res.clear();
        dyntree.k_nearest([-40,-40],3,|a,_dis|res.push(a.inner.id),min_rect,min_oned);
        assert!(res.len()==3);
        println!("res={:?}",res);
        assert!(res[0]==0);
        assert!(res[1]==1);
        assert!(res[2]==4);
    }


}



