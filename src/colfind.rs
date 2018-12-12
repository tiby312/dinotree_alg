//!
//! # User Guide
//!
//! Provides broadphase collision detection.
//!
//! There a multiple versions of the same fundamental query algorithm. There are parallel/sequential and 
//! advanced versions. 
//!
//! ```ignore
//! pub fn query_seq_mut<A:AxisTrait,T:HasAabb>(
//!             tree:&mut DinoTree<A,(),T>,
//!             func:impl FnMut(&mut T,&mut T));
//!
//! ```
//! The user supplies a reference to the tree, and a function to be called on every pair. The order in which
//! each pair is handled is not defined and has no meaning to the user.
//! 
//!
//! # Safety
//!
//! There is unsafe code to reuse code between the sequential and parallel versions.
//!
//! 
use inner_prelude::*;
use oned;
use node_handle::*;

///Naive algorithm.
pub fn query_naive_mut<T:HasAabb>(bots:&mut [T],mut func:impl FnMut(&mut T,&mut T)){
    tools::for_every_pair(bots,|a,b|{
        if a.get().get_intersect_rect(b.get()).is_some(){
            func(a,b);
        }
    });
}


///Sweep and prune algorithm.
pub fn query_sweep_mut<T:HasAabb>(axis:impl AxisTrait,bots:&mut [T],func:impl FnMut(&mut T,&mut T)){  
    ///Sorts the bots.
    fn sweeper_update<I:HasAabb,A:AxisTrait>(axis:A,collision_botids: &mut [I]) {

        let sclosure = |a: &I, b: &I| -> std::cmp::Ordering {
            let (p1,p2)=(a.get().get_range(axis).left,b.get().get_range(axis).left);
            if p1 > p2 {
                return std::cmp::Ordering::Greater;
            }
            std::cmp::Ordering::Less
        };

        collision_botids.sort_unstable_by(sclosure);
    }

    sweeper_update(axis,bots);


    struct Bl<T:HasAabb,F: FnMut(&mut T,&mut T)> {
        func: F,
        _p:PhantomData<T>
    }

    impl<T:HasAabb,F: FnMut(&mut T,&mut T)> ColMulti for Bl<T,F> {
        type T = T;
        fn collide(&mut self, a: &mut Self::T, b: &mut Self::T) {    
            (self.func)(a, b);
        }
       
    }

    let mut s=oned::Sweeper::new();
    s.find_2d(axis,bots,&mut Bl{func,_p:PhantomData});
}





fn go_down<
    A: AxisTrait, //this axis
    B: AxisTrait, //anchor axis
    X: HasAabb
>(
    this_axis: A,
    sweeper: &mut impl NodeHandler<T=X>,
    anchor: &mut DestructuredNode<X,B>,
    m: VistrMut<(),X>,
) {
    let anchor_axis=anchor.axis;
    let (nn,rest)=m.next();

    match rest{
        Some((extra,left,right))=>{
            let &FullComp{div,cont}=match extra{
                Some(d)=>d,
                None=>return
            };
            
            sweeper.handle_children((anchor_axis,&mut anchor.range,&anchor.cont),(this_axis,nn.range,Some(&cont)));
            
            //This can be evaluated at compile time!
            if this_axis.is_equal_to(anchor_axis) {
                if !(div < anchor.cont.left) {
                    self::go_down(this_axis.next(), sweeper, anchor, left);
                };
                if !(div > anchor.cont.right) {
                    self::go_down(this_axis.next(), sweeper, anchor, right);
                };
            } else {
                self::go_down(this_axis.next(), sweeper, anchor, left);
                self::go_down(this_axis.next(), sweeper, anchor,right);
            }
        },
        None=>{
            sweeper.handle_children((anchor_axis,&mut anchor.range,&anchor.cont),(this_axis,nn.range,None));
        }
    }
}







struct DestructuredNode<'a,T:HasAabb+'a,AnchorAxis:AxisTrait+'a>{
    cont:Range<T::Num>,
    _div:T::Num,
    range:&'a mut [T],
    axis:AnchorAxis
}

fn recurse<
    A: AxisTrait,
    JJ: par::Joiner,
    X: HasAabb + Send ,
    K:Splitter+Send,
    S:NodeHandler<T=X>+Splitter+Send+Sync
