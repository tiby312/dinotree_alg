use std;
use SweepTrait;
use axgeom;
use NumTrait;
use ordered_float::NotNaN;
use DepthLevel;
use compt::LevelDesc;
use InnerRect;

//A convenience wrapper that implements the NumTrait around any number that implements the 
//required traits for a NumTrait.
#[derive(Copy,Clone,Default,Debug,Eq,PartialEq,PartialOrd,Ord)]
pub struct NumWrapper<T:Ord+Copy+Send+Sync+std::fmt::Debug+Default>(pub T);
impl<T:Ord+Copy+Send+Sync+std::fmt::Debug+Default> NumTrait for NumWrapper<T>{}

#[derive(Copy,Clone,Default,Debug,Eq,PartialEq,PartialOrd,Ord)]
pub struct Numf32(pub NotNaN<f32>);
impl NumTrait for Numf32{}

#[derive(Copy,Clone,Default,Debug,Eq,PartialEq,PartialOrd,Ord)]
pub struct Numf64(pub NotNaN<f64>);
impl NumTrait for Numf64{}

#[derive(Copy,Clone,Default,Debug,Eq,PartialEq,PartialOrd,Ord)]
pub struct Numisize(pub isize);
impl NumTrait for Numisize{}

/*
struct InnerR<X,Nu:NumTrait>{
    rect:axgeom::Rect<Nu>,
    stuff:X
}
*/


///A generic container that implements the kdtree trait.
#[derive(Copy,Clone,Debug)]
pub struct BBox<Nu:NumTrait,X:InnerRect<Num=Nu>+Send+Sync,T:Send+Sync>{
    //rect:axgeom::Rect<Nu>,
    pub stuff:X,
    pub val:T
}

impl<Nu:NumTrait,X:InnerRect<Num=Nu>+Send+Sync,T:Send+Sync> SweepTrait for BBox<Nu,X,T>{
    type InnerRect=X;
    type Inner=T;
    type Num=Nu;

    ///Destructure into the bounding box and mutable parts.
    fn get_mut<'a>(&'a mut self)->(&'a Self::InnerRect,&'a mut Self::Inner){
        (&self.stuff,&mut self.val)
    }

    ///Destructue into the bounding box and inner part.
    fn get<'a>(&'a self)->(&'a Self::InnerRect,&'a Self::Inner){
        (&self.stuff,&self.val)
    }
    
    /*
    fn get_mut<'a>(&'a mut self)->(&'a axgeom::Rect<Nu>,&'a mut Self::Inner){
        (&self.rect,&mut self.val)
    }
    fn get<'a>(&'a self)->(&'a axgeom::Rect<Nu>,&'a Self::Inner){
        (&self.rect,&self.val)
    }
    */
}

impl<Nu:NumTrait,X:InnerRect<Num=Nu>+Send+Sync,T:Send+Sync> BBox<Nu,X,T>{

    #[inline(always)]
    pub fn new(val:T,r:X)->BBox<Nu,X,T>{
        BBox{stuff:r,val:val}
    }
    /*
    #[inline(always)]
    pub fn update_box(&mut self,rect:axgeom::Rect<Nu>){
        self.rect=rect;
    }
    */
}


///A default depth level from which to switch to sequential.
pub struct DefaultDepthLevel;

impl DepthLevel for DefaultDepthLevel{
    fn switch_to_sequential(a:LevelDesc)->bool{
        a.get_depth()>=5
    }
}


pub mod closure_struct{
    use super::*;
    use ColPair;
    use std::marker::PhantomData;
    use ColSeq;
    use ColSingle;
    use ColSing;
    use ColMulti;

    pub struct ColSeqStruct<T:SweepTrait,F:FnMut(ColPair<T>)>{
        d:F,
        p:PhantomData<T>
    }
    impl<T:SweepTrait,F:FnMut(ColPair<T>)> ColSeqStruct<T,F>{
        pub fn new(a:F)->ColSeqStruct<T,F>{
            ColSeqStruct{d:a,p:PhantomData}
        }
    }
    impl<T:SweepTrait,F:FnMut(ColPair<T>)> ColSeq for ColSeqStruct<T,F>{
        type T=T;
        fn collide(&mut self,a:ColPair<Self::T>){
            (self.d)(a);
        }
    }
    
    pub struct ColSingStruct<T:SweepTrait,F:FnMut(ColSingle<T>)>{
        d:F,
        p:PhantomData<T>
    }
    impl<T:SweepTrait,F:FnMut(ColSingle<T>)> ColSingStruct<T,F>{
        pub fn new(a:F)->ColSingStruct<T,F>{
            ColSingStruct{d:a,p:PhantomData}
        }
    }
    impl<T:SweepTrait,F:FnMut(ColSingle<T>)> ColSing for ColSingStruct<T,F>{
        type T=T;
        fn collide(&mut self,a:ColSingle<Self::T>){
            (self.d)(a);
        }
    }
    
    
    pub struct ColMultiStruct<'a,
        T:SweepTrait<Inner=I>,
        I:Send+Sync,
        F:Fn(ColPair<T>)+Send+Sync+'a,
        F2:Fn()->I+Send+Sync+'a,
        F3:Fn(&mut I,&I)+Send+Sync+'a
        >{
        a:&'a F,
        b:&'a F2,
        c:&'a F3,
        p:PhantomData<T>
    }

    impl
    <
        'a,
        T:SweepTrait<Inner=I>,
        I:Send+Sync,
        F:Fn(ColPair<T>)+Send+Sync,
        F2:Fn()->I+Send+Sync,
        F3:Fn(&mut I,&I)+Send+Sync
        > ColMultiStruct<'a,T,I,F,F2,F3>{
        pub fn new(a:&'a F,b:&'a F2,c:&'a F3)->ColMultiStruct<'a,T,I,F,F2,F3>{
            ColMultiStruct{a,b,c,p:PhantomData}
        }
    }

    impl
    <
        'a,
        T:SweepTrait<Inner=I>,
        I:Send+Sync,
        F:Fn(ColPair<T>)+Send+Sync,
        F2:Fn()->I+Send+Sync,
        F3:Fn(&mut I,&I)+Send+Sync
        >Clone for ColMultiStruct<'a,T,I,F,F2,F3>{
        fn clone(&self)->Self{
            ColMultiStruct{a:self.a.clone(),b:self.b.clone(),c:self.c.clone(),p:PhantomData}
        }
    }


    impl
    <
        'a,
        T:SweepTrait<Inner=I>,
        I:Send+Sync,
        F:Fn(ColPair<T>)+Send+Sync,
        F2:Fn()->I+Send+Sync,
        F3:Fn(&mut I,&I)+Send+Sync
        >ColMulti for ColMultiStruct<'a,T,I,F,F2,F3>{

        type T=T;
        fn identity(&self)->I{
            (self.b)()
        }
        fn add(&self,a:&mut I,b:&I){
            (self.c)(a,b);
        }
        fn collide(&self,a:ColPair<T>){
            (self.a)(a);
        }

    }
    
    
}