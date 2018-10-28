//!
//! # User Guide
//!
//! Provides broadphase collision detection.
//!
//! There a multiple versions of the same fundamental query algorithm. There are parallel/sequential and 
//! debug/non debug versions. 
//!
//! ```
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
    s.find_2d(axis,bots,Bl{func,_p:PhantomData});
}




fn go_down<
    A: AxisTrait, //this axis
    B: AxisTrait, //anchor axis
    X: HasAabb ,
    F: ColMulti<T = X>
>(
    this_axis: A,
    sweeper: &mut oned::Sweeper<X>,
    anchor: &mut DestructuredNode<X,B>,
    m: VistrMut<(),X>,
    func: &mut F,
) {
    let anchor_axis=anchor.axis;
    let (nn,rest)=m.next();
    match rest{
        Some((extra,left,right))=>{
            let &FullComp{div,cont}=match extra{
                Some(d)=>d,
                None=>return
            };


            
            {
                let func=ColMultiWrapper(func);
                if !this_axis.is_equal_to(anchor_axis) {

                    let (anchor_box,anchor_bots)=(&anchor.cont,&mut anchor.range);

                    let r1 = oned::get_section_mut(anchor_axis,&mut nn.range, anchor_box);

                    let r2= oned::get_section_mut(this_axis,anchor_bots,&cont);     

                    sweeper.find_perp_2d(r1,r2,func);

                } else {
                    if cont.intersects(&anchor.cont){
                        sweeper.find_parallel_2d(
                            this_axis.next(),
                            &mut nn.range,
                            anchor.range,
                            func,
                        );
                    }
                }
            }
                                      
            

            //This can be evaluated at compile time!
            if this_axis.is_equal_to(anchor_axis) {
                if !(div < anchor.cont.left) {
                    self::go_down(this_axis.next(), sweeper, anchor, left, func);
                };
                if !(div > anchor.cont.right) {
                    self::go_down(this_axis.next(), sweeper, anchor, right, func);
                };
            } else {
                self::go_down(this_axis.next(), sweeper, anchor, left, func);
                self::go_down(this_axis.next(), sweeper, anchor,right, func);
            }
        },
        None=>{
            let func=ColMultiWrapper(func);
            if !this_axis.is_equal_to(anchor_axis) {

                let (anchor_box,anchor_bots)=(&anchor.cont,&mut anchor.range);

                let r1 =oned::get_section_mut(anchor_axis,&mut nn.range, anchor_box);
                let r2= anchor_bots;

                sweeper.find_perp_2d(r1,r2,func);

            } else {
                sweeper.find_parallel_2d(
                    this_axis.next(),
                    &mut nn.range,
                    anchor.range,
                    func,
                );
            }
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
    F: ColMulti<T = X>+Splitter+Send,
    K:Splitter+Send
>(
    this_axis: A,
    par: JJ,
    sweeper:&mut oned::Sweeper<F::T>,
    m: LevelIter<VistrMut<(),X>>,
    mut clos: F,
    mut splitter:K
) -> (F,K) {


    clos.node_start();
    splitter.node_start();

    let((depth,nn),rest)=m.next();

    //std::thread::sleep(std::time::Duration::from_millis(100));
    match rest{
        Some((extra,mut left,mut right))=>{
            let &FullComp{div,cont}=match extra{
                Some(d)=>d,
                None=>{
                    clos.node_end();
                    splitter.node_end();
                    return (clos,splitter)
                } //TODO is this okay?
            };
            

            let mut nn=DestructuredNode{range:&mut nn.range,cont,_div:div,axis:this_axis};
            {
                sweeper.find_2d(this_axis.next(),&mut nn.range, ColMultiWrapper(&mut clos));

                let left=left.inner.create_wrap_mut();
                let right=right.inner.create_wrap_mut();

                self::go_down(this_axis.next(), sweeper, &mut nn, left, &mut clos,);
                self::go_down(this_axis.next(), sweeper, &mut nn, right, &mut clos);
            }

            let (splitter1,splitter2)=splitter.div();
                
            let (clos,splitter1,splitter2)=if !par.should_switch_to_sequential(depth) {
                let (clos1,clos2)=clos.div();
                let af = || {
                    self::recurse(this_axis.next(),par,sweeper,left,clos1,splitter1)
                };
                let bf = || {
                    let mut sweeper = oned::Sweeper::new();
                    self::recurse(this_axis.next(),par,&mut sweeper,right,clos2,splitter2)
                };
                let ((clos1,splitter1), (clos2,splitter2)) = rayon::join(af, bf);
                let clos=clos1.add(clos2);
                (clos,splitter1,splitter2)
            } else {
                let (clos,splitter1) = self::recurse(this_axis.next(),par.into_seq(),sweeper,left,clos,splitter1);
                let (clos,splitter2) = self::recurse(this_axis.next(),par.into_seq(),sweeper,right,clos,splitter2);
                (clos,splitter1,splitter2)
            };

            (clos,splitter1.add(splitter2))
        },
        None=>{
            sweeper.find_2d(this_axis.next(),&mut nn.range, ColMultiWrapper(&mut clos));
            clos.node_end();
            splitter.node_end();
            (clos,splitter) //TODO is this okay?
        }
    }
}


pub trait ColMulti:Sized {
    type T: HasAabb;
    fn collide(&mut self, a: &mut Self::T, b: &mut Self::T);
}

struct ColMultiWrapper<'a, C: ColMulti + 'a>(pub &'a mut C);

impl<'a, C: ColMulti + 'a> ColMulti for ColMultiWrapper<'a, C> {
    type T = C::T;
    fn collide(&mut self, a:&mut Self::T, b: &mut Self::T) {
        self.0.collide(a, b);
    }
}


//TODO implement
mod todo{
    use super::*;
    #[allow(dead_code)]
    pub fn query<A:AxisTrait,T:HasAabb>(_tree:&DinoTree<A,(),T>,mut _func:impl FnMut(&T,&T)){
        unimplemented!("Versions that do not borrow the tree mutable are implemented.  Waiting for parametric mutability.")
    }
    #[allow(dead_code)]
    pub fn query_par<A:AxisTrait,T:HasAabb+Send>(_tree:&DinoTree<A,(),T>,_func:impl Fn(&T,&T)+Copy+Send){
        unimplemented!("Versions that do not borrow the tree mutable are implemented.  Waiting for parametric mutability.")
    }
}


///Sequential
pub fn query_seq_mut<A:AxisTrait,T:HasAabb>(tree:&mut DinoTree<A,(),T>,func:impl FnMut(&mut T,&mut T)){
    struct Bo<T,F>(F,PhantomData<T>);
    impl<T:HasAabb,F:FnMut(&mut T,&mut T)> ColMulti for Bo<T,F>{
        type T=T;
        fn collide(&mut self,a:&mut T,b:&mut T){
            self.0(a,b);
        }   
    }
    impl<T,F> Splitter for Bo<T,F>{
        fn div(self)->(Self,Self){
            unreachable!()
        }
        fn add(self,_:Self)->Self{
            unreachable!()
        }
        fn node_start(&mut self){}
        fn node_end(&mut self){}
    }

    let b=Bo(func,PhantomData);

    query_seq_adv_mut(tree,b,SplitterEmpty);
}


 const DEPTH_SEQ:usize=2;

///Parallel
pub fn query_mut<A:AxisTrait,T:HasAabb+Send>(tree:&mut DinoTree<A,(),T>,func:impl Fn(&mut T,&mut T)+Copy+Send){
    struct Bo<T,F>(F,PhantomData<T>);
    impl<T:HasAabb,F:Fn(&mut T,&mut T)> ColMulti for Bo<T,F>{
        type T=T;
        fn collide(&mut self,a:&mut T,b:&mut T){
            self.0(a,b);
        }   
    }
    impl<T,F:Copy> Splitter for Bo<T,F>{
        fn div(self)->(Self,Self){
            let b=Bo(self.0,PhantomData);
            (self,b)
        }
        fn add(self,_:Self)->Self{
            self
        }
        fn node_start(&mut self){}
        fn node_end(&mut self){}
    }

    let b=Bo(func,PhantomData);

    query_adv_mut(tree,b,SplitterEmpty,DEPTH_SEQ);
}


///See query_adv_mut
pub fn query_seq_adv_mut<
    A: AxisTrait,
    T: HasAabb,
    F: ColMulti<T = T>,
    K:Splitter>(    
    kdtree: &mut DinoTree<A,(), T>,
    clos: F,
    splitter:K
)->(F,K){
  

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
        pub struct Wrapper<T, F>(
            pub F,
            pub PhantomData<T>,
        );

        impl<T: HasAabb, F: ColMulti<T=T>> self::ColMulti for Wrapper<T, F> {
            type T = Wrap<T>;
            fn collide(&mut self, a: &mut Wrap<T>, b: &mut Wrap<T>) {
                self.0.collide(&mut a.0,&mut b.0);
            }

       
        }
        impl<T,F> Splitter for Wrapper<T,F>{
            fn div(self)->(Self,Self){
                unreachable!()
            }
            fn add(self,_:Self)->Self{
                unreachable!()
            }
            fn node_start(&mut self){}
            fn node_end(&mut self){}
        }

        //Unsafely implement send and Sync
        //Safe to do since our algorithms first clone this struct before
        //passing it to another thread. This sadly has to be indiviually
        //verified.
        unsafe impl<T, F> Send for Wrapper<T, F>{}
        unsafe impl<T, F> Sync for Wrapper<T, F>{}

        pub struct SplitterWrapper<T>(
            pub T,
        );

        impl<T:Splitter> Splitter for SplitterWrapper<T>{
            fn div(self)->(Self,Self){
                let (a,b)=self.0.div();
                (SplitterWrapper(a),SplitterWrapper(b))
            }
            fn add(self,a:Self)->Self{
                let a=self.0.add(a.0);
                SplitterWrapper(a)
            }
            fn node_start(&mut self){self.0.node_start()}
            fn node_end(&mut self){self.0.node_end()}
        }        
        unsafe impl<T> Send for SplitterWrapper<T>{}
        unsafe impl<T> Sync for SplitterWrapper<T>{}

    }


    let clos=wrap::Wrapper(clos,PhantomData);
    let splitter=wrap::SplitterWrapper(splitter);
    let kdtree:&mut DinoTree<A,(),wrap::Wrap<T>>=unsafe{std::mem::transmute(kdtree)};

    let this_axis=kdtree.axis();
    let dt = kdtree.vistr_mut().with_depth(Depth(0));
    let mut sweeper = oned::Sweeper::new();

    let (a,b)=self::recurse(this_axis, par::Sequential, &mut sweeper, dt, clos,splitter);
    (a.0,b.0)
}

///The user has more control using this version of the query.
///The splitter will split and add at every level.
///The clos will split and add only at levels that are handled in parallel.
///This can be useful if the use wants to create a list of colliding pair indicies, but still want paralleism.
pub fn query_adv_mut<
    A: AxisTrait,
    T: HasAabb+Send,
    F: ColMulti<T = T>+Splitter+Send,
    K: Splitter+Send
>(
    kdtree: &mut DinoTree<A,(), T>,
    clos: F,
    splitter:K,
    height_switch_seq:usize
) -> (F,K) {
    let par={
       
        let height=kdtree.height();
        let gg=if height<=height_switch_seq{
            Depth(0)
        }else{
            Depth(height-height_switch_seq)
        };
        par::Parallel::new(gg)
    };

    let this_axis=kdtree.axis();
    let dt = kdtree.vistr_mut().with_depth(Depth(0));
    let mut sweeper = oned::Sweeper::new();

    self::recurse(this_axis, par, &mut sweeper, dt, clos,splitter)
}
