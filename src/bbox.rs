use crate::inner_prelude::*;


///Equivalent to: `&mut (Rect<N>,T)`
#[repr(transparent)]
pub struct BBoxIndirect<'a,T>{
    pub inner: &'a mut T
}
impl<'a,T> BBoxIndirect<'a,T>{
    pub fn new(inner:&'a mut T)->Self{
        BBoxIndirect{inner}
    }
}


unsafe impl<'a,T:HasAabb> HasAabb for BBoxIndirect<'a,T> {
    type Num = T::Num;
    #[inline(always)]
    fn get(&self) -> &Rect<Self::Num>{
        self.inner.get()
    }
}
impl<'a,T:HasInner> HasInner for BBoxIndirect<'a,T>{
    type Inner= T::Inner;

    #[inline(always)]
    fn get_inner(&self)->(&Rect<T::Num>,&Self::Inner){
        self.inner.get_inner()
    }

    #[inline(always)]
    fn get_inner_mut(&mut self)->(&Rect<T::Num>,&mut Self::Inner){
        self.inner.get_inner_mut()
    }
}





///Equivalent to: `(Rect<N>,&mut T)` 
#[repr(C)]
pub struct BBoxMut<'a,N, T> {
    pub rect: axgeom::Rect<N>,
    pub inner: &'a mut T,
}

impl<'a,N, T> BBoxMut<'a,N, T> {
    #[inline(always)]
    pub fn new(rect: axgeom::Rect<N>, inner: &'a mut T) -> BBoxMut<'a,N, T> {
        BBoxMut { rect, inner }
    }
}


unsafe impl<'a,N: NumTrait, T> HasAabb for BBoxMut<'a,N, T> {
    type Num = N;
    #[inline(always)]
    fn get(&self) -> &Rect<Self::Num>{
        &self.rect
    }
}
impl<'a,N:NumTrait,T> HasInner for BBoxMut<'a,N,T>{
    type Inner= T;

    #[inline(always)]
    fn get_inner(&self)->(&Rect<N>,&Self::Inner){
        (&self.rect,self.inner)
    }

    #[inline(always)]
    fn get_inner_mut(&mut self)->(&Rect<N>,&mut Self::Inner){
        (&self.rect,self.inner)
    }
}






#[derive(Copy, Clone)]
#[repr(C)]
///Equivalent to: `(Rect<N>,T)` 
pub struct BBox<N, T> {
    pub rect: axgeom::Rect<N>,
    pub inner: T,
}


impl<N, T> BBox<N, T> {
    #[inline(always)]
    pub fn new(rect: axgeom::Rect<N>, inner: T) -> BBox<N, T> {
        BBox { rect, inner }
    }
}



unsafe impl<N: NumTrait, T> HasAabb for &mut BBox<N, T> {
    type Num = N;
    #[inline(always)]
    fn get(&self) -> &Rect<Self::Num>{
        &self.rect
    }
}
impl<N:NumTrait,T> HasInner for &mut BBox<N,T>{
    type Inner= T;

    #[inline(always)]
    fn get_inner(&self)->(&Rect<N>,&Self::Inner){
        (&self.rect,&self.inner)
    }

    #[inline(always)]
    fn get_inner_mut(&mut self)->(&Rect<N>,&mut Self::Inner){
        (&self.rect,&mut self.inner)
    }
}




unsafe impl<N: NumTrait, T> HasAabb for BBox<N, T> {
    type Num = N;
    #[inline(always)]
    fn get(&self) -> &Rect<Self::Num>{
        &self.rect
    }
}
impl<N:NumTrait,T> HasInner for BBox<N,T>{
    type Inner= T;

    #[inline(always)]
    fn get_inner(&self)->(&Rect<N>,&Self::Inner){
        (&self.rect,&self.inner)
    }

    #[inline(always)]
    fn get_inner_mut(&mut self)->(&Rect<N>,&mut Self::Inner){
        (&self.rect,&mut self.inner)
    }
}

