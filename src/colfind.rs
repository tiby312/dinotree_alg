use inner_prelude::*;
use oned::Bleek;
use compt::LevelIter;
use compt::WrapGen;
use std::cell::UnsafeCell;
use dinotree_inner::par::Joiner;


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

pub trait ColMulti: Send + Sync + Sized {
    type T: SweepTrait;
    fn collide(&mut self, a: ColSingle<Self::T>, b: ColSingle<Self::T>);
    fn div(self)->(Self,Self);
    fn add(self,b:Self)->Self;
}

pub struct ColMultiWrapper<'a, C: ColMulti + 'a>(pub &'a mut C);

impl<'a, C: ColMulti + 'a> Bleek for ColMultiWrapper<'a, C> {
    type T = C::T;
    fn collide(&mut self, a: ColSingle<Self::T>, b: ColSingle<Self::T>) {
        self.0.collide(a, b);
    }
}






mod anchor{
    use super::*;


    pub struct DestructuredAnchor<'a,T:SweepTrait+'a,AnchorAxis:AxisTrait+'a>{
        cont:&'a Range<T::Num>,
        _div:&'a T::Num,
        range:&'a mut [T],
        _p:PhantomData<AnchorAxis>
    }
    pub enum ErrEnum{
        NoBots,
        NoChildrenOrBots
    }
    impl<'a,T:SweepTrait+'a,AnchorAxis:AxisTrait+'a> DestructuredAnchor<'a,T,AnchorAxis>{

        pub fn get(&mut self)->(&Range<T::Num>,&mut [T]){
            (self.cont,self.range)
        }
        pub fn new(nd:&'a mut NodeDyn<T>)->Result<DestructuredAnchor<'a,T,AnchorAxis>,ErrEnum>{
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
    C: CTreeIterator<Item = &'x mut NodeDyn<X>> + Send,
    X: SweepTrait + 'x,
    F: ColMulti<T = X>
>(
    this_axis: A,
    parent_axis: B,
    sweeper: &mut Sweeper<F::T>,
    anchor: &mut anchor::DestructuredAnchor<X,B>,
    m: WrapGen<LevelIter<C>>,
    func: &mut F
) {
    {
        let (mut bo, rest) = m.next();
        let &mut (_, ref mut nn) = bo.get_mut();

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
                        self::go_down(this_axis.next(), parent_axis, sweeper, anchor, left, func);
                    };
                    if !(div > anchor.get().0.end) {
                        self::go_down(this_axis.next(), parent_axis, sweeper, anchor, right, func);
                    };
                } else {
                    self::go_down(this_axis.next(), parent_axis, sweeper, anchor, left, func);
                    self::go_down(this_axis.next(), parent_axis, sweeper, anchor,right, func);
                }
               
            }
            _ => {
                self::for_every_bijective_pair::<A, B, _,_>(nn, anchor, sweeper, ColMultiWrapper(func),IsLeaf);
            }
        };
    }
}

fn recurse<
    'x,
    A: AxisTrait,
    JJ: par::Joiner,
    X: SweepTrait + 'x,
    F: ColMulti<T = X>,
    C: CTreeIterator<Item = &'x mut NodeDyn<X>> + Send,
    K: TreeTimerTrait
>(
    this_axis: A,
    par: JJ,
    sweeper: &mut Sweeper<F::T>,
    m: LevelIter<C>,
    mut clos: F,
    mut timer_log: K
) -> (F,K::Bag) {
    timer_log.start();

    let ((level, nn), rest) = m.next();

    let k = match rest {
        None => {
            self::sweeper_find_2d::<A::Next, _>(sweeper, &mut nn.range, ColMultiWrapper(&mut clos));

            (clos,timer_log.leaf_finish())
        },
        Some((mut left, mut right)) => {

            match anchor::DestructuredAnchor::<X,A>::new(nn){
                Ok(mut nn)=>{
                    self::sweeper_find_2d::<A::Next, _>(sweeper, nn.get().1, ColMultiWrapper(&mut clos));

                    let left = compt::WrapGen::new(&mut left);
                    let right = compt::WrapGen::new(&mut right);

                    self::go_down(this_axis.next(), this_axis, sweeper, &mut nn, left, &mut clos);
                    self::go_down(this_axis.next(), this_axis, sweeper, &mut nn, right, &mut clos);
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
                        
                    )
                };
                let bf = || {
                    let mut sweeper = Sweeper::new();
                    self::recurse(
                        this_axis.next(),
                        par,
                        &mut sweeper,
                        right,
                        bb,
                        tb,
                        
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
                    
                );
                let (clos,tb) = self::recurse(
                    this_axis.next(),
                    par.into_seq(),
                    sweeper,
                    right,
                    clos,
                    tb,
                    
                );

                (clos,ta, tb)
            };

            let b=K::combine(ta, tb);
            (clos,b)
        }
    };

    k
}


pub fn for_every_col_pair_seq<
    A: AxisTrait,
    T: SweepTrait,
    F: FnMut(ColSingle<T>, ColSingle<T>),
    K: TreeTimerTrait,
