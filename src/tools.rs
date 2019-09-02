use core;
use core::marker::PhantomData;
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

