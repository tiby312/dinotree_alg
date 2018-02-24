

use super::*;
use oned::sup::BleekSF;
use oned::Sweeper;
use SweepTrait;
use ColSingle;
use axgeom::AxisTrait;
use tools::par;

///A construct to allow querying non-intersecting rectangles to retrive mutable references to what is inside them.
///
///#Examples
/// ```ignore //TODO fix
///extern crate axgeom;
///extern crate collie;
///use collie::kdtree::{TreeCache,KdTreeWrapper,KdTreeReal};
///use collie::extensions::Rects;
///use collie::support::BBox; 
///use collie::ColSingle;
///
///fn main(){    
///    #[derive(Copy,Clone)]
///    struct Bot{
///        id:usize
///    }
///    let b1=BBox::new(Bot{id:0},axgeom::Rect::new(0,10,  0,10));
///    let b2=BBox::new(Bot{id:1},axgeom::Rect::new(5,15,  5,15));
///    let b3=BBox::new(Bot{id:2},axgeom::Rect::new(22,40,  22,40));
///    let mut k=vec!(b1,b2,b3);
///
///    let world=axgeom::Rect::new(0,20,0,20);
///    let mut tc=TreeCache::new(axgeom::XAXIS,5,&mut k);
///
///    let mut kd=KdTreeWrapper::new(&mut tc,&mut k);
///    let mut k=kd.get();
///    let mut rects=Rects::new(&mut k);//.create_rects();
///
///    //Need to create a seperate function so that 
///    //we can get a named lifetime from the closure.
///    fn query<'b>(rects:&mut Rects<'b,KdTreeReal<BBox<isize,Bot>>>){
///
///        let mut bots1=Vec::new();
///        rects.for_all_in_rect(
///                    &axgeom::Rect::new(0,20,0,20),
///                    &mut |cc:ColSingle<'b,BBox<isize,Bot>>|{bots1.push(cc.1)});
///
///        let mut bots2=Vec::new();
///        rects.for_all_in_rect(
///                    &axgeom::Rect::new(21,50,21,50),
///                    &mut |cc:ColSingle<'b,BBox<isize,Bot>>|{bots2.push(cc.1)});
///
///        assert!(bots1[0].id==0);
///        assert!(bots1[1].id==1);
///        assert!(bots2[0].id==2);
///    }
///    query(&mut rects);
///}
/// ```
pub struct Rects<'a,C:DynTreeTrait+'a>{
    tree:&'a mut C,
    rects:Vec<axgeom::Rect<C::Num>>
}


impl<'a,C:DynTreeTrait+'a> Rects<'a,C>{

    pub fn new(tree:&'a mut C)->Rects<'a,C>{
        Rects{tree:tree,rects:Vec::new()}
    }

    ///Iterate over all bots in a rectangle.
    ///It is safe to call this function multiple times with rectangles that 
    ///do not intersect. Because the rectangles do not intersect, all bots retrieved
    ///from inside either rectangle are guarenteed to be disjoint. 
    ///If a rectangle is passed that does intersect one from a previous call, this function will panic.
    ///
    ///Note the lifetime of the mutable reference in the passed function.
    ///The user is allowed to move this reference out and hold on to it for 
    ///the lifetime of this struct.
    pub fn for_all_in_rect<F>(&mut self,rect:&axgeom::Rect<C::Num>,func:&mut F)
        where F:FnMut(ColSingle<'a,C::T>){

        
        for k in self.rects.iter(){
            if rect.intersects_rect(k){
                panic!("Rects cannot intersect! {:?}",(k,rect));
            }
        }

        {
            let mut fu=|c:ColSingle<C::T>|{
                //axgeom::Rect<C::Num>
                let a=unsafe{&*(c.0 as *const <C::T as SweepTrait>::InnerRect)};
                let b=unsafe{&mut *(c.1 as *mut <C::T as SweepTrait>::Inner)};               
                let cn=ColSingle(a,b);
                //let a=unsafe{(&*(r as *const geom::Rect),&mut *(a as *mut <C::T as BBoxTrait>::Inner))};
                func(cn);
            };

            self.tree.for_all_in_rect(rect,&mut fu);
        }
        
        self.rects.push(*rect);
    }
}


//use oned::Blee;
///Find all bots that collide along the specified axis only between two rectangles.
///So the bots may not actually collide in 2d space, but collide alone the x or y axis.
///This is useful when implementing "wrap around" behavior of bots that pass over a rectangular border.
pub fn collide_two_rect_parallel<'a:'b,'b,A:AxisTrait,K:DynTreeTrait,F:FnMut(ColPair<K::T>)>(rects:&mut Rects<'b,K>,rect1:&Rect<K::Num>,rect2:&Rect<K::Num>,func:&mut F)
{
    struct Ba<'z,J:SweepTrait+Send+'z>(ColSingle<'z,J>);
    impl<'z,J:SweepTrait+Send+'z> SweepTrait for Ba<'z,J>{
        type InnerRect=J::InnerRect;
        type Inner=J::Inner;
        type Num=J::Num;

        ///Destructure into the bounding box and mutable parts.
        fn get_mut<'a>(&'a mut self)->(&'a Self::InnerRect,&'a mut Self::Inner){
            let r=&mut self.0;
            (r.0,r.1)
        }

        ///Destructue into the bounding box and inner part.
        fn get<'a>(&'a self)->(&'a Self::InnerRect,&'a Self::Inner){
            let r=&self.0;
            (r.0,r.1)
        }
        
    }

    let mut buffer1:Vec<Ba<'b,K::T>>=Vec::new();
    {
        let mut f=|cc:ColSingle<'b,K::T>|{
            buffer1.push(Ba(cc));
        };
        rects.for_all_in_rect(rect1,&mut f);
    }

    let mut buffer2:Vec<Ba<'b,K::T>>=Vec::new();
    {
        let mut f=|cc:ColSingle<'b,K::T>|{
            buffer2.push(Ba(cc));
        };
        rects.for_all_in_rect(rect2,&mut f);
    }



    let cols:(&mut [Ba<K::T>],&mut [Ba<K::T>])=(&mut buffer1,&mut buffer2);

    {
        //let blee=Blee::new(axis);

        Sweeper::update::<A,par::Parallel>(cols.0);
        Sweeper::update::<A,par::Parallel >(cols.1);

        let mut func2=|cc:ColPair<Ba<K::T>>|{
            let c=ColPair{a:(cc.a.0,cc.a.1),b:(cc.b.0,cc.b.1)};
            func(c);
        };
        let mut sweeper=Sweeper::new();

        //let r1=rect1.get_range(axis);
        //let r2=rect2.get_range(axis);
        //println!("{:?}",(r1,r2));
        //let r3=&r1.get_intersection(r2).unwrap(); //TODO dont special case this
        let mut b=BleekSF::new(&mut func2);
        sweeper.find_bijective_parallel::<A,_>(cols,&mut b);
    }
}