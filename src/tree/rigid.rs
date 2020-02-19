
//TODO cleanup


use crate::inner_prelude::*;

unsafe impl<T> Send for Cpair<T>{}
unsafe impl<T> Sync for Cpair<T>{}

#[derive(Debug)]
pub(crate) struct Cpair<T>([*mut T;2]);
impl<T> Cpair<T>{
    #[inline(always)]
    pub(crate) fn get_mut(&mut self)->[&mut T;2]{
        let [a,b]=&mut self.0;
        unsafe{[&mut **a,&mut **b]}
    }
    #[inline(always)]
    pub(crate) fn new(a:&mut T,b:&mut T)->Cpair<T>{
        Cpair([a as *mut _,b as *mut _])
    }
}


pub struct CollisionList<'a,T,K>{
    pub(crate) _p:core::marker::PhantomData<&'a mut T>,
    pub(crate) vec:Vec<(Cpair<T>,K)>
}

impl<'a,T,K> CollisionList<'a,T,K>{
    pub fn for_every_collision(&mut self,mut func:impl FnMut(PMut<T>,PMut<T>,&mut K)){
        for a in self.vec.iter_mut(){
            let (a,b)=a;
            let [c,d]=a.get_mut();
            (func)(PMut::new(c),PMut::new(d),b);
        }
    }
}
