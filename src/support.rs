use inner_prelude::*;
use ordered_float::NotNaN;


///A convenience wrapper that implements the NumTrait around any number that implements the 
///required traits for a NumTrait.
#[derive(Copy,Clone,Default,Debug,Eq,PartialEq,PartialOrd,Ord)]
pub struct NumWrapper<T:Ord+Copy+Send+Sync+std::fmt::Debug+Default>(pub T);
impl<T:Ord+Copy+Send+Sync+std::fmt::Debug+Default> NumTrait for NumWrapper<T>{}

///A premade f32 wrapper that implements NumTrait
#[derive(Copy,Clone,Default,Debug,Eq,PartialEq,PartialOrd,Ord)]
pub struct Numf32(pub NotNaN<f32>);
impl NumTrait for Numf32{}

///A premade f64 wrapper that implements NumTrait
#[derive(Copy,Clone,Default,Debug,Eq,PartialEq,PartialOrd,Ord)]
pub struct Numf64(pub NotNaN<f64>);
impl NumTrait for Numf64{}

///A premade isize wrapper that implements NumTrait
#[derive(Copy,Clone,Default,Debug,Eq,PartialEq,PartialOrd,Ord)]
pub struct Numisize(pub isize);
impl NumTrait for Numisize{}

///A premade usize wrapper that implements NumTrait
#[derive(Copy,Clone,Default,Debug,Eq,PartialEq,PartialOrd,Ord)]
pub struct Numusize(pub usize);
impl NumTrait for Numusize{}



///A generic container that implements the kdtree trait.
#[derive(Copy,Clone,Debug)]
pub struct BBox<Nu:NumTrait,T:Send+Sync>{
    pub rect:Rect<Nu>,
    pub val:T
}

impl<Nu:NumTrait,T:Send+Sync> SweepTrait for BBox<Nu,T>{
    type Inner=T;
    type Num=Nu;

    ///Destructure into the bounding box and mutable parts.
    fn get_mut<'a>(&'a mut self)->(&'a Rect<Nu>,&'a mut Self::Inner){
        (&self.rect,&mut self.val)
    }

    ///Destructue into the bounding box and inner part.
    fn get<'a>(&'a self)->(&'a Rect<Nu>,&'a Self::Inner){
        (&self.rect,&self.val)
    }
}

impl<Nu:NumTrait,T:Send+Sync> BBox<Nu,T>{

    #[inline(always)]
    pub fn new(val:T,r:Rect<Nu>)->BBox<Nu,T>{
        BBox{rect:r,val:val}
    }
}


///A default depth level from which to switch to sequential.
pub struct DefaultDepthLevel;

impl DepthLevel for DefaultDepthLevel{
    fn switch_to_sequential(a:LevelDesc)->bool{
        a.get_depth()>=5
    }
}




//use oned::Blee;
///Find all bots that collide along the specified axis only between two rectangles.
///So the bots may not actually collide in 2d space, but collide alone the x or y axis.
///This is useful when implementing "wrap around" behavior of bots that pass over a rectangular border.
pub fn collide_two_rect_parallel<'a:'b,'b,
    A:AxisTrait,
    K:RectsTreeTrait<T=T,Num=Num>,
    Num:NumTrait,
    T:SweepTrait<Num=Num>+'b,
    F:FnMut(ColPair<T>)>(
        rects:&mut Rects<'b,K>,rect1:&Rect<T::Num>,rect2:&Rect<T::Num>,mut func:F)
{
    
    struct Ba<'z,J:SweepTrait+Send+'z>(ColSingle<'z,J>);
    impl<'z,J:SweepTrait+Send+'z> SweepTrait for Ba<'z,J>{
        type Inner=J::Inner;
        type Num=J::Num;

        ///Destructure into the bounding box and mutable parts.
        fn get_mut<'a>(&'a mut self)->(&'a Rect<J::Num>,&'a mut Self::Inner){
            let r=&mut self.0;
            (r.0,r.1)
        }

        ///Destructue into the bounding box and inner part.
        fn get<'a>(&'a self)->(&'a Rect<J::Num>,&'a Self::Inner){
            let r=&self.0;
            (r.0,r.1)
        }
        
    }
    
    let mut buffer1={
        let mut buffer1:Vec<Ba<'b,T>>=Vec::new();
        //let mut wrap=Wrap{a:buffer1};
        //rects.for_all_in_rect(rect1,&mut wrap);
        //wrap.a
        rects.for_all_in_rect(rect1,|a:ColSingle<T>|buffer1.push(Ba(a)));
        buffer1
    };

    let mut buffer2={
        let mut buffer2:Vec<Ba<'b,T>>=Vec::new();
        //let mut wrap=Wrap{a:buffer2};
        //rects.for_all_in_rect(rect2,&mut wrap);
        //wrap.a
        rects.for_all_in_rect(rect2,|a:ColSingle<T>|buffer2.push(Ba(a)));
        buffer2
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
            func(c);
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