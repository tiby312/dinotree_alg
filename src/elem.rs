use crate::inner_prelude::*;


///Forbids the user from swapping two nodes around.
#[repr(transparent)]
pub struct ProtectedNode<'a,T>{
    inner:&'a mut T
}
impl<'a,T:NodeTrait> ProtectedNode<'a,T>{
    pub fn new(inner:&'a mut T)->Self{
        ProtectedNode{inner}
    }

    pub fn get(self)->NodeRef<'a,T::T>{
        self.inner.get()
    }
    pub fn get_mut(self)->NodeRefMut<'a,T::T>{
        self.inner.get_mut()
    }
    pub fn as_ref(&mut self)->ProtectedNode<T>{
        ProtectedNode{inner:self.inner}
    }
}


///Forbids the user from swapping aabb's around.
#[repr(transparent)]
pub struct ProtectedBBox<'a,T>{
    inner:&'a mut T
}


impl<'a,T> ProtectedBBox<'a,T>{
    #[inline(always)]
    pub fn as_mut(&mut self)->ProtectedBBox<T>{
        ProtectedBBox{inner:self.inner}
    }

}





unsafe impl<'a,T:HasAabb> HasAabb for ProtectedBBox<'a,T>{
    type Num=T::Num;
    #[inline(always)]
    fn get(&self)->&Rect<Self::Num>{
        self.inner.get()
    }
}
impl<'a,T:HasInner> HasInner for ProtectedBBox<'a,T>{
    type Inner=T::Inner;
    #[inline(always)]
    fn get_inner(&self)->(&Rect<T::Num>,&Self::Inner){
        self.inner.get_inner()
    }

    #[inline(always)]
    fn get_inner_mut(&mut self)->(&Rect<T::Num>,&mut Self::Inner){
        self.inner.get_inner_mut()
    }
}




impl<'a,T> core::borrow::Borrow<T> for ProtectedBBox<'a,T>{
    #[inline(always)]
    fn borrow(&self)->&T{
        self.inner
    }
}

impl<'a,T> AsRef<T> for ProtectedBBox<'a,T>{
    #[inline(always)]
    fn as_ref(&self)->&T{
        self.inner
    }
}









impl<'a,T> core::borrow::Borrow<[T]> for ProtectedBBoxSlice<'a,T>{
    #[inline(always)]
    fn borrow(&self)->&[T]{
        self.inner
    }
}




impl<'a, T> core::iter::IntoIterator for ProtectedBBoxSlice<'a,T> {
    type Item = ProtectedBBox<'a,T>;
    type IntoIter = ProtectedBBoxIter<'a,T>;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<'a,T> AsRef<[T]> for ProtectedBBoxSlice<'a,T>{
    #[inline(always)]
    fn as_ref(&self)->&[T]{
        self.inner
    }
}


///Forbids the user from swapping mutable slices in the nodes around.
#[repr(transparent)]
pub struct ProtectedBBoxSlice<'a,T>{
    inner:&'a mut [T]
}

impl<'a,T> ProtectedBBoxSlice<'a,T>{
    
    #[inline(always)]
    pub fn len(&self)->usize{
        self.inner.len()
    }

    #[inline(always)]
    pub fn is_empty(&self)->bool{
        self.inner.is_empty()
    }

    #[inline(always)]
    pub fn split_first_mut(self)->Option<(ProtectedBBox<'a,T>,ProtectedBBoxSlice<'a,T>)>{
        self.inner.split_first_mut().map(|(first,inner)|(ProtectedBBox{inner:first},ProtectedBBoxSlice::new(inner)))
    }


    #[inline(always)]
    pub fn truncate_to(self,a:core::ops::RangeTo<usize>)->Self{
        ProtectedBBoxSlice{inner:&mut self.inner[a]}
    }
    #[inline(always)]
    pub fn truncate_from(self,a:core::ops::RangeFrom<usize>)->Self{
        ProtectedBBoxSlice{inner:&mut self.inner[a]} 
    }


    #[inline(always)]
    pub fn truncate(self,a:core::ops::Range<usize>)->Self{
        ProtectedBBoxSlice{inner:&mut self.inner[a]}
    }

    #[inline(always)]
    pub fn as_mut(&mut self)->ProtectedBBoxSlice<T>{
        ProtectedBBoxSlice{inner:self.inner}
    }

    #[inline(always)]
    pub fn new(inner:&'a mut [T])->Self{
        ProtectedBBoxSlice{inner}
    }

    #[inline(always)]
    pub fn iter(self)->core::slice::Iter<'a,T>{
        self.inner.iter()
    }
    #[inline(always)]
    pub fn iter_mut(self)->ProtectedBBoxIter<'a,T>{
        ProtectedBBoxIter{inner:self.inner.iter_mut()}
    }
}

///Iterator produced by `ProtectedBBoxSlice<T>` that generates `ProtectedBBox<T>`
pub struct ProtectedBBoxIter<'a,T>{
    inner:core::slice::IterMut<'a,T>
}
impl<'a,T> Iterator for ProtectedBBoxIter<'a,T>{
    type Item=ProtectedBBox<'a,T>;

    #[inline(always)]
    fn next(&mut self)->Option<ProtectedBBox<'a,T>>{
        self.inner.next().map(|inner|ProtectedBBox{inner})
    }

    #[inline(always)]
    fn size_hint(&self)->(usize,Option<usize>){
        self.inner.size_hint()
    }
}

impl<'a,T> core::iter::FusedIterator for ProtectedBBoxIter<'a,T>{}
impl<'a,T> core::iter::ExactSizeIterator for ProtectedBBoxIter<'a,T>{}

impl<'a, T> DoubleEndedIterator for ProtectedBBoxIter<'a, T> {
    #[inline(always)]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back().map(|inner|ProtectedBBox{inner})
    }
}



use alloc::vec::Vec;

//They are always send and sync because the only time the vec is used
//is when it is borrowed for the lifetime.
unsafe impl<T> core::marker::Send for PreVecMut<T> {}
unsafe impl<T> core::marker::Sync for PreVecMut<T> {}

///An vec api to avoid excessive dynamic allocation by reusing a Vec
#[derive(Default)]
pub struct PreVecMut<T> {
    vec:Vec<core::ptr::NonNull<T>>
}

impl<T> PreVecMut<T> {
    
    #[inline(always)]
    pub fn new() -> PreVecMut<T> {

        debug_assert_eq!(core::mem::size_of::<core::ptr::NonNull<T>>() ,core::mem::size_of::<&mut T>() );

        PreVecMut {
            vec:Vec::new()
        }
    }


    ///Clears the vec and returns a mutable reference to a vec.
    #[inline(always)]
    pub fn get_empty_vec_mut<'a,'b:'a>(&'a mut self) -> &'a mut Vec<ProtectedBBox<'b,T>> {
        self.vec.clear();
        let v: &mut Vec<_> = &mut self.vec;
        unsafe{&mut *(v as *mut _ as *mut Vec<_>)}
    }    
}


