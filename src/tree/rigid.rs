
//TODO cleanup


use crate::inner_prelude::*;
use compt::Visitor;

pub fn create_collision_list<'a,A:Axis,N:Node + Send+Sync,D>
                (tree:&'a mut DinoTree<A,N>,func:impl Fn(&mut <N::T as HasInner>::Inner,&mut <N::T as HasInner>::Inner)->Option<D>+Send+Sync)->CollisionList<'a,<N::T as HasInner>::Inner,D>
                where N::T:Aabb+HasInner+Send+Sync{

    struct Foo<T:Visitor>{
        current:T::Item,
        next:Option<[T;2]>,
    }
    impl<T:Visitor> Foo<T>{
        fn new(a:T)->Foo<T>{
            let (n,f)=a.next();
            Foo{current:n,next:f}
        }
    }

    let height=1+par::compute_level_switch_sequential(par::SWITCH_SEQUENTIAL_DEFAULT,tree.get_height()).get_depth_to_switch_at();
    //dbg!(tree.get_height(),height);
    let mut nodes:Vec<Vec<Collision<<N::T as HasInner>::Inner,D>>>=(0..compt::compute_num_nodes(height)).map(|_|Vec::new()).collect();
    let mtree=compt::dfs_order::CompleteTree::from_preorder_mut(&mut nodes).unwrap();
    
    tree.find_collisions_mut_par_ext(|a|{
        let next=a.next.take();
        if let Some([left,right])=next{
            let l=Foo::new(left);
            let r=Foo::new(right);
            *a=l;
            r
        }else{
            unreachable!()
        }
    },|_a,_b|{},|c,a,b|{
        if let Some(d)=func(a,b){
            c.current.push(Collision::new(a,b,d));
        }
    },Foo::new(mtree.vistr_mut()));

    CollisionList{nodes,_p:PhantomData}
}

struct Collision<T,D>{
    a:*mut T,
    b:*mut T,
    d:D
}
impl<T,D> Collision<T,D>{
    fn new(a:&mut T,b:&mut T,d:D)->Self{
        Collision{a:a as *mut _,b:b as *mut _,d}
    }
}
unsafe impl<T,D> Send for Collision<T,D>{}
unsafe impl<T,D> Sync for Collision<T,D>{}

fn parallelize<T:Visitor+Send+Sync>(a:T,func:impl Fn(T::Item)+Sync+Send+Copy) where T::Item:Send+Sync{
    let (n,l)=a.next();
    func(n);
    if let Some([left,right])=l{
        rayon::join(||parallelize(left,func),||parallelize(right,func));
    }
}

use core::marker::PhantomData;
pub struct CollisionList<'a,T,D>{
    _p:PhantomData<&'a T>,
    nodes:Vec<Vec<Collision<T,D>>>
}
impl<'a,T:Send+Sync,D:Send+Sync> CollisionList<'a,T,D>{
    pub fn for_every_pair_seq_mut(&mut self,mut func:impl FnMut(&mut T,&mut T,&mut D)+Send+Sync+Copy){
        for a in self.nodes.iter_mut(){
            for c in a.iter_mut(){
                let a=unsafe{&mut *c.a};
                let b=unsafe{&mut *c.b};
                func(a,b,&mut c.d)
            }
        }
    }
    pub fn for_every_pair_par_mut(&mut self,func:impl Fn(&mut T,&mut T,&mut D)+Send+Sync+Copy){
        /*
        for a in self.nodes.iter(){
            print!("{},",a.len());
        }
        println!();
        */
        let mtree=compt::dfs_order::CompleteTree::from_preorder_mut(&mut self.nodes).unwrap();

        parallelize(mtree.vistr_mut(),|a|{
            for c in a.iter_mut(){
                let a=unsafe{&mut *c.a};
                let b=unsafe{&mut *c.b};
                func(a,b,&mut c.d)
            }
        })
    }
}


                /*

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
    pub fn for_every_collision(&mut self,mut func:impl FnMut(&mut T,&mut T,&mut K)){
        for a in self.vec.iter_mut(){
            let (a,b)=a;
            let [c,d]=a.get_mut();
            (func)(c,d,b);
        }
    }
}
*/