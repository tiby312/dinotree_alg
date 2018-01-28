use std;
use SweepTrait;
use axgeom;
use NumTrait;
use ordered_float::NotNaN;
use DepthLevel;
use compt::LevelDesc;

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


///A generic container that implements the kdtree trait.
#[derive(Copy,Clone,Debug)]
pub struct BBox<Nu:NumTrait,T:Send>{
    rect:axgeom::Rect<Nu>,
    pub val:T
}

impl<Nu:NumTrait,T:Send> SweepTrait for BBox<Nu,T>{
    type Inner=T;
    type Num=Nu;
    fn get_mut<'a>(&'a mut self)->(&'a axgeom::Rect<Nu>,&'a mut Self::Inner){
        (&self.rect,&mut self.val)
    }
    fn get<'a>(&'a self)->(&'a axgeom::Rect<Nu>,&'a Self::Inner){
        (&self.rect,&self.val)
    }
}

impl<Nu:NumTrait,T:Send> BBox<Nu,T>{

    #[inline(always)]
    pub fn new(val:T,r:axgeom::Rect<Nu>)->BBox<Nu,T>{
        BBox{rect:r,val:val}
    }

    #[inline(always)]
    pub fn update_box(&mut self,rect:axgeom::Rect<Nu>){
        self.rect=rect;
    }
}


///A default depth level from which to switch to sequential.
pub struct DefaultDepthLevel;

impl DepthLevel for DefaultDepthLevel{
    fn switch_to_sequential(a:LevelDesc)->bool{
        a.get_depth()>4
    }
}