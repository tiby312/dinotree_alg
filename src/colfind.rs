use inner_prelude::*;
use oned;
use dinotree_inner::par::Joiner;

///Naive version.
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




use self::anchor::DestructuredNode;
mod anchor{
    use super::*;
    pub struct DestructuredNode<'a,T:HasAabb+'a,AnchorAxis:AxisTrait+'a>{
        pub cont:&'a Range<T::Num>,
        pub div:&'a T::Num,
        pub range:&'a mut [T],
        _p:PhantomData<AnchorAxis>
    }
    pub enum ErrEnum{
        NoBots,
        NoChildrenOrBots
    }
    impl<'a,T:HasAabb+'a,AnchorAxis:AxisTrait+'a> DestructuredNode<'a,T,AnchorAxis>{

        pub fn new(nd:&'a mut NodeDyn<(),T>)->Result<DestructuredNode<'a,T,AnchorAxis>,ErrEnum>{
            let cont=match &nd.cont{
                &Some(ref x)=>{x},
                &None=>return Err(ErrEnum::NoBots)
            };
            let div=match &nd.div{
                &Some(ref x)=>{x},
                &None=>return Err(ErrEnum::NoChildrenOrBots)
            };
            
            let range=&mut nd.range;
            Ok(DestructuredNode{_p:PhantomData,cont,div:div,range})
        }
    }
}



fn go_down<
    A: AxisTrait, //this axis
    B: AxisTrait, //anchor axis
    X: HasAabb ,
    F: ColMulti<T = X>
>(
    this_axis: A,
    anchor_axis: B,
    sweeper: &mut oned::Sweeper<X>,
    anchor: &mut DestructuredNode<X,B>,
    m: NdIterMut<(),X>,
    func: &mut F,
    depth:Depth
) {

    let (nn,rest) = m.next();


    {
        let func=ColMultiWrapper(func);
        if !this_axis.is_equal_to(anchor_axis) {

            let (anchor_box,anchor_bots)=(anchor.cont,&mut anchor.range);

            let r1 = oned::get_section_mut(anchor_axis,&mut nn.range, anchor_box);

            let r2=if rest.is_some(){

                //This node could possible not have bots in it.
                match &nn.cont{
                    Some(cont)=>{
                        oned::get_section_mut(this_axis,anchor_bots, cont)       
                    },
                    None=>{
                        anchor_bots
                    }
                }
            }else{
                anchor_bots
            };

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

    
    match rest {
        Some((left, right)) => {
            let div=match nn.div{
                Some(div)=>div,
                None=>return
            };
                    
            //This can be evaluated at compile time!
            if this_axis.is_equal_to(anchor_axis) {
                if !(div < anchor.cont.left) {
                    self::go_down(this_axis.next(), anchor_axis, sweeper, anchor, left, func,depth.next_down());
                };
                if !(div > anchor.cont.right) {
                    self::go_down(this_axis.next(), anchor_axis, sweeper, anchor, right, func,depth.next_down());
                };
            } else {
                self::go_down(this_axis.next(), anchor_axis, sweeper, anchor, left, func,depth.next_down());
                self::go_down(this_axis.next(), anchor_axis, sweeper, anchor,right, func,depth.next_down());
            }
        }
        _ => {}
    }
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
    m: NdIterMut<(),X>,
    mut clos: F,
    mut timer_log: K,
    level:Depth
) -> (F,K::Bag) {
    timer_log.start();

    let (nn, rest) = m.next();

    let k = match rest {
        None => {
            sweeper.find_2d(this_axis.next(),&mut nn.range, ColMultiWrapper(&mut clos));

            (clos,timer_log.leaf_finish())
        },
        Some((mut left, mut right)) => {

            match anchor::DestructuredNode::<X,A>::new(nn){
                Ok(mut nn)=>{
                    sweeper.find_2d(this_axis.next(),nn.range, ColMultiWrapper(&mut clos));

                    let left=left.create_wrap_mut();
                    let right=right.create_wrap_mut();

                    self::go_down(this_axis.next(), this_axis, sweeper, &mut nn, left, &mut clos,level.next_down());
                    self::go_down(this_axis.next(), this_axis, sweeper, &mut nn, right, &mut clos,level.next_down());
                },
                Err(e)=>{
                    match e{
                        anchor::ErrEnum::NoBots=>{
                            //Do nothing. Dont need to check against self, or children
                        },
                        anchor::ErrEnum::NoChildrenOrBots=>{
                            //Dont even need to recurse futher down.
                            return (clos,timer_log.leaf_finish())
                        }
                    }
                }
            }


            let (ta, tb) = timer_log.next();

            let (clos,ta, tb) = if !par.should_switch_to_sequential(level) {
                let (mut aa,mut bb)=clos.div();

                let af = || {
                    self::recurse(
                        this_axis.next(),
                        par,
                        sweeper,
                        left,
                        aa,
                        ta,
                        level.next_down()
                        
                    )
                };
                let bf = || {
                    let mut sweeper = oned::Sweeper::new();
                    self::recurse(
                        this_axis.next(),
                        par,
                        &mut sweeper,
                        right,
                        bb,
                        tb,
                        level.next_down()
                        
                    )
                };
                let (ta, tb) = rayon::join(af, bf);

                let a=ta.0.add(tb.0);
                (a,ta.1, tb.1)
            } else {
                let (clos,ta) = self::recurse(
                    this_axis.next(),
                    par.into_seq(),
                    sweeper,
                    left,
                    clos,
                    ta,
                    level.next_down()
                    
                );
                let (clos,tb) = self::recurse(
                    this_axis.next(),
                    par.into_seq(),
                    sweeper,
                    right,
                    clos,
                    tb,
                    level.next_down()
                    
                );

                (clos,ta, tb)
            };

            let b=K::combine(ta, tb);
            (clos,b)
        }
    };

    k
}







pub trait ColMulti:Sized {
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

pub fn query_mut<A:AxisTrait,T:HasAabb>(tree:&mut DynTree<A,(),T>,mut func:impl FnMut(&mut T,&mut T)){

    mod wrap{
        //Use this to get rid of Send trait constraint.
        #[repr(transparent)]
        pub struct Wrap<T:HasAabb>(T);
        unsafe impl<T:HasAabb> Send for Wrap<T>{}
        unsafe impl<T:HasAabb> Sync for Wrap<T>{}
        impl<T:HasAabb> HasAabb for Wrap<T>{
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
    self::query_par_adv_mut::<_,_, _, _, TreeTimerEmpty>(
        par::Sequential,
        tree,
        wrap,
    );
    
}


pub fn query_par_mut<A:AxisTrait,T:HasAabb+Send>(tree:&mut DynTree<A,(),T>,func:impl Fn(&mut T,&mut T)+Copy+Send){

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

    self::query_par_adv_mut::<_,_, _, _, TreeTimerEmpty>(
        par::Parallel::new(gg),
        tree,
        clos,
    );        
}

///The user has more control using this version of the query.
///It also returns time information.
pub fn query_par_adv_mut<
    A: AxisTrait,
    JJ: par::Joiner,
    T: HasAabb+Send,
    F: ColMulti<T = T>+Send,
    K: TreeTimerTrait,
>(
    par: JJ,
    kdtree: &mut DynTree<A,(), T>,
    clos: F,
) -> (F,K::Bag) {
    let this_axis=kdtree.get_axis();
    let height = kdtree.get_height();
    let dt = kdtree.get_iter_mut();
    let mut sweeper = oned::Sweeper::new();

    let h = K::new(height);
    let bag = self::recurse(this_axis, par, &mut sweeper, dt, clos, h,Depth(0));
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

