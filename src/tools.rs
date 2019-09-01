use core;
use core::marker::PhantomData;
use alloc::vec::Vec;
use dinotree::SlicePin;
use core::pin::Pin;

pub fn for_every_pair<T>(mut arr:&mut SlicePin<T>,mut func:impl FnMut(Pin<&mut T>,Pin<&mut T>)){
    loop{
        let temp=arr;
        match temp.split_first_mut(){
            Some((mut b1,x))=>{
                for b2 in x.iter_mut(){
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
unsafe impl<T> core::marker::Send for PreVecMut<T> {}
unsafe impl<T> core::marker::Sync for PreVecMut<T> {}



use dinotree::advanced::Unique;
///An vec api to avoid excessive dynamic allocation by reusing a Vec
#[derive(Clone)]
pub struct PreVecMut<T> {
    vec:Vec<Unique<T>>
}
impl<T> PreVecMut<T> {
    #[inline(always)]
    pub fn new() -> PreVecMut<T> {
        PreVecMut {
            vec:Vec::new()
        }
    }

    ///Clears the vec and returns a mutable reference to a vec.
    #[inline(always)]
    pub fn get_empty_vec_mut<'a,'b:'a>(&'a mut self) -> &'a mut Vec<Pin<&'b mut T>> {
        self.vec.clear();
        let v: &mut Vec<Unique<T>> = &mut self.vec;
        unsafe{&mut *(v as *mut Vec<Unique<T>> as *mut Vec<Pin<&'b mut T>>)}
    }
}


