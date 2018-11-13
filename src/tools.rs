use std;
use std::marker::PhantomData;


pub fn for_every_pair<T>(mut arr:&mut [T],mut func:impl FnMut(&mut T,&mut T)){
    loop{
        let temp=arr;
        match temp.split_first_mut(){
            Some((b1,x))=>{
                for b2 in x.iter_mut(){
                    func(b1,b2);
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

use smallvec;


unsafe impl<T: Send> std::marker::Send for PreVecMut<T> {}
unsafe impl<T: Sync> std::marker::Sync for PreVecMut<T> {}


use std::ptr::Unique;


///An vec api to avoid excessive dynamic allocation by reusing a Vec
pub struct PreVecMut<T> {
    vec: smallvec::SmallVec<[Unique<T>; 64]>,
}
impl<T> PreVecMut<T> {
    #[inline(always)]
    pub fn new() -> PreVecMut<T> {
        PreVecMut {
            vec: smallvec::SmallVec::new(),
        }
    }

    ///Clears the vec and returns a mutable reference to a vec.
    #[inline(always)]
    pub fn get_empty_vec_mut<'a>(&'a mut self) -> &mut smallvec::SmallVec<[&'a mut T; 64]> {
        self.vec.clear();
        let v: &mut smallvec::SmallVec<[Unique<T>; 64]> = &mut self.vec;
        unsafe { std::mem::transmute(v) }
    }
}


