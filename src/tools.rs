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


pub use self::undo_iterator::UndoIterator;
mod undo_iterator{
    use smallvec::SmallVec;
    pub struct UndoIterator<I:Iterator>{
        it:I,
        item:SmallVec<[I::Item;1]>
    }

    impl<I:Iterator> UndoIterator<I>{
        pub fn new(it:I)->UndoIterator<I>{
            UndoIterator{it,item:SmallVec::new()}
        }
        pub fn add_back(&mut self,item:I::Item){
            self.item.push(item);
        }

        //We deliberately do not implement Iterator
        //since this violates the model of an Iterator
        //being able to 'put things back' into it.
        pub fn next(&mut self)->Option<I::Item>{
            match self.item.pop(){
                Some(x)=>{
                    Some(x)
                },
                None=>{
                    self.it.next()
                }
            }
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

///An vec api to avoid excessive dynamic allocation by reusing a Vec
pub struct PreVecMut<T> {
    vec: smallvec::SmallVec<[*mut T; 64]>,
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
        let v: &mut smallvec::SmallVec<[*mut T; 64]> = &mut self.vec;
        unsafe { std::mem::transmute(v) }
    }
}



unsafe impl<T: Send> std::marker::Send for PreVec<T> {}
unsafe impl<T: Sync> std::marker::Sync for PreVec<T> {}

///An vec api to avoid excessive dynamic allocation by reusing a Vec
pub struct PreVec<T> {
    vec: smallvec::SmallVec<[*const T; 64]>,
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
    pub fn get_empty_vec<'a>(&'a mut self) -> &mut smallvec::SmallVec<[&'a T; 64]> {
        self.vec.clear();
        let v: &mut smallvec::SmallVec<[*const T; 64]> = &mut self.vec;
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
