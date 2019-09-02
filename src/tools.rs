use core;
use core::marker::PhantomData;
use alloc::vec::Vec;
use dinotree::prelude::*;


pub fn for_every_pair<T:HasAabbMut>(mut arr:ElemSliceMut<T>,mut func:impl FnMut(BBoxRefMut<T::Num,T::Inner>,BBoxRefMut<T::Num,T::Inner>)){
    loop{
        let temp=arr;
        match temp.split_first_mut(){
            Some((mut b1,mut x))=>{
                for b2 in x.as_mut().iter_mut(){
                    func(b1.as_mut(),b2);
                }
                arr=x;
            },
            None=>break
        }
    }
}


///A phantom data type that unsafely implements send,sync.
pub(crate) struct PhantomSendSync<T>(pub PhantomData<T>);
unsafe impl<T> Send for PhantomSendSync<T> {}
unsafe impl<T> Sync for PhantomSendSync<T> {}
impl<T> Copy for PhantomSendSync<T> {}
impl<T> Clone for PhantomSendSync<T> {
    fn clone(&self) -> PhantomSendSync<T> {
        *self
    }
}



//They are always send and sync because the only time the vec is used
//is when it is borrowed for the lifetime.
unsafe impl<N:NumTrait,T> core::marker::Send for PreVecMut<N,T> {}
unsafe impl<N:NumTrait,T> core::marker::Sync for PreVecMut<N,T> {}



///An vec api to avoid excessive dynamic allocation by reusing a Vec
pub struct PreVecMut<N:NumTrait,T> {
    vec:Vec<BBoxRefPtr<N,T>>
}
impl<N:NumTrait,T> PreVecMut<N,T> {
    #[inline(always)]
    pub fn new() -> PreVecMut<N,T> {
        PreVecMut {
            vec:Vec::new()
        }
    }

    ///Clears the vec and returns a mutable reference to a vec.
    #[inline(always)]
    pub fn get_empty_vec_mut<'a,'b:'a>(&'a mut self) -> &'a mut Vec<BBoxRefMut<'b,N,T>> {
        self.vec.clear();
        let v: &mut Vec<_> = &mut self.vec;
        unsafe{&mut *(v as *mut _ as *mut Vec<BBoxRefMut<'b,N,T>>)}
    }
}


