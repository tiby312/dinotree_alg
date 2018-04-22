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
fn k_nearest_par_point(b: &mut Bencher) {
    
    let mut bots=create_bots_isize(|id|Bot{id,col:Vec::new()},&[0,200,0,200],2000,[0,1]);
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

    let mut tree = DinoTree::new(&mut bots,  StartAxis::Yaxis);


    

    b.iter(|| {

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
    });

    //assert!(false);
}


#[bench]
fn k_nearest_par_point2(b: &mut Bencher) {
    let mut bots=create_bots_isize(|id|Bot{id,col:Vec::new()},&[0,800,0,800],500,[0,1]);
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


    

    b.iter(|| {

        let mut total_dis=0;
        let mut num_found=0;
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


            let mut counter=0;
            tree.k_nearest(*p,2,|a,dis|{

                if counter==1{
                    total_dis+=dis;
                    num_found+=1;
                }
                counter+=1;
                
            },min_rect,min_oned);
        }

        let avg=total_dis/(points.len() as isize);
        //println!("avg dis={:?}",));
        //Check that the average distance the the nearest object to every other object
        //is small

        assert!(avg < 10, " avg={:?} ",avg);

        assert_eq!(num_found,points.len());
        /*
        let k=tree.intersect_every_pair_debug(|a, b| {
            a.inner.col.push(b.inner.id);
            b.inner.col.push(a.inner.id);
        });
        */
        //println!("{:?}",k.into_vec());
        black_box(avg);
    });

    //assert!(false);
}

