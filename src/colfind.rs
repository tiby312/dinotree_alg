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
//!             tree:&mut DynTree<A,(),T>,
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
use compt::timer::TreeTimer2;
use compt::timer::TreeTimeResultIterator;

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
    m: NdIterMut<(),X>,
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
    m: LevelIter<NdIterMut<(),X>>,
    mut clos: F,
    splitter:K
) -> (F,K) {
    
    let((depth,nn),rest)=m.next();

    match rest{
        Some((extra,mut left,mut right))=>{
            let &FullComp{div,cont}=match extra{
                Some(d)=>d,
                None=>return (clos,splitter) //TODO is this okay?
            };
            

            let mut nn=DestructuredNode{range:&mut nn.range,cont,_div:div,axis:this_axis};
            {
                sweeper.find_2d(this_axis.next(),&mut nn.range, ColMultiWrapper(&mut clos));

                let left=left.inner.create_wrap_mut();
                let right=right.inner.create_wrap_mut();

                self::go_down(this_axis.next(), sweeper, &mut nn, left, &mut clos,);
                self::go_down(this_axis.next(), sweeper, &mut nn, right, &mut clos);
            }

            if !par.should_switch_to_sequential(depth) {
                let (splitter1,splitter2)=splitter.div(IsParallel::Parallel);
                let (clos1,clos2)=clos.div(IsParallel::Parallel);
                let af = || {
                    self::recurse(this_axis.next(),par,sweeper,left,clos1,splitter1)
                };
                let bf = || {
                    let mut sweeper = oned::Sweeper::new();
                    self::recurse(this_axis.next(),par,&mut sweeper,right,clos2,splitter2)
                };
                let ((clos1,splitter1), (clos2,splitter2)) = rayon::join(af, bf);
                let clos=clos1.add(clos2,IsParallel::Parallel);
                let splitter=splitter1.add(splitter2,IsParallel::Parallel);
                (clos,splitter)
            } else {
                let (splitter1,splitter2)=splitter.div(IsParallel::Sequential);
                let (clos,splitter1) = self::recurse(this_axis.next(),par.into_seq(),sweeper,left,clos,splitter1);
                let (clos,splitter2) = self::recurse(this_axis.next(),par.into_seq(),sweeper,right,clos,splitter2);
                (clos,splitter1.add(splitter2,IsParallel::Sequential))
            }
        },
        None=>{
            sweeper.find_2d(this_axis.next(),&mut nn.range, ColMultiWrapper(&mut clos));
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
    pub fn query<A:AxisTrait,T:HasAabb>(_tree:&DynTree<A,(),T>,mut _func:impl FnMut(&T,&T)){
        unimplemented!("Versions that do not borrow the tree mutable are implemented.  Waiting for parametric mutability.")
    }
    #[allow(dead_code)]
    pub fn query_par<A:AxisTrait,T:HasAabb+Send>(_tree:&DynTree<A,(),T>,_func:impl Fn(&T,&T)+Copy+Send){
        unimplemented!("Versions that do not borrow the tree mutable are implemented.  Waiting for parametric mutability.")
    }
}

/*
///Debug Sequential
pub fn query_debug_seq_mut<A:AxisTrait,T:HasAabb>(tree:&mut DynTree<A,(),T>,func:impl FnMut(&mut T,&mut T))->TreeTimeResultIterator{
    let height=tree.get_height();
    query_seq_mut_inner(tree,func,TreeTimer2::new(height)).into_iter()
}

///Sequential
pub fn query_seq_mut<A:AxisTrait,T:HasAabb>(tree:&mut DynTree<A,(),T>,func:impl FnMut(&mut T,&mut T)){
    let _ = query_seq_mut_inner(tree,func,TreeTimerEmpty);
}
*/

/*
fn query_seq_mut_inner<A:AxisTrait,T:HasAabb,F:FnMut(&mut T,&mut T)>(tree:&mut DynTree<A,(),T>,mut func:F,h:K)->K::Bag{

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
        pub struct Wrapper<'a, T: HasAabb, F: FnMut(&mut T, &mut T) + 'a>(
            pub &'a mut F,
            pub PhantomData<T>,
        );

        impl<'a, T: HasAabb, F: FnMut(&mut T, &mut T) + 'a> Clone for Wrapper<'a, T, F> {
            fn clone(&self) -> Wrapper<'a, T, F> {
                unreachable!()
            }
        }

        impl<'a, T: HasAabb, F: FnMut(&mut T, &mut T) + 'a> self::ColMulti for Wrapper<'a, T, F> {
            type T = Wrap<T>;

            fn collide(&mut self, a: &mut Wrap<T>, b: &mut Wrap<T>) {
                self.0(&mut a.0,&mut b.0);
            }
        }

        //Unsafely implement send and Sync
        //Safe to do since our algorithms first clone this struct before
        //passing it to another thread. This sadly has to be indiviually
        //verified.
        unsafe impl<'a, T: HasAabb, F: FnMut(&mut T, &mut T) + 'a> Send
            for Wrapper<'a, T, F>
        {
        }
        unsafe impl<'a, T: HasAabb, F: FnMut(&mut T, &mut T) + 'a> Sync
            for Wrapper<'a, T, F>
        {
        }
    }

    let wrap=wrap::Wrapper(&mut func,PhantomData);

    let tree:&mut DynTree<A,(),wrap::Wrap<T>>=unsafe{std::mem::transmute(tree)};
    self::query_par_adv_mut(
        par::Sequential,
        tree,
        wrap,
    ).1
    
}
*/
/*
///Debug Parallel
pub fn query_debug_mut<A:AxisTrait,T:HasAabb+Send>(tree:&mut DynTree<A,(),T>,func:impl Fn(&mut T,&mut T)+Copy+Send)->TreeTimeResultIterator{
    
    let c1=move |_:&mut (),a:&mut T,b:&mut T|{
        func(a,b);
    };

    let c2=|_:()|((),());
    let c3=|_:(),_:()|();

    let clos = self::closure_struct::ColMultiStruct{aa
        :(),a:c1,f2:c2,f3:c3,_p:PhantomData};



    const DEPTH_SEQ:usize=4;

    let height=tree.get_height();
    let gg=if height<=DEPTH_SEQ{
        Depth(0)
    }else{
        Depth(height-DEPTH_SEQ)
    };

    self::query_par_adv_mut(
        par::Parallel::new(gg),
        tree,
        TreeTimer2::new(height),
        clos,
    ).1.into_iter()
}
*/




///Sequential
pub fn query_seq_mut<A:AxisTrait,T:HasAabb>(tree:&mut DynTree<A,(),T>,func:impl FnMut(&mut T,&mut T)){
    struct Bo<T,F>(F,PhantomData<T>);
    impl<T:HasAabb,F:FnMut(&mut T,&mut T)> ColMulti for Bo<T,F>{
        type T=T;
        fn collide(&mut self,a:&mut T,b:&mut T){
            self.0(a,b);
        }   
    }
    impl<T,F> Splitter for Bo<T,F>{
        fn div(self,a:IsParallel)->(Self,Self){
            unreachable!()
        }
        fn add(self,a:Self,b:IsParallel)->Self{
            unreachable!()
        }
    }

    let b=Bo(func,PhantomData);

    query_seq_adv_mut(tree,b,SplitterEmpty);
}
pub fn query_mut<A:AxisTrait,T:HasAabb+Send>(tree:&mut DynTree<A,(),T>,func:impl Fn(&mut T,&mut T)+Copy+Send){
    struct Bo<T,F>(F,PhantomData<T>);
    impl<T:HasAabb,F:Fn(&mut T,&mut T)> ColMulti for Bo<T,F>{
        type T=T;
        fn collide(&mut self,a:&mut T,b:&mut T){
            self.0(a,b);
        }   
    }
    impl<T,F:Copy> Splitter for Bo<T,F>{
        fn div(self,a:IsParallel)->(Self,Self){
            let b=Bo(self.0,PhantomData);
            (self,b)
        }
        fn add(self,a:Self,b:IsParallel)->Self{
            self
        }
    }

    let b=Bo(func,PhantomData);

    query_seq_adv_mut(tree,b,SplitterEmpty);
}


pub fn query_seq_adv_mut<
    A: AxisTrait,
    T: HasAabb,
    F: ColMulti<T = T>,
    K:Splitter>(    
    kdtree: &mut DynTree<A,(), T>,
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
            fn div(self,p:IsParallel)->(Self,Self){
                unreachable!()
            }
            fn add(self,a:Self,p:IsParallel)->Self{
                unreachable!()
            }
        }

        //Unsafely implement send and Sync
        //Safe to do since our algorithms first clone this struct before
        //passing it to another thread. This sadly has to be indiviually
        //verified.
        unsafe impl<T, F> Send for Wrapper<T, F>{}
        unsafe impl<T, F> Sync for Wrapper<T, F>{}
    }


    let clos=wrap::Wrapper(clos,PhantomData);
    let splitter:wrap::Wrapper<T,K>=wrap::Wrapper(splitter,PhantomData);
    let kdtree:&mut DynTree<A,(),wrap::Wrap<T>>=unsafe{std::mem::transmute(kdtree)};

    let this_axis=kdtree.get_axis();
    let dt = kdtree.get_iter_mut().with_depth(Depth(0));
    let mut sweeper = oned::Sweeper::new();

    let (a,b)=self::recurse(this_axis, par::Sequential, &mut sweeper, dt, clos,splitter);
    (a.0,b.0)
}

///The user has more control using this version of the query.
///It also returns time information.
pub fn query_par_adv_mut<
    A: AxisTrait,
    T: HasAabb+Send,
    F: ColMulti<T = T>+Splitter+Send,
    K: Splitter+Send
>(
    kdtree: &mut DynTree<A,(), T>,
    clos: F,
    splitter:K
) -> (F,K) {
    let par={
        const DEPTH_SEQ:usize=4;

        let height=kdtree.get_height();
        let gg=if height<=DEPTH_SEQ{
            Depth(0)
        }else{
            Depth(height-DEPTH_SEQ)
        };
        par::Parallel::new(gg)
    };

    let this_axis=kdtree.get_axis();
    let dt = kdtree.get_iter_mut().with_depth(Depth(0));
    let mut sweeper = oned::Sweeper::new();

    self::recurse(this_axis, par, &mut sweeper, dt, clos,splitter)
}
