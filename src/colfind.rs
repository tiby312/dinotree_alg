//!
//! An mulithreaded hybrid kdtree/mark and sweep algorithm used for broadphase detection.
//!
//!
//! There a multiple versions of the same fundamental query algorithm. There are parallel/sequential and 
//! debug/non debug versions. 
//!
//! ```
//! pub fn query_seq_mut<A:AxisTrait,T:HasAabb>(tree:&mut DynTree<A,(),T>,func:impl FnMut(&mut T,&mut T));
//!
//! ```
//!
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
        fn div(self)->(Self,Self){
            unreachable!();
        }
        fn add(self,_:Self)->Self{
            unreachable!();
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
    F: ColMulti<T = X>+Send,
    K: TreeTimerTrait
>(
    this_axis: A,
    par: JJ,
    sweeper:&mut oned::Sweeper<F::T>,
    m: LevelIter<NdIterMut<(),X>>,
    mut clos: F,
    mut timer_log: K
) -> (F,K::Bag) {
    timer_log.start();

    let((depth,nn),rest)=m.next();

    match rest{
        Some((extra,mut left,mut right))=>{
            let &FullComp{div,cont}=match extra{
                Some(d)=>d,
                None=>return (clos,timer_log.leaf_finish())
            };
            

            let mut nn=DestructuredNode{range:&mut nn.range,cont,_div:div,axis:this_axis};
            {
                sweeper.find_2d(this_axis.next(),&mut nn.range, ColMultiWrapper(&mut clos));

                let left=left.inner.create_wrap_mut();
                let right=right.inner.create_wrap_mut();

                self::go_down(this_axis.next(), sweeper, &mut nn, left, &mut clos,);
                self::go_down(this_axis.next(), sweeper, &mut nn, right, &mut clos);
            }
            let (ta, tb) = timer_log.next();

            let (clos,ta, tb) = if !par.should_switch_to_sequential(depth) {
                let (mut aa,mut bb)=clos.div();

                let af = || {
                    self::recurse(this_axis.next(),par,sweeper,left,aa,ta)
                };
                let bf = || {
                    let mut sweeper = oned::Sweeper::new();
                    self::recurse(this_axis.next(),par,&mut sweeper,right,bb,tb)
                };
                let (ta, tb) = rayon::join(af, bf);

                let a=ta.0.add(tb.0);
                (a,ta.1, tb.1)
            } else {
                let (clos,ta) = self::recurse(this_axis.next(),par.into_seq(),sweeper,left,clos,ta,);
                let (clos,tb) = self::recurse(this_axis.next(),par.into_seq(),sweeper,right,clos,tb,);

                (clos,ta, tb)
            };

            let b=K::combine(ta, tb);
            (clos,b)

        },
        None=>{
            sweeper.find_2d(this_axis.next(),&mut nn.range, ColMultiWrapper(&mut clos));
            (clos,timer_log.leaf_finish())
        }
    }
}


pub(crate) trait ColMulti:Sized {
    type T: HasAabb;
    fn collide(&mut self, a: &mut Self::T, b: &mut Self::T);
    fn div(self)->(Self,Self);
    fn add(self,b:Self)->Self;
}

struct ColMultiWrapper<'a, C: ColMulti + 'a>(pub &'a mut C);

impl<'a, C: ColMulti + 'a> ColMulti for ColMultiWrapper<'a, C> {
    type T = C::T;
    fn collide(&mut self, a:&mut Self::T, b: &mut Self::T) {
        self.0.collide(a, b);
    }
    fn div(self)->(Self,Self){
        unreachable!();
    }
    fn add(self,_:Self)->Self{
        unreachable!();
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


///Debug Sequential
pub fn query_debug_seq_mut<A:AxisTrait,T:HasAabb>(tree:&mut DynTree<A,(),T>,func:impl FnMut(&mut T,&mut T))->TreeTimeResultIterator{
    let height=tree.get_height();
    query_seq_mut_inner(tree,func,TreeTimer2::new(height)).into_iter()
}

///Sequential
pub fn query_seq_mut<A:AxisTrait,T:HasAabb>(tree:&mut DynTree<A,(),T>,func:impl FnMut(&mut T,&mut T)){
    let _ = query_seq_mut_inner(tree,func,TreeTimerEmpty);
}

fn query_seq_mut_inner<A:AxisTrait,T:HasAabb,F:FnMut(&mut T,&mut T),K:TreeTimerTrait>(tree:&mut DynTree<A,(),T>,mut func:F,h:K)->K::Bag{

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
            fn div(self)->(Self,Self){
                unreachable!();
            }
            fn add(self,_b:Self)->Self{
                unreachable!();
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
        h,
        wrap,
    ).1
    
}

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

///Parallel
pub fn query_mut<A:AxisTrait,T:HasAabb+Send>(tree:&mut DynTree<A,(),T>,func:impl Fn(&mut T,&mut T)+Copy+Send){

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
        TreeTimerEmpty,
        clos,
    );        
}

///The user has more control using this version of the query.
///It also returns time information.
fn query_par_adv_mut<
    A: AxisTrait,
    JJ: par::Joiner,
    T: HasAabb+Send,
    F: ColMulti<T = T>+Send,
    K: TreeTimerTrait,
>(
    par: JJ,
    kdtree: &mut DynTree<A,(), T>,
    h:K,
    clos: F,
) -> (F,K::Bag) {
    let this_axis=kdtree.get_axis();
    let dt = kdtree.get_iter_mut().with_depth(Depth(0));
    let mut sweeper = oned::Sweeper::new();

    let bag = self::recurse(this_axis, par, &mut sweeper, dt, clos, h);
    bag
}


mod closure_struct {
    use super::*;

    pub struct ColMultiStruct<
        A:Send,
        T: HasAabb,
        F: Fn(&mut A,&mut T, &mut T) + Send + Copy ,
        F2:Fn(A)->(A,A)+Copy,
        F3:Fn(A,A)->A+Copy
    > {
        pub a: F,
        pub f2: F2,
        pub f3: F3,
        pub aa:A,
        pub _p: PhantomData<(T)>,
    }


    impl<
        A:Send+Sync,
        T: HasAabb,
        F: Fn(&mut A,&mut T, &mut T) + Send + Copy,
        F2:Fn(A)->(A,A)+Copy,
        F3:Fn(A,A)->A+Copy
    > ColMulti for ColMultiStruct<A,T,  F,F2,F3>
    {
        type T = T;
    
        fn collide(&mut self,a: &mut T, b: &mut T) {
            (self.a)(&mut self.aa,a,b);
        }
        fn div(self)->(Self,Self){
            let (aa1,aa2)=(self.f2)(self.aa);
            
            let c1=ColMultiStruct{a:self.a,f2:self.f2,f3:self.f3,aa:aa1,_p:PhantomData};
            let c2=ColMultiStruct{a:self.a,f2:self.f2,f3:self.f3,aa:aa2,_p:PhantomData};
            (c1,c2)
        }
        fn add(self,b:Self)->Self{

            let aa_n=(self.f3)(self.aa,b.aa);
            
            ColMultiStruct{a:self.a,f2:self.f2,f3:self.f3,aa:aa_n,_p:PhantomData}
        }
    }
}

