use crate::pmut::PMut;
use itertools::Itertools;

use alloc::vec::Vec;

//They are always send and sync because the only time the vec is used
//is when it is borrowed for the lifetime.
unsafe impl<T> core::marker::Send for PreVecMut<T> {}
unsafe impl<T> core::marker::Sync for PreVecMut<T> {}

///An vec api to avoid excessive dynamic allocation by reusing a Vec
#[derive(Default)]
pub struct PreVecMut<T> {
    vec: Vec<core::ptr::NonNull<T>>,
}

impl<T> PreVecMut<T> {
    #[inline(always)]
    pub fn new() -> PreVecMut<T> {
        debug_assert_eq!(
            core::mem::size_of::<core::ptr::NonNull<T>>(),
            core::mem::size_of::<&mut T>()
        );

        PreVecMut { vec: Vec::new() }
    }

    ///Clears the vec and returns a mutable reference to a vec.
    #[inline(always)]
    pub fn get_empty_vec_mut<'a, 'b: 'a>(&'a mut self) -> &'a mut Vec<PMut<'b, T>> {
        self.vec.clear();
        let v: &mut Vec<_> = &mut self.vec;
        unsafe { &mut *(v as *mut _ as *mut Vec<_>) }
    }
}

///Splits a mutable slice into multiple slices
///The splits occur where the predicate returns false.
pub struct SliceSplitMut<'a, T, F> {
    arr: Option<&'a mut [T]>,
    func: F,
}

impl<'a, T, F: FnMut(&T, &T) -> bool> SliceSplitMut<'a, T, F> {
    pub fn new(arr: &'a mut [T], func: F) -> SliceSplitMut<'a, T, F> {
        SliceSplitMut {
            arr: Some(arr),
            func,
        }
    }
}

impl<'a, T, F: FnMut(&T, &T) -> bool> Iterator for SliceSplitMut<'a, T, F> {
    type Item = &'a mut [T];
    fn next(&mut self) -> Option<Self::Item> {
        let (last, arr) = {
            let arr = self.arr.take()?;
            let i = arr.get(0)?;
            let count = arr.iter().peeking_take_while(|a| (self.func)(a, i)).count();
            (count, arr)
        };
        let (first, rest) = arr.split_at_mut(last);
        self.arr = Some(rest);
        Some(first)
    }
}

///Splits a mutable slice into multiple slices
///The splits occur where the predicate returns false.
pub struct SliceSplit<'a, T, F> {
    arr: Option<&'a [T]>,
    func: F,
}
impl<'a, T, F: FnMut(&T, &T) -> bool> SliceSplit<'a, T, F> {
    pub fn new(arr: &'a [T], func: F) -> SliceSplit<'a, T, F> {
        SliceSplit {
            arr: Some(arr),
            func,
        }
    }
}
impl<'a, T, F: FnMut(&T, &T) -> bool> Iterator for SliceSplit<'a, T, F> {
    type Item = &'a [T];
    fn next(&mut self) -> Option<Self::Item> {
        let (last, arr) = {
            let arr = self.arr.take()?;
            let i = arr.get(0)?;
            let count = arr.iter().peeking_take_while(|a| (self.func)(a, i)).count();
            (count, arr)
        };
        let (first, rest) = arr.split_at(last);
        self.arr = Some(rest);
        Some(first)
    }
}





//TODO use this!!!!
pub mod small_ref{
    use core::marker::PhantomData;
    pub struct SmallRef<'a,T>(u16,PhantomData<&'a mut T>);


    pub struct Base<'a,T>(*const [T],PhantomData<&'a T>);

    impl<'a,T> Base<'a,T>{
        fn conv_mut(&'a self,b:&'a mut SmallRef<'a,T>)->&'a mut T{
            let k=unsafe{&*self.0};
            let j=&k[b.0 as usize] as *const _;
            let l=unsafe{&mut *(j as *mut _)};
            l
        }
    }


    pub struct IterMut<'a,T>{
        counter:usize,
        length:usize,
        _p:PhantomData<&'a mut T>
    }
    impl<'a,T> Iterator for IterMut<'a,T>{
        type Item=SmallRef<'a,T>;
        fn next(&mut self)->Option<SmallRef<'a,T>>{
            let k=if self.counter>=self.length{
                None
            }else{
                Some(SmallRef(self.counter as u16,PhantomData))
            };
            self.counter+=1;
            k
        }
    }

    pub fn make<'a,T>(arr:&'a mut [T])->(Base<'a,T>,IterMut<'a,T>){
        assert!(arr.len()<u16::max_value() as usize);
        let base=Base(arr as *const _,PhantomData);
        let it=IterMut{counter:0,length:arr.len(),_p:PhantomData};
        (base,it)
    }



}