>(
    kdtree: &mut DynTree<A, T>,
    mut clos: F,
) -> (F,K::Bag) {

    mod wrap{
        use super::*;
        pub struct Wrapper<'a, T: SweepTrait, F: FnMut(ColSingle<T>, ColSingle<T>) + 'a>(
            pub UnsafeCell<&'a mut F>,
            pub PhantomData<T>,
        );

        impl<'a, T: SweepTrait, F: FnMut(ColSingle<T>, ColSingle<T>) + 'a> Clone for Wrapper<'a, T, F> {
            fn clone(&self) -> Wrapper<'a, T, F> {
                unreachable!()
            }
        }

        impl<'a, T: SweepTrait, F: FnMut(ColSingle<T>, ColSingle<T>) + 'a> ColMulti for Wrapper<'a, T, F> {
            type T = T;

            fn collide(&mut self, a: ColSingle<Self::T>, b: ColSingle<Self::T>) {
                //Protected by the fact that cloning thus struct
                //results in panic!.
                let k = unsafe { &mut *self.0.get() };
                k(a, b);
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
        unsafe impl<'a, T: SweepTrait, F: FnMut(ColSingle<T>, ColSingle<T>) + 'a> Send
            for Wrapper<'a, T, F>
        {
        }
        unsafe impl<'a, T: SweepTrait, F: FnMut(ColSingle<T>, ColSingle<T>) + 'a> Sync
            for Wrapper<'a, T, F>
        {
        }
    }

    let (_,bag)={
        let wrapper =wrap::Wrapper(UnsafeCell::new(&mut clos), PhantomData);


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

pub fn for_every_col_pair<
    A: AxisTrait,
    T: SweepTrait,
    F: ColMulti<T = T>,
    K: TreeTimerTrait,
>(
    kdtree: &mut DynTree<A, T>,
    clos: F,
) -> (F,K::Bag) {

    let height=kdtree.get_height();
    
    const DEPTH_SEQ:usize=4;

    let gg=if height<=DEPTH_SEQ{
        0
    }else{
        height-DEPTH_SEQ
    };
    

    self::for_every_col_pair_inner::<_, _, _, _, K>(
        A::new(),
        par::Parallel::new(Depth(gg)),
        kdtree,
        clos,
    )
}

fn for_every_col_pair_inner<
    A: AxisTrait,
    JJ: par::Joiner,
    T: SweepTrait,
    F: ColMulti<T = T>,
    K: TreeTimerTrait,
>(
    this_axis: A,
    par: JJ,
    kdtree: &mut DynTree<A, T>,
    clos: F,
) -> (F,K::Bag) {
    let height = kdtree.get_height();
    let level = kdtree.get_level_desc();
    let dt = kdtree.get_iter_mut();
    //let dt = compt::LevelIter::new(dt, level);
    let dt=dt.with_depth();
    let mut sweeper = Sweeper::new();

    let h = K::new(height);
    let bag = self::recurse(this_axis, par, &mut sweeper, dt, clos, h);
    bag
}



fn for_every_bijective_pair<A: AxisTrait, B: AxisTrait, F: Bleek,L:LeafTracker>(
    this: &mut NodeDyn<F::T>,
    parent: &mut anchor::DestructuredAnchor<F::T,B>,
    sweeper: &mut Sweeper<F::T>,
    mut func: F,
    leaf_tracker:L
) {
    //Can be evaluated at compile time
    if A::get() != B::get() {

        let (parent_box,parent_bots)=parent.get();

        let r1 = Sweeper::get_section::<B>(&mut this.range, parent_box);

        let r2=if !leaf_tracker.is_leaf(){
            let this_box=this.cont.unwrap();
    
            Sweeper::get_section::<A>(parent_bots, &this_box)
        }else{
            parent_bots
        };

        for inda in r1.iter_mut() {
            let (rect_a, aval) = inda.get_mut();
            for indb in r2.iter_mut() {
                let (rect_b, bval) = indb.get_mut();
                if rect_a.0.intersects_rect(&rect_b.0) {
                    let a = ColSingle {
                        rect: rect_a,
                        inner: aval,
                    };
                    let b = ColSingle {
                        rect: rect_b,
                        inner: bval,
                    };
                    func.collide(a, b);
                }
            }
        }
    } else {
        self::sweeper_find_parallel_2d::<A::Next, _>(
            sweeper,
            &mut this.range,
            parent.get().1,
            func,
        );
    }
}


use colfind::bl::sweeper_find_2d;
use colfind::bl::sweeper_find_parallel_2d;
mod bl {
    use super::*;
    use std::marker::PhantomData;
    struct Bl<A: AxisTrait, F: Bleek> {
        a: F,
        _p: PhantomData<A>,
    }

    impl<A: AxisTrait, F: Bleek> Bleek for Bl<A, F> {
        type T = F::T;

        fn collide(&mut self, a: ColSingle<Self::T>, b: ColSingle<Self::T>) {
            //only check if the opoosite axis intersects.
            //already know they intersect
            let a2 = A::Next::get();
            if (a.rect)
                .0
                .get_range(a2)
                .intersects((b.rect).0.get_range(a2))
            {
                self.a.collide(a, b);
            }
        }
    }

    //Bots a sorted along the axis.
    pub fn sweeper_find_2d<A: AxisTrait, F: Bleek>(
        sweeper: &mut Sweeper<F::T>,
        bots: &mut [F::T],
        clos2: F,
    ) {
        let b: Bl<A, _> = Bl {
            a: clos2,
            _p: PhantomData,
        };
        sweeper.find::<A, _>(bots, b);
    }
    pub fn sweeper_find_parallel_2d<A: AxisTrait, F: Bleek>(
        sweeper: &mut Sweeper<F::T>,
        bots1: &mut [F::T],
        bots2: &mut [F::T],
        clos2: F,
    ) {
        let b: Bl<A, _> = Bl {
            a: clos2,
            _p: PhantomData,
        };

        sweeper.find_bijective_parallel::<A, _>((bots1, bots2), b);
    }
}
