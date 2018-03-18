use std;
use std::marker::PhantomData;

///A phantom data type that unsafely implements send,sync.
pub struct PhantomSendSync<T>(pub PhantomData<T>);
unsafe impl<T> Send for PhantomSendSync<T> {}
unsafe impl<T> Sync for PhantomSendSync<T> {}
impl<T> Copy for PhantomSendSync<T> {}
impl<T> Clone for PhantomSendSync<T> {
    fn clone(&self) -> PhantomSendSync<T> {
        *self
    }
}

unsafe impl<T: Send> std::marker::Send for PreVec<T> {}
use smallvec;
///An vec api to avoid excessive dynamic allocation by reusing a Vec
pub struct PreVec<T> {
    vec: smallvec::SmallVec<[*mut T; 64]>,
}
impl<T> PreVec<T> {
    #[inline(always)]
    pub fn new() -> PreVec<T> {
        PreVec {
            vec: smallvec::SmallVec::new(),
        }
    }

    ///Clears the vec and returns a mutable reference to a vec.
    #[inline(always)]
    pub fn get_empty_vec_mut<'a>(&'a mut self) -> &mut smallvec::SmallVec<[&'a mut T; 64]> {
        self.vec.clear();
        let v: &mut smallvec::SmallVec<[*mut T; 64]> = &mut self.vec;
        unsafe { std::mem::transmute(v) }
    }
}

/*
///An vec api to avoid excessive dynamic allocation by reusing a Vec
pub struct PreVec<T>{
    vec:Vec<* mut T>
}
impl<T> PreVec<T>{
    #[inline(always)]
    pub fn new()->PreVec<T>{
        PreVec{vec:Vec::new()}
    }

    #[inline(always)]
    pub fn with_capacity(size:usize)->PreVec<T>{
        PreVec{vec:Vec::with_capacity(size)}
    }

    ///Clears the vec and returns a mutable reference to a vec.
    #[inline(always)]
    pub fn get_empty_vec_mut<'a>(&'a mut self)->&mut Vec<&'a mut T>{
        self.vec.clear();
        let v:&mut Vec<*mut T> = &mut self.vec;
        unsafe{std::mem::transmute(v)}
    }
}
*/

/*
///Returns a combined slice given two slices that are next to each other in memory.
///Panics if they are not next to each other.
pub fn join<'a,T>(first: &'a [T],second:&'a [T])->&'a [T]{
    let f1=first.len();
    if first[f1..].as_ptr() == second.as_ptr(){
        unsafe{
            return std::slice::from_raw_parts(first.as_ptr(),f1+second.len());
        }
    }else{
        panic!("Slices are not next to each other in memory.");
    }
}
*/
