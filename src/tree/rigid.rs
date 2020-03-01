use crate::inner_prelude::*;
use compt::Visitor;

pub fn create_collision_list<'a,A:Axis,N:Node + Send+Sync,D>
                (tree:&mut DinoTree<A,N>,func:impl Fn(&mut <N::T as HasInner>::Inner,&mut <N::T as HasInner>::Inner)->Option<D>+Send+Sync)->CollisionList<N::T,D>
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

    //TODO might break if user uses custom height
    let height=1+par::compute_level_switch_sequential(par::SWITCH_SEQUENTIAL_DEFAULT,tree.get_height()).get_depth_to_switch_at();
    //dbg!(tree.get_height(),height);
    let mut nodes:Vec<Vec<Collision<N::T,D>>>=(0..compt::compute_num_nodes(height)).map(|_|Vec::new()).collect();
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

    CollisionList{bot_ptr:tree.bot_ptr,nodes}
}

struct Collision<T:HasInner,D>{
    a:*mut T::Inner,
    b:*mut T::Inner,
    d:D
}
impl<T:HasInner,D> Collision<T,D>{
    #[inline(always)]
    fn new(a:&mut T::Inner,b:&mut T::Inner,d:D)->Self{
        Collision{a:a as *mut _,b:b as *mut _,d}
    }
}
unsafe impl<T:HasInner,D> Send for Collision<T,D>{}
unsafe impl<T:HasInner,D> Sync for Collision<T,D>{}

fn parallelize<T:Visitor+Send+Sync>(a:T,func:impl Fn(T::Item)+Sync+Send+Copy) where T::Item:Send+Sync{
    let (n,l)=a.next();
    func(n);
    if let Some([left,right])=l{
        rayon::join(||parallelize(left,func),||parallelize(right,func));
    }
}

pub struct CollisionList<T:HasInner,D>{
    bot_ptr:*const [T],
    nodes:Vec<Vec<Collision<T,D>>>
}
impl<T:HasInner+Send+Sync,D:Send+Sync> CollisionList<T,D>{
    /* //TODO implmement
    pub fn for_every_pair_mut(&mut self,mut func:impl FnMut(&mut T,&mut T,&mut D)+Send+Sync+Copy){
        for a in self.nodes.iter_mut(){
            for c in a.iter_mut(){
                let a=unsafe{&mut *c.a};
                let b=unsafe{&mut *c.b};
                func(a,b,&mut c.d)
            }
        }
    }
    */
    pub fn for_every_pair_par_mut(&mut self,bots:&mut [T],func:impl Fn(&mut T::Inner,&mut T::Inner,&mut D)+Send+Sync+Copy){
        assert_eq!(bots as *const _,self.bot_ptr);
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


           