use inner_prelude::*;
use oned;
use std::cell::UnsafeCell;
use dinotree_inner::par::Joiner;
use dinotree_inner::*;

trait LeafTracker{
    fn is_leaf(&self)->bool;
}
struct IsLeaf;
struct IsNotLeaf;
impl LeafTracker for IsNotLeaf{
    fn is_leaf(&self)->bool{
        false
    }
}
impl LeafTracker for IsLeaf{
    fn is_leaf(&self)->bool{
        true
    }
}





pub trait ColMultiConst:Sized {
    type T: HasAabb;
    fn collide(&mut self, a: &Self::T, b: &Self::T);
    fn div(self)->(Self,Self);
    fn add(self,b:Self)->Self;
}

pub trait ColMulti:Sized {
    type T: HasAabb;
    fn collide(&mut self, a: &mut Self::T, b: &mut Self::T);
    fn div(self)->(Self,Self);
    fn add(self,b:Self)->Self;
}

struct ColMultiWrapper<'a, C: ColMulti + 'a>(pub &'a mut C);

impl<'a, C: ColMulti + 'a> oned::mod_mut::Bleek for ColMultiWrapper<'a, C> {
    type T = C::T;
    fn collide(&mut self, a:&mut Self::T, b: &mut Self::T) {
        self.0.collide(a, b);
    }
}






mod anchor{
    use super::*;

    pub struct DestructuredAnchor<'a,T:HasAabb+'a,AnchorAxis:AxisTrait+'a>{
        cont:&'a Range<T::Num>,
        _div:&'a T::Num,
        range:&'a mut [T],
        _p:PhantomData<AnchorAxis>
    }
    pub enum ErrEnum{
        NoBots,
        NoChildrenOrBots
    }
    impl<'a,T:HasAabb+'a,AnchorAxis:AxisTrait+'a> DestructuredAnchor<'a,T,AnchorAxis>{

        pub fn get(&mut self)->(&Range<T::Num>,&mut [T]){
            (self.cont,self.range)
        }
        pub fn new(nd:&'a mut NodeDyn<(),T>)->Result<DestructuredAnchor<'a,T,AnchorAxis>,ErrEnum>{
            let cont=match &nd.cont{
                &Some(ref x)=>{x},
                &None=>return Err(ErrEnum::NoBots)
            };
            let div=match &nd.div{
                &Some(ref x)=>{x},
                &None=>return Err(ErrEnum::NoChildrenOrBots)
            };
            
            let range=&mut nd.range;
            Ok(DestructuredAnchor{_p:PhantomData,cont,_div:div,range})
        }
    }
}





fn go_down<
    'x,
    A: AxisTrait, //this axis
    B: AxisTrait, //parent axis
    X: HasAabb + 'x,
    F: ColMulti<T = X>
>(
    this_axis: A,
    parent_axis: B,
    sweeper: &mut oned::mod_mut::Sweeper<F::T>,
    anchor: &mut anchor::DestructuredAnchor<X,B>,
    m: NdIterMut<(),X>,
    func: &mut F,
    depth:Depth
) {
    {
        let (nn,rest) = m.next();

        match rest {
            Some((left, right)) => {

                let div=match nn.div{
                    Some(div)=>div,
                    None=>return
                };

                self::for_every_bijective_pair::<A, B, _,_>(nn, anchor, sweeper, ColMultiWrapper(func),IsNotLeaf);
        
                
                //This can be evaluated at compile time!
                if B::get() == A::get() {
                    if !(div < anchor.get().0.start) {
                        self::go_down(this_axis.next(), parent_axis, sweeper, anchor, left, func,depth.next_down());
                    };
                    if !(div > anchor.get().0.end) {
                        self::go_down(this_axis.next(), parent_axis, sweeper, anchor, right, func,depth.next_down());
                    };
                } else {
                    self::go_down(this_axis.next(), parent_axis, sweeper, anchor, left, func,depth.next_down());
                    self::go_down(this_axis.next(), parent_axis, sweeper, anchor,right, func,depth.next_down());
                }
               
            }
            _ => {
                self::for_every_bijective_pair::<A, B, _,_>(nn, anchor, sweeper, ColMultiWrapper(func),IsLeaf);
            }
        };
    }
}

fn recurse<
    A: AxisTrait,
    JJ: par::Joiner,
    X: HasAabb + Send,
    F: ColMulti<T = X>+Send,
    K: TreeTimerTrait
