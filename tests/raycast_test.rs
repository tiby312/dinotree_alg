 #![feature(test)]
mod test_support;

extern crate dinotree;
extern crate test;
extern crate num;
extern crate axgeom;
extern crate rand;

use dinotree::support::BBox;
use test::*;
use dinotree::*;
use test_support::*;

/*
#[test]
fn test_raycast(){
    fn from_point(a:isize,b:isize)->AABBox<isize>{
        AABBox::new((a-10,a+10),(b-10,b+10))
    }

    let mut bots=Vec::new();
    bots.push(BBox::new(Bot::new(0),from_point(-30,0)));
    bots.push(BBox::new(Bot::new(1),from_point(30,0)));
    bots.push(BBox::new(Bot::new(2),from_point(0,-100)));

    let ray=Ray{point:Vec2{x:0,y:0},dir:Vec2{x:0,y:-1},tmax:None};

    //https://tavianator.com/fast-branchless-raybounding-box-intersections/

    let ray_touch_box=|a:ColSingle<BBox<isize,Bot>>|->Option<isize>{
        let ((x1,x2),(y1,y2))=a.rect.get();
        let point=ray.point;
        let dir=ray.dir;

        //top and bottom
        //s(t)=point+t*dir
        let mut tmin=isize::min_value();
        let mut tmax=isize::max_value();

        if dir.x!=0{
            let tx1=(x1-point.x)/dir.x;
            let tx2=(x2-point.x)/dir.x;

            tmin=tmin.max(tx1.min(tx2));
            tmax=tmax.min(tx1.max(tx2));
            
        }
        if dir.y!=0{
            let ty1=(y1-point.y)/dir.y;
            let ty2=(y2-point.y)/dir.y;

            tmin=tmin.max(ty1.min(ty2));
            tmax=tmax.min(ty1.max(ty2));
        }
        if tmax>=tmin && tmin>=0{
            return Some(tmin);
        }
        
        return None
    };


    {
        let mut dyntree = DinoTree::new(&mut bots,  StartAxis::Yaxis);
        let k=dyntree.raycast(ray,ray_touch_box).expect("nothing hit the ray!");
        println!("{:?}",k.0.inner);
        //assert!(false);
    }


}



#[test]
fn test_raycast2(){
    fn from_point(a:isize,b:isize)->AABBox<isize>{
        AABBox::new((a-10,a+10),(b-10,b+10))
    }

    let mut p = PointGenerator::new(
        &test_support::make_rect((0, 1000), (0, 1000)),
        &[100, 42, 6],
    );

    let mut bots = Vec::new();
    for id in 0..10000 {
        let ppp = p.random_point();
        let k = test_support::create_rect_from_point(ppp);
        bots.push(BBox::new(
            Bot {
                id,
                col: Vec::new(),
            },
            k,
        ));
    }

    /*
    let mut bots=Vec::new();
    bots.push(BBox::new(Bot::new(0),from_point(-30,0)));
    bots.push(BBox::new(Bot::new(1),from_point(30,0)));
    bots.push(BBox::new(Bot::new(2),from_point(0,-100)));
    */

    let ray=Ray{point:Vec2{x:0,y:0},dir:Vec2{x:1,y:1},tmax:None};

    //https://tavianator.com/fast-branchless-raybounding-box-intersections/

    let mut num_considered=std::sync::Mutex::new(0);
    


    {
        let ray_touch_box=|a:ColSingle<BBox<isize,Bot>>|->Option<isize>{
            *num_considered.lock().unwrap()+=1;
            let ((x1,x2),(y1,y2))=a.rect.get();
            let point=ray.point;
            let dir=ray.dir;
 
            //top and bottom
            //s(t)=point+t*dir
            let mut tmin=isize::min_value();
            let mut tmax=isize::max_value();

            if dir.x!=0{
                let tx1=(x1-point.x)/dir.x;
                let tx2=(x2-point.x)/dir.x;

                tmin=tmin.max(tx1.min(tx2));
                tmax=tmax.min(tx1.max(tx2));
                
            }
            if dir.y!=0{
                let ty1=(y1-point.y)/dir.y;
                let ty2=(y2-point.y)/dir.y;

                tmin=tmin.max(ty1.min(ty2));
                tmax=tmax.min(ty1.max(ty2));
            }
            //println!("max min ={:?}",(tmin,tmax));
            if tmax>=tmin && tmin>=0{
                //println!("TOUCH!");
                return Some(tmin);
            }
            
            return None
        };

        let mut dyntree = DinoTree::new(&mut bots,  StartAxis::Yaxis);
        let k=dyntree.raycast(ray,ray_touch_box).expect("nothing hit the ray!");
        println!("{:?}",k.0.inner);
        
    }

    let mut num_considered=num_considered.into_inner();
    println!("num considered:{:?}",num_considered);
    assert!(false);
}
*/