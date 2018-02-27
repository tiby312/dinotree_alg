

use super::*;
//use oned::sup::BleekSF;
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

use std::marker::PhantomData;


pub trait RectsTrait<'c>{
    type T:SweepTrait;
    fn collide(&mut self,a:ColSingle<'c,Self::T>);  
}


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
    pub fn for_all_in_rect<F:RectsTrait<'a,T=C::T>>(&mut self,rect:&axgeom::Rect<C::Num>,func:&mut F)
    {

        
        for k in self.rects.iter(){
            if rect.intersects_rect(k){
                panic!("Rects cannot intersect! {:?}",(k,rect));
            }
        }

        {
            struct Wrapper<'a,'b:'a,F:RectsTrait<'b>+'a>{
                closure:&'a mut F,
                p:PhantomData<&'b usize>
            };

            impl<'a,'b:'a,T:SweepTrait+'b,F:RectsTrait<'b,T=T>+'a> ColSing for Wrapper<'a,'b,F>{
                type T=F::T;
                fn collide(&mut self,c:ColSingle<Self::T>){
                    let (a,b)=(c.0 as *const <F::T as SweepTrait>::InnerRect,c.1 as *mut <F::T as SweepTrait>::Inner);
                    //Unsafely extend the lifetime to accocomate the
                    //lifetime of RectsTrait.
                    let (a,b)=unsafe{(&*a,&mut *b)};
                    
                    let cn=ColSingle(a,b);
                    self.closure.collide(cn);
    
                }
            }

            let mut wrapper=Wrapper{closure:func,p:PhantomData};
            self.tree.for_all_in_rect(rect,&mut wrapper);
        }
        
        self.rects.push(*rect);
    }
}


//use oned::Blee;
///Find all bots that collide along the specified axis only between two rectangles.
///So the bots may not actually collide in 2d space, but collide alone the x or y axis.
///This is useful when implementing "wrap around" behavior of bots that pass over a rectangular border.
pub fn collide_two_rect_parallel<'a:'b,'b,A:AxisTrait,K:DynTreeTrait<T=T,Num=Num>,Num:NumTrait,T:SweepTrait<Num=Num>+'b,F:ColSeq<T=T>>(rects:&mut Rects<'b,K>,rect1:&Rect<T::Num>,rect2:&Rect<T::Num>,func:&mut F)
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


    struct Wrap<'c,T:SweepTrait+'c>{
        a:Vec<Ba<'c,T>>
    }

    impl<'c,T:SweepTrait> RectsTrait<'c> for Wrap<'c,T>{
        type T=T;
        fn collide(&mut self,cc:ColSingle<'c,T>){
            self.a.push(Ba(cc));
        }
    }

    
    let mut buffer1={
        let mut buffer1:Vec<Ba<'b,T>>=Vec::new();
        let mut wrap=Wrap{a:buffer1};
        rects.for_all_in_rect(rect1,&mut wrap);
        wrap.a
    };

    let mut buffer2={
        let mut buffer2:Vec<Ba<'b,T>>=Vec::new();
        let mut wrap=Wrap{a:buffer2};
        rects.for_all_in_rect(rect2,&mut wrap);
        wrap.a
    };


    let cols:(&mut [Ba<T>],&mut [Ba<T>])=(&mut buffer1,&mut buffer2);

    {
        //let blee=Blee::new(axis);

        Sweeper::update::<A,par::Parallel>(cols.0);
        Sweeper::update::<A,par::Parallel >(cols.1);
        /*
        let mut func2=|cc:ColPair<Ba<K::T>>|{
            let c=ColPair{a:(cc.a.0,cc.a.1),b:(cc.b.0,cc.b.1)};
            func.collide(c);
        };
        */


        use std::marker::PhantomData;
        use oned::Bleek;
        struct Bo<T:SweepTrait,F:FnMut(ColPair<T>)>(
            F,
            PhantomData<T>
        );

        impl<T:SweepTrait,F:FnMut(ColPair<T>)> Bleek for Bo<T,F>{
            type T=T;
            fn collide(&mut self,cc:ColPair<Self::T>){
                self.0(cc);
            }
        }
        
        let mut func2=|cc:ColPair<Ba<K::T>>|{
            let c=ColPair{a:(cc.a.0,cc.a.1),b:(cc.b.0,cc.b.1)};
            func.collide(c);
        };
        

        //let r1=rect1.get_range(axis);
        //let r2=rect2.get_range(axis);
        //println!("{:?}",(r1,r2));
        //let r3=&r1.get_intersection(r2).unwrap(); //TODO dont special case this
        //let mut b=BleekSF::new(&mut func2);
        let mut sweeper=Sweeper::new();

        let b=Bo(func2,PhantomData);
        sweeper.find_bijective_parallel::<A,_>(cols,b);
    }
}