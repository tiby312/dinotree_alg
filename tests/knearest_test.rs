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




#[test]
fn k_nearest_par_point() {
    
    let mut bots=create_bots_isize(|id|Bot{id,col:Vec::new()},&[0,200,0,200],2000,[0,1]);
 

    let points:Vec<[isize;2]>=bots.iter().map(|b|{
        let ((x1,_),(y1,_))=b.rect.get();
        [x1,y1]
    }).collect();

    let mut tree = DinoTree::new(&mut bots,  StartAxis::Yaxis);


    

    {

        for (i,p) in points.iter().enumerate(){
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

            tree.k_nearest(*p,1,|a,_|{
                if a.inner.id!=i{
                    let ((a,b),(c,d))=a.rect.get();
                    assert_eq!(a,p[0]);
                    assert_eq!(b,p[0]);
                    assert_eq!(c,p[1]);
                    assert_eq!(d,p[1]);
                }
                
            },min_rect,min_oned);
        }
        /*
        let k=tree.intersect_every_pair_debug(|a, b| {
            a.inner.col.push(b.inner.id);
            b.inner.col.push(a.inner.id);
        });
        */
        //println!("{:?}",k.into_vec());
        //black_box(k);
    }

    //assert!(false);
}


#[test]
fn k_nearest_par_point2() {
    let mut bots=create_bots_isize(|id|Bot{id,col:Vec::new()},&[0,200,0,200],2000,[0,20]);
    /*
    let mut p = PointGenerator::new(
        &support::make_rect((0, 200), (0, 200)),
        &[100, 42, 6],
    );

    let mut bots = Vec::new();
    let mut points=Vec::new();
    for id in 0..2000 {
        let ppp = p.random_point();
        points.push(ppp);
        //let k = test_support::create_rect_from_point(ppp);
        bots.push(BBox::new(
            Bot {
                id,
                col: Vec::new(),
            },
            AABBox::<isize>::new((ppp.0,ppp.0),(ppp.1,ppp.1)),
        ));
    }
    */

    let points:Vec<[isize;2]>=bots.iter().map(|b|{
        let ((x1,_),(y1,_))=b.rect.get();
        [x1,y1]
    }).collect();



    //println!("bot 716={:?}",&bots[716]);
    //println!("point 19={:?} bot19={:?}",&points[19],&bots[19]);    
    let mut tree = DinoTree::new(&mut bots,  StartAxis::Xaxis);


    

    {

        let mut total_dis=0;
        let mut num_found=0;
        for (i,p) in points.iter().enumerate(){
            let min_rect=|point:[isize;2],aabb:&AABBox<isize>|{
                let (px,py)=(point[0],point[1]);

                let ((a,b),(c,d))=aabb.get();

                let xx=num::clamp(px,a,b);
                let yy=num::clamp(py,c,d);

                (xx-px)*(xx-px) + (yy-py)*(yy-py)
            };

            let min_oned=|p1:isize,p2:isize|{
                (p2-p1)*(p2-p1)
            };


            let mut counter=0;
            tree.k_nearest(*p,2,|a,dis|{

                if counter==1{
                    total_dis+=dis;
                    num_found+=1;
                }
                counter+=1;
                
            },min_rect,min_oned);
        }

        println!("total dis={}",total_dis);
        let avg=total_dis/(points.len() as isize);

        //Check that the average distance the the nearest object to every other object
        //is small
        assert!(avg < 10, " avg={:?} ",avg);

        assert_eq!(num_found,points.len());

        black_box(avg);
    }

    //assert!(false);
}
