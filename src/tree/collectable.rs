

use dinotree_owned::MyPtr;

use dinotree_owned::myptr;

use core::ptr::NonNull;
use dinotree_owned::*;
use super::*;
pub struct CollectableDinoTree<'a,A:Axis,N:Num,T>{
    bots:&'a mut [T],
    tree:DinoTreeOwned<A,BBox<N,MyPtr<T>>>
}
impl<'a,N:Num,T> CollectableDinoTree<'a,DefaultA,N,T>{
    pub fn new(bots:&'a mut [T],mut func:impl FnMut(&mut T)->Rect<N>)->CollectableDinoTree<'a,DefaultA,N,T>{
        let  bboxes:Vec<_>=bots.iter_mut().map(|a|BBox::new(func(a),myptr(a))).collect();

        let tree=DinoTreeOwned::new(bboxes);

        CollectableDinoTree{bots,tree}
    }
}
impl<'a,A:Axis,N:Num,T> CollectableDinoTree<'a,A,N,T>{

    pub fn get_bots_mut(&mut self)->&mut [T]{
        self.bots
    }

    pub fn get_mut(&mut self)->&mut DinoTree<A,NodeMut<BBox<N,&mut T>>>{
        let k=self.tree.as_tree_mut() as *mut _;
        let j=k as *mut DinoTree<A,NodeMut<BBox<N,&mut T>>>;
        unsafe{&mut *j}
    }

    /*


    pub fn collect_all<D>(&mut self,mut func:impl FnMut(&Rect<N::Num>,&mut <N::T as HasInner>::Inner)->Option<D>)->Vec<D>{
        let mut res=Vec::new();
        for node in self.inner.get_nodes_mut().iter_mut(){
            for b in node.get_mut().bots.iter_mut(){
                let (x,y)=b.unpack();
                if let Some(d)=func(x,y){
                    res.push(d);
                }
            }
        }
        res
    }*/

}


#[derive(Copy,Clone)]
pub struct Collision<T>{
    pub a:T,
    pub b:T,
}

impl<'a,A:Axis+Send+Sync,N:Num+Send+Sync,T:Send+Sync> CollectableDinoTree<'a,A,N,T>{

    pub fn collect_all_par<D:Send+Sync>(&mut self,func:impl Fn(&Rect<N>,&mut T)->Option<D>+Send+Sync+Copy)->SingleCollisionList<'a,T,D>{
         
        let tree=self.tree.as_tree_mut();
        use rayon::prelude::*;

        let par=tree.inner.get_nodes_mut().par_iter_mut().map(|node|{
            let mut a=Vec::new();
            for b in node.get_mut().bots.iter_mut(){
                let (x,y)=b.unpack();
                if let Some(d)=func(x,unsafe{y.as_mut()}){
                    a.push((*y,d))
                }
            }
            a
        }).flat_map(|a|a);

        let a:Vec<_>=par.collect();

        SingleCollisionList{_p:PhantomData,a}
    }
    pub fn collect_collisions_list_par <D:Send+Sync>(&mut self,func:impl Fn(&mut T,&mut T)->Option<D>+Send+Sync+Copy)->BotCollision<'a,T,D>{
    
        let cols=self.tree.as_tree_mut().collect_collisions_list_par(|a,b|{
            match func(unsafe{a.as_mut()},unsafe{b.as_mut()}){
                Some(d)=>{
                    Some((Collision{a:*a,b:*b},d))
                },
                None=>{
                    None
                }
            }
        });
        BotCollision{cols,_p:PhantomData}
    }
}

use core::marker::PhantomData;
pub struct SingleCollisionList<'a,T,D>{
    _p:PhantomData<&'a mut T>,
    a:Vec<(MyPtr<T>,D)>
}
impl<'a,T,D> SingleCollisionList<'a,T,D>{
    pub fn for_every<'b,A:Axis,N:Num>(&'b mut self,_:&'b mut CollectableDinoTree<'a,A,N,T>,mut func:impl FnMut(&mut T,&mut D)){
        for (a,d) in self.a.iter_mut(){
            func(unsafe{&mut *a.as_mut()},d)
        }
    }

    pub fn get<'b,A:Axis,N:Num>(&self,_:&'b CollectableDinoTree<'a,A,N,T>)->&[(&T,D)]{
        let k=unsafe{&*(self.a.as_slice() as *const _ as *const [(&T,D)])};
        k
    }
}
impl<'a,T:Send+Sync,D:Send+Sync> SingleCollisionList<'a,T,D>{
    pub fn for_every_par<'b,A:Axis,N:Num>(&'b mut self,_:&'b mut CollectableDinoTree<'a,A,N,T>,func:impl Fn(&mut T,&mut D)+Send+Sync+Copy){
        use rayon::prelude::*;
        self.a.par_iter_mut().for_each(|(a,d)|{
            func(unsafe{&mut *a.as_mut()},d)
        });
    }

}

pub struct BotCollision<'a,T,D>{
    _p:PhantomData<&'a mut T>,
    cols:CollisionList<(Collision<MyPtr<T>>,D)>
}

impl<'a,T,D> BotCollision<'a,T,D>{
    ///IMPORTANT iter_mut() not allowed since user could store the returned mutable references, but iter() is allowed.
    pub fn iter<'b,A:Axis,N:Num>(&self,_:&'b CollectableDinoTree<'a,A,N,T>)->impl Iterator<Item=((&T,&T,&D))>{
        self.cols.iter().map(|(Collision{a,b},d)|{
            let a=unsafe{&*a.as_ref()};
            let b=unsafe{&*b.as_ref()};
            (a,b,d)
        })
    }
    pub fn for_every_pair<'b,A:Axis,N:Num>(&'b mut self,_:&'b mut CollectableDinoTree<'a,A,N,T>,mut func:impl FnMut(&mut T,&mut T,&mut D)){
        
        self.cols.for_every_pair_mut(|(Collision{a,b},d)|{
            let a=unsafe{&mut *a.as_mut()};
            let b=unsafe{&mut *b.as_mut()};
            func(a,b,d)
        })
    }
}
impl<'a,T:Send+Sync,D:Send+Sync> BotCollision<'a,T,D>{
    /*
    pub fn par_iter(&self)->rayon::slice::Iter<Vec<(Collision<&T>,D)>>{
        use rayon::prelude::*;
        let sl=unsafe{& *((&self.cols.nodes) as *const _ as *const Vec<Vec<(Collision<&T>,D)>>)};
        let ss:& [_]=sl;
        ss.par_iter()
    }
    */
    pub fn for_every_pair_par<'b,A:Axis,N:Num>(&'b mut self,_:&'b mut CollectableDinoTree<'a,A,N,T>,func:impl Fn(&mut T,&mut T,&mut D)+Send+Sync+Copy){
        
        self.cols.for_every_pair_par_mut(|(Collision{a,b},d)|{
            let a=unsafe{&mut *a.as_mut()};
            let b=unsafe{&mut *b.as_mut()};
            func(a,b,d)
        })
    }
}