>(
    this_axis: A,
    par: JJ,
    sweeper:&mut S,
    m: LevelIter<VistrMut<(),X>>,
    splitter:&mut K
){


    sweeper.node_start();
    splitter.node_start();

    let((depth,nn),rest)=m.next();

    sweeper.handle_node(this_axis.next(),nn.range);
                
    match rest{
        Some((extra,mut left,mut right))=>{
            let &FullComp{div,cont}=match extra{
                Some(d)=>d,
                None=>{
                    sweeper.node_end();
                    splitter.node_end();
                    return;
                }
            };
            

            let mut nn=DestructuredNode{range:nn.range,cont,_div:div,axis:this_axis};
            {
                let left=left.inner.create_wrap_mut();
                let right=right.inner.create_wrap_mut();
                self::go_down(this_axis.next(), sweeper, &mut nn, left);
                self::go_down(this_axis.next(), sweeper, &mut nn, right);
            }

            let mut splitter2=splitter.div();
                
            let splitter={
                let splitter2=&mut splitter2;
                if !par.should_switch_to_sequential(depth) {
                    let mut sweeper2=sweeper.div();
                    
                    let (sweeper,splitter)={
                        let sweeper2=&mut sweeper2;
                        let af = move || {
                            self::recurse(this_axis.next(),par,sweeper,left,splitter);(sweeper,splitter)
                        };
                        let bf = move || {
                            self::recurse(this_axis.next(),par,sweeper2,right,splitter2)
                        };
                        rayon::join(af, bf).0
                    };
                    sweeper.add(sweeper2);
                    splitter
                } else {
                    self::recurse(this_axis.next(),par.into_seq(),sweeper,left,splitter);
                    self::recurse(this_axis.next(),par.into_seq(),sweeper,right,splitter2);
                    splitter
                }
            };

            splitter.add(splitter2);
        },
        None=>{
            sweeper.node_end();
            splitter.node_end();
        }
    }
}


///Used for the advanced algorithms.
///Trait that user implements to handling aabb collisions.
///The user supplies a struct that implements this trait instead of just a closure
///so that the user may also have the struct implement Splitter.
pub trait ColMulti{
    type T: HasAabb;
    fn collide(&mut self, a: &mut Self::T, b: &mut Self::T);
}






///Sequential
pub fn query_seq_mut<A:AxisTrait,T:HasAabb>(axis:A,vistr_mut:VistrMut<(),T>,func:impl FnMut(&mut T,&mut T)){
    struct Bo<T,F>(F,PhantomData<T>);
    impl<T:HasAabb,F:FnMut(&mut T,&mut T)> ColMulti for Bo<T,F>{
        type T=T;
        fn collide(&mut self,a:&mut T,b:&mut T){
            self.0(a,b);
        }   
    }
    impl<T,F> Splitter for Bo<T,F>{
        fn div(&mut self)->Self{
            unreachable!()
        }
        fn add(&mut self,_:Self){
            unreachable!()
        }
        fn node_start(&mut self){}
        fn node_end(&mut self){}
    }



    let b=Bo(func,PhantomData);
    
    let mut sweeper=HandleSorted::new(b);
    let mut splitter=SplitterEmpty;
    inner_query_seq_adv_mut(axis,vistr_mut,&mut splitter,&mut sweeper);
   
    //unimplemented!();
    //inner_query_seq_adv_mut(tree,b,SplitterEmpty,HandleSorted::<T,Bo<T,_>>::new());
}



///Parallel
pub fn query_mut<A:AxisTrait,T:HasAabb+Send>(axis:A,vistr_mut:VistrMut<(),T>,func:impl Fn(&mut T,&mut T)+Copy+Send){
    struct Bo<T,F>(F,PhantomData<T>);
    impl<T:HasAabb,F:Fn(&mut T,&mut T)> ColMulti for Bo<T,F>{
        type T=T;
        fn collide(&mut self,a:&mut T,b:&mut T){
            self.0(a,b);
        }   
    }
    impl<T,F:Copy> Splitter for Bo<T,F>{
        fn div(&mut self)->Self{
            Bo(self.0,PhantomData)
        }
        fn add(&mut self,_:Self){
            
        }
        fn node_start(&mut self){}
        fn node_end(&mut self){}
    }
    unsafe impl<T,F> Sync for Bo<T,F>{}
    let b=Bo(func,PhantomData);

    query_adv_mut(axis,vistr_mut,b,&mut SplitterEmpty,None);
}


///Parallel
pub fn query_nosort_mut<A:AxisTrait,T:HasAabb+Send>(tree:&mut NotSorted<A,(),T>,func:impl Fn(&mut T,&mut T)+Copy+Send){
    struct Bo<T,F>(F,PhantomData<T>);
    impl<T:HasAabb,F:Fn(&mut T,&mut T)> ColMulti for Bo<T,F>{
        type T=T;
        fn collide(&mut self,a:&mut T,b:&mut T){
            self.0(a,b);
        }   
    }
    impl<T,F:Copy> Splitter for Bo<T,F>{
        fn div(&mut self)->Self{
            Bo(self.0,PhantomData)
        }
        fn add(&mut self,_:Self){
            
        }
        fn node_start(&mut self){}
        fn node_end(&mut self){}
    }
    unsafe impl<T,F> Sync for Bo<T,F>{}
    let b=Bo(func,PhantomData);


    let mut sweeper=HandleNoSorted::new(b);

    inner_query_adv_mut(tree.0.axis(),tree.0.vistr_mut(),&mut SplitterEmpty,&mut sweeper,None);


}


///Advanced sequential version.
pub fn query_seq_adv_mut<A: AxisTrait,
    T: HasAabb,
    K:Splitter>(    
    axis:A,
    vistr_mut:VistrMut<(),T>,
    func:impl FnMut(&mut T,&mut T),
    splitter:&mut K
){
    struct Bo<T,F>(F,PhantomData<T>);
    impl<T:HasAabb,F:FnMut(&mut T,&mut T)> ColMulti for Bo<T,F>{
        type T=T;
        fn collide(&mut self,a:&mut T,b:&mut T){
            self.0(a,b);
        }   
    }
    impl<T,F> Splitter for Bo<T,F>{
        fn div(&mut self)->Self{
            unreachable!()
        }
        fn add(&mut self,_:Self){
            unreachable!()
        }
        fn node_start(&mut self){}
        fn node_end(&mut self){}
    }

    let b=Bo(func,PhantomData);
    
    let mut sweeper=HandleSorted::new(b);


    inner_query_seq_adv_mut(axis,vistr_mut,splitter,&mut sweeper);
}


///Advanced sequential version.
pub fn query_nosort_seq_mut<A: AxisTrait,
    T: HasAabb>(    
    tree: &mut NotSorted<A,(), T>,
    func:impl FnMut(&mut T,&mut T),
){
    struct Bo<T,F>(F,PhantomData<T>);
    impl<T:HasAabb,F:FnMut(&mut T,&mut T)> ColMulti for Bo<T,F>{
        type T=T;
        fn collide(&mut self,a:&mut T,b:&mut T){
            self.0(a,b);
        }   
    }
    impl<T,F> Splitter for Bo<T,F>{
        fn div(&mut self)->Self{
            unreachable!()
        }
        fn add(&mut self,_:Self){
            unreachable!()
        }
        fn node_start(&mut self){}
        fn node_end(&mut self){}
    }

    let b=Bo(func,PhantomData);
    
    let mut sweeper=HandleNoSorted::new(b);

    inner_query_seq_adv_mut(tree.0.axis(),tree.0.vistr_mut(),&mut SplitterEmpty,&mut sweeper);
   
}

///Advanced sequential version.
pub fn query_nosort_seq_adv_mut<A: AxisTrait,
    T: HasAabb,
    K:Splitter>(    
    tree: &mut NotSorted<A,(), T>,
    func:impl FnMut(&mut T,&mut T),
    splitter:&mut K
){
    struct Bo<T,F>(F,PhantomData<T>);
    impl<T:HasAabb,F:FnMut(&mut T,&mut T)> ColMulti for Bo<T,F>{
        type T=T;
        fn collide(&mut self,a:&mut T,b:&mut T){
            self.0(a,b);
        }   
    }
    impl<T,F> Splitter for Bo<T,F>{
        fn div(&mut self)->Self{
            unreachable!()
        }
        fn add(&mut self,_:Self){
            unreachable!()
        }
        fn node_start(&mut self){}
        fn node_end(&mut self){}
    }

    let b=Bo(func,PhantomData);
    
    let mut sweeper=HandleNoSorted::new(b);

    inner_query_seq_adv_mut(tree.0.axis(),tree.0.vistr_mut(),splitter,&mut sweeper);
    
}



///See query_adv_mut
fn inner_query_adv_mut<
    A: AxisTrait,
    T: HasAabb+Send,
    K:Splitter+Send,
    S: NodeHandler<T=T>+Splitter+Send+Sync>(  
    axis:A,
    vistr_mut:VistrMut<(),T>,
    splitter:&mut K,
    sweeper:&mut S, 
    height_switch_seq:Option<usize>
){
    let par=dinotree::advanced::compute_default_level_switch_sequential(height_switch_seq,vistr_mut.height());

    let dt = vistr_mut.with_depth(Depth(0));
    self::recurse(axis, par, sweeper, dt,splitter);
    
}
///See query_adv_mut
fn inner_query_seq_adv_mut<
    A: AxisTrait,
    T: HasAabb,
    K:Splitter,
    S: NodeHandler<T=T>+Splitter>(    
    //kdtree: &mut DinoTree<A,(), T>,
    axis:A,
    vistr_mut:VistrMut<(),T>,
    splitter:&mut K,
    sweeper:&mut S
){
  

    mod wrap{
        //Use this to get rid of Send trait constraint.
        #[repr(transparent)]
        pub struct Wrap<T:HasAabb>(T);
        unsafe impl<T:HasAabb> Send for Wrap<T>{}
        unsafe impl<T:HasAabb> Sync for Wrap<T>{}
        unsafe impl<T:HasAabb> HasAabb for Wrap<T>{
            type Num=T::Num;
            fn get(&self)->&Rect<Self::Num>{
                self.0.get()
            }
        }

        use super::*;
    
        #[repr(transparent)]
        pub struct SplitterWrapper<T>(
            pub T,
        );

        impl<T:Splitter> Splitter for SplitterWrapper<T>{
            fn div(&mut self)->Self{
                SplitterWrapper(self.0.div())
            }
            fn add(&mut self,a:Self){
                self.0.add(a.0)
            }
            fn node_start(&mut self){self.0.node_start()}
            fn node_end(&mut self){self.0.node_end()}
        }        
        unsafe impl<T> Send for SplitterWrapper<T>{}
        unsafe impl<T> Sync for SplitterWrapper<T>{}


        #[repr(transparent)]
        pub struct NodeHandlerWrapper<T>(pub T);
        

        impl<T:NodeHandler> NodeHandler for NodeHandlerWrapper<T>{
            type T=Wrap<T::T>;
            fn handle_node(&mut self,axis:impl AxisTrait,bots:&mut [Self::T])
            {
                let bots:&mut [T::T]=unsafe{std::mem::transmute(bots)};
                self.0.handle_node(axis,bots);
            }
            fn handle_children(&mut self,
                anchor:(impl AxisTrait,&mut [Self::T],&Range<<Self::T as HasAabb>::Num>),
                current:(impl AxisTrait,&mut [Self::T],Option<&Range<<Self::T as HasAabb>::Num>>)){
                let (a,b,c)=anchor;
                let (d,e,f)=current;

                let anchor:&mut [T::T]=unsafe{std::mem::transmute(b)};
                let current:&mut [T::T]=unsafe{std::mem::transmute(e)};

                self.0.handle_children((a,anchor,c),(d,current,f));
            }
        }
        impl<T:NodeHandler+Splitter> Splitter for NodeHandlerWrapper<T>{
            fn div(&mut self)->Self{
                NodeHandlerWrapper(self.0.div())
            }
            fn add(&mut self,a:Self){
                self.0.add(a.0)
            }
            fn node_start(&mut self){
                self.0.node_start();
            }
            fn node_end(&mut self){
                self.0.node_end();
            }
        }
        unsafe impl<T> Send for NodeHandlerWrapper<T>{}
        unsafe impl<T> Sync for NodeHandlerWrapper<T>{}
        


    }

    
    let splitter:&mut wrap::SplitterWrapper<K>=unsafe{std::mem::transmute(splitter)};//wrap::SplitterWrapper(splitter);
    //let kdtree:&mut DinoTree<A,(),wrap::Wrap<T>>=unsafe{std::mem::transmute(kdtree)};
    let vistr_mut:VistrMut<(),wrap::Wrap<T>>=unsafe{std::mem::transmute(vistr_mut)};
    let sweeper:&mut wrap::NodeHandlerWrapper<S>=unsafe{std::mem::transmute(sweeper)};//wrap::NodeHandlerWrapper(sweeper);

    let dt = vistr_mut.with_depth(Depth(0));
    //let mut sweeper = oned::Sweeper::new();
    
    
    self::recurse(axis, par::Sequential, sweeper, dt,splitter);
    
}


///The user has more control using this version of the query.
///The splitter will split and add at every level.
///The clos will split and add only at levels that are handled in parallel.
///This can be useful if the use wants to create a list of colliding pair indicies, but still wants paralleism.
pub fn query_adv_mut<
    A: AxisTrait,
    T: HasAabb+Send,
    F: ColMulti<T = T>+Splitter+Send+Sync,
    K: Splitter+Send,
    
>(
    axis:A,
    vistr_mut:VistrMut<(),T>,
    //kdtree: &mut DinoTree<A,(), T>,
    clos: F,
    splitter:&mut K,
    height_switch_seq:Option<usize>,
) -> F {

    let par=dinotree::advanced::compute_default_level_switch_sequential(height_switch_seq,vistr_mut.height());


    let dt = vistr_mut.with_depth(Depth(0));
    //let mut sweeper = oned::Sweeper::new();
    let mut sweeper=HandleSorted::new(clos);
    self::recurse(axis, par, &mut sweeper, dt,splitter);
    sweeper.func
}
