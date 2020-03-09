use crate::inner_prelude::*;
use compt::Visitor;

pub fn create_collision_list<'a,A:Axis,N:Node + Send+Sync,D>
                (tree:&mut DinoTree<A,N>,func:impl Fn(&mut <N::T as HasInner>::Inner,&mut <N::T as HasInner>::Inner)->Option<D>+Send+Sync)->CollisionList<N::T,D>
                where N::T:HasInner+Send+Sync{
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
    let mut nodes:Vec<Vec<Collision<_,D>>>=(0..compt::compute_num_nodes(height)).map(|_|Vec::new()).collect();
    let mtree=compt::dfs_order::CompleteTree::from_preorder_mut(&mut nodes).unwrap();
    
    tree.find_collisions_par_ext(|a|{
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

struct Collision<T,D>{
    a:*mut T,
    b:*mut T,
    d:D
}

impl<T,D> Collision<T,D>{
    #[inline(always)]
    fn new(a:&mut T,b:&mut T,d:D)->Self{
        Collision{a,b,d}
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

pub struct CollisionList<T:HasInner,D>{
    bot_ptr:*const [T],
    nodes:Vec<Vec<Collision<T::Inner,D>>>
}

impl<T:HasInner,D> CollisionList<T,D>{

    pub fn for_every_pair_mut(&mut self,bot_ptr:&mut [T],mut func:impl FnMut(&mut T::Inner,&mut T::Inner,&mut D)){
        assert_eq!(self.bot_ptr,bot_ptr as *const _);
        for n in self.nodes.iter_mut(){
            for c in n.iter_mut(){
                func(unsafe{&mut *c.a},unsafe{&mut *c.b},&mut c.d)
            }
        }
    }
}
impl<T:HasInner+Send+Sync,D:Send+Sync> CollisionList<T,D>{
    
    pub fn for_every_pair_par_mut(&mut self,bot_ptr:&mut[T],func:impl Fn(&mut T::Inner,&mut T::Inner,&mut D)+Send+Sync+Copy){
        assert_eq!(self.bot_ptr,bot_ptr as *const _);
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


           