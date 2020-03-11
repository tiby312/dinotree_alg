

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

    pub fn collect_all<D>(&mut self,mut func:impl FnMut(&Rect<N>,&mut T)->Option<D>)->SingleCollisionList<'a,T,D>{
         
        let a=self.tree.as_tree_mut().collect_all(|a,b|{
            match func(a,unsafe{b.as_mut()}){
                Some(d)=>{
                    Some((*b,d))
                },
                None=>{
                    None
                }
            }
        });
        SingleCollisionList{_p:PhantomData,a}
    }
}


#[derive(Copy,Clone)]
pub struct Collision<T>{
    pub a:T,
    pub b:T,
}

impl<'a,A:Axis+Send+Sync,N:Num+Send+Sync,T:Send+Sync> CollectableDinoTree<'a,A,N,T>{

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
}

pub struct BotCollision<'a,T,D>{
    _p:PhantomData<&'a mut T>,
    cols:CollisionList<(Collision<MyPtr<T>>,D)>
}

impl<'a,T,D> BotCollision<'a,T,D>{
    pub fn for_every_pair<'b,A:Axis,N:Num>(&'b mut self,_:&'b mut CollectableDinoTree<'a,A,N,T>,mut func:impl FnMut(&mut T,&mut T,&mut D)){
        
        self.cols.for_every_pair_mut(|(Collision{a,b},d)|{
            let a=unsafe{&mut *a.as_mut()};
            let b=unsafe{&mut *b.as_mut()};
            func(a,b,d)
        })
    }
}
impl<'a,T:Send+Sync,D:Send+Sync> BotCollision<'a,T,D>{
    pub fn for_every_pair_par<'b,A:Axis,N:Num>(&'b mut self,_:&'b mut CollectableDinoTree<'a,A,N,T>,func:impl Fn(&mut T,&mut T,&mut D)+Send+Sync+Copy){
        
        self.cols.for_every_pair_par_mut(|(Collision{a,b},d)|{
            let a=unsafe{&mut *a.as_mut()};
            let b=unsafe{&mut *b.as_mut()};
            func(a,b,d)
        })
    }
}