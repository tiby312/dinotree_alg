extern crate dinotree;
extern crate dinotree_alg;
use dinotree::*;
use dinotree::copy::*;
use dinotree_alg::k_nearest;

pub fn clamp<T: PartialOrd>(input: T, min: T, max: T) -> T {
    debug_assert!(min <= max, "min must be less than or equal to max");
    if input < min {
        min
    } else if input > max {
        max
    } else {
        input
    }
}

///Returns the squred distance from a point to a rectangle if the point is outisde the rectangle.
///If the point is insert the rectangle, it will return None.
fn distance_squared_point_to_rect(point:[isize;2],rect:&axgeom::Rect<isize>)->Option<isize>{
    let (px,py)=(point[0],point[1]);

    let ((a,b),(c,d))=rect.get();

    let xx=clamp(px,a,b);
    let yy=clamp(py,c,d);

    
    let dis=(xx-px)*(xx-px) + (yy-py)*(yy-py);

    //Then the point must be insert the rect.
    //In this case, lets return something negative.
    if xx>a && xx<b && yy>c && yy< d{
        None
    }else{
        Some(dis)
    }
}

struct Kn;

impl k_nearest::Knearest for Kn{
    type T=BBox<isize,Bot>;
    type N=isize;
    type D=isize;
    fn twod_check(&mut self, point:[Self::N;2],bot:&Self::T)->Self::D{
        
        let dis=distance_squared_point_to_rect(point,bot.get());
        let dis=match dis{
            Some(dis)=>{
                dis
            },
            None=>{
                //Lets just return zero for cases where the point is inside a rectangle.
                //This can lead to some non intuitive results about the closest rectangle
                //if many differently shaped rectangles are intersecting the point we are 
                //querying.
                0
            }
        };
        dis
    }

    fn oned_check(&mut self,p1:Self::N,p2:Self::N)->Self::D{
        let diff=p2-p1;
        diff*diff
    }

    //create a range around n.
    fn create_range(&mut self,b:Self::N,d:Self::D)->[Self::N;2]{
        
        let dis=(d as f64).sqrt() as isize;
        [b-dis,b+dis]
    
    }
}




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


    //    |--------------|
    //             ||
    //                      ||
    //                                ||

    bots.push(make_bot(0, (10, 40), (0, 0)));
    bots.push(make_bot(1, (30, 30), (0, 0)));
    bots.push(make_bot(2, (50, 50), (0, 0)));
    bots.push(make_bot(3, (70, 70), (0, 0)));


    {
        let dinotree = DinoTreeBuilder::new(axgeom::XAXISS,&mut bots,|a|a.rect).build_seq();

        {
            let mut v=Vec::new();
            for a in k_nearest::k_nearest(&dinotree,[100,0],1,Kn){
                v.push(a);
            }
            assert_eq!(v[0].bots[0].inner.id,3);
            assert_eq!(v[0].mag,(100-70)*(100-70));
        }
        {
            let mut v=Vec::new();
            for a in k_nearest::k_nearest(&dinotree,[41,0],2,Kn){
                v.push(a);
            }
            assert_eq!(v[0].bots[0].inner.id,0);
            assert_eq!(v[1].bots[0].inner.id,2);
            assert_eq!(v[0].mag,1);
        }
    }

    
}