>(
    this_axis: A,
    par: JJ,
    sweeper: &mut oned::mod_mut::Sweeper<F::T>,
    m: NdIterMut<(),X>,
    mut clos: F,
    mut timer_log: K,
    level:Depth
) -> (F,K::Bag) {
    timer_log.start();

    let (nn, rest) = m.next();

    let k = match rest {
        None => {
            sweeper.find_2d::<A::Next, _>(&mut nn.range, ColMultiWrapper(&mut clos));

            (clos,timer_log.leaf_finish())
        },
        Some((mut left, mut right)) => {

            match anchor::DestructuredAnchor::<X,A>::new(nn){
                Ok(mut nn)=>{
                    sweeper.find_2d::<A::Next, _>(nn.get().1, ColMultiWrapper(&mut clos));

                    //let left = compt::WrapGen::new(&mut left);
                    //let right = compt::WrapGen::new(&mut right);
                    //left.create_wrap();
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
                    let mut sweeper = oned::mod_mut::Sweeper::new();
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



/*
pub fn for_every_col_pair_seq_mut<
    A: AxisTrait,
    T: HasAabb+Send,
    F: FnMut(ColSingle<T>, ColSingle<T>),
    K: TreeTimerTrait,
>(
    kdtree: &mut DynTreeMut<A,(), T>,
    mut clos: F,
) -> (F,K::Bag) {

    mod wrap{
        use super::*;
        pub struct Wrapper<'a, T: HasAabb, F: FnMut(ColSingle<T>, ColSingle<T>) + 'a>(
            pub &'a mut F,
            pub PhantomData<T>,
        );

        impl<'a, T: HasAabb, F: FnMut(ColSingle<T>, ColSingle<T>) + 'a> Clone for Wrapper<'a, T, F> {
            fn clone(&self) -> Wrapper<'a, T, F> {
                unreachable!()
            }
        }

        impl<'a, T: HasAabb, F: FnMut(ColSingle<T>, ColSingle<T>) + 'a> ColMulti for Wrapper<'a, T, F> {
            type T = T;

            fn collide(&mut self, a: ColSingle<Self::T>, b: ColSingle<Self::T>) {
                self.0(a,b);
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
        unsafe impl<'a, T: HasAabb, F: FnMut(ColSingle<T>, ColSingle<T>) + 'a> Send
            for Wrapper<'a, T, F>
        {
        }
        unsafe impl<'a, T: HasAabb, F: FnMut(ColSingle<T>, ColSingle<T>) + 'a> Sync
            for Wrapper<'a, T, F>
        {
        }
    }

    let (_,bag)={
        let wrapper =wrap::Wrapper(&mut clos, PhantomData);


        //All of the above is okay because we start with SEQUENTIAL
        self::for_every_col_pair_inner::<_, _, _, _, K>(
            A::new(),
            par::Sequential::new(Depth(0)),
            kdtree,
            wrapper,
        )
    };
    (clos,bag)
}
*/




pub fn for_every_col_pair<
    'a,
    JJ: par::Joiner,
    A: AxisTrait,
    T: HasAabb+Send,
    F: ColMultiConst<T = T>+Send,
    K: TreeTimerTrait,
>(
    joiner:JJ,
    kdtree: &DynTree<'a,A,(), T>,
    clos: F,
    )->(F,K::Bag){
    unimplemented!();
}



pub fn for_every_col_pair_mut<
    JJ: par::Joiner,
    A: AxisTrait,
    T: HasAabb+Send,
    F: ColMulti<T = T>+Send,
    K: TreeTimerTrait,
>(
    joiner:JJ,
    kdtree: &mut DynTreeMut<A,(), T>,
    clos: F,
) -> (F,K::Bag) {

    let height=kdtree.get_height();
    
    /*
    const DEPTH_SEQ:usize=4;

    let gg=if height<=DEPTH_SEQ{
        0
    }else{
        height-DEPTH_SEQ
    };
    */
    

    self::for_every_col_pair_inner::<_, _, _, _, K>(
        A::new(),
        //par::Parallel::new(Depth(gg)),
        joiner,
        kdtree,
        clos,
    )
}

fn for_every_col_pair_inner<
    A: AxisTrait,
    JJ: par::Joiner,
    T: HasAabb+Send,
    F: ColMulti<T = T>+Send,
    K: TreeTimerTrait,
>(
    this_axis: A,
    par: JJ,
    kdtree: &mut DynTreeMut<A,(), T>,
    clos: F,
) -> (F,K::Bag) {
    let height = kdtree.get_height();
    //let level = kdtree.get_level_desc();
    let dt = kdtree.get_iter_mut();
    //let dt = compt::LevelIter::new(dt, level);
    //let dt=dt.with_depth(Depth(0));
    let mut sweeper = oned::mod_mut::Sweeper::new();

    let h = K::new(height);
    let bag = self::recurse(this_axis, par, &mut sweeper, dt, clos, h,Depth(0));
    bag
}


macro_rules! get_mut_slice{
    ($range:expr)=>{{
        &mut $range
    }}
}

macro_rules! colfind{
    ($sweeper:ty,$node:ty,$get_slice:ident)=>{

        fn for_every_bijective_pair<A: AxisTrait, B: AxisTrait, F: oned::mod_mut::Bleek,L:LeafTracker>(
            this: &mut NodeDyn<(),F::T>,
            parent: &mut anchor::DestructuredAnchor<F::T,B>,
            sweeper: &mut oned::mod_mut::Sweeper<F::T>,
            mut func: F,
            leaf_tracker:L
        ) {
            //Can be evaluated at compile time
            if A::get() != B::get() {

                let (parent_box,parent_bots)=parent.get();

                let r1 = sweeper.get_section::<B>($get_slice!(this.range), parent_box);

                let r2=if !leaf_tracker.is_leaf(){
                    let this_box=this.cont.unwrap();
            
                    sweeper.get_section::<A>(parent_bots, &this_box)
                }else{
                    parent_bots
                };

                sweeper.find_perp_2d(r1,r2,func);

            } else {
                sweeper.find_parallel_2d::<A::Next, _>(
                    $get_slice!(this.range),
                    parent.get().1,
                    func,
                );
            }
        }

    }
}

colfind!(oned::mod_mut::Sweeper<F::T>,&NodeDyn<(),F::T>,get_mut_slice);