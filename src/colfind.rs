use inner_prelude::*;
use oned::Bleek;
use compt::LevelIter;
use compt::WrapGen;
use std::cell::UnsafeCell;
use dinotree_inner::par::Joiner;

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

fn go_down<
    'x,
    A: AxisTrait, //this axis
    B: AxisTrait, //parent axis
    C: CTreeIterator<Item = &'x mut NodeDyn<X>> + Send,
    X: SweepTrait + 'x,
    F: ColMulti<T = X>,
>(
    this_axis: A,
    parent_axis: B,
    sweeper: &mut Sweeper<F::T>,
    anchor: &mut &mut NodeDyn<X>,
    m: WrapGen<LevelIter<C>>,
    func: &mut F,
) {
    {
        let (mut bo, rest) = m.next();
        let &mut (_, ref mut nn) = bo.get_mut();

        self::for_every_bijective_pair::<A, B, _>(nn, anchor, sweeper, ColMultiWrapper(func));

        match rest {
            Some((left, right)) => {
                let div = nn.divider;

                //This can be evaluated at compile time!
                if B::get() == A::get() {
                    if !(div < anchor.container_box.start) {
                        self::go_down(this_axis.next(), parent_axis, sweeper, anchor, left, func);
                    };
                    if !(div > anchor.container_box.end) {
                        self::go_down(this_axis.next(), parent_axis, sweeper, anchor, right, func);
                    };
                } else {
                    self::go_down(this_axis.next(), parent_axis, sweeper, anchor, left, func);
                    self::go_down(this_axis.next(), parent_axis, sweeper, anchor, right, func);
                }
            }
            _ => {}
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
    K: TreeTimerTrait,
>(
    this_axis: A,
    par: JJ,
    sweeper: &mut Sweeper<F::T>,
    m: LevelIter<C>,
    mut clos: F,
    mut timer_log: K,
) -> (F,K::Bag) {
    timer_log.start();

    let ((level, mut nn), rest) = m.next();

    self::sweeper_find_2d::<A::Next, _>(sweeper, &mut nn.range, ColMultiWrapper(&mut clos));

    let k = match rest {
        None => (clos,timer_log.leaf_finish()),
        Some((mut left, mut right)) => {
            {
                let left = compt::WrapGen::new(&mut left);
                let right = compt::WrapGen::new(&mut right);

                self::go_down(this_axis.next(), this_axis, sweeper, &mut nn, left, &mut clos);
                self::go_down(this_axis.next(), this_axis, sweeper, &mut nn, right, &mut clos);
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


pub fn k_nearest<
    A:AxisTrait,
    T:SweepTrait,
    F: FnMut(ColSingle<T>),
    MF:Fn((T::Num,T::Num),&AABBox<T::Num>)->T::Num,
    MF2:Fn(T::Num,T::Num)->T::Num,
    >(tree:&mut DynTree<A,T>,point:(T::Num,T::Num),num:usize,mut func:F,mf:MF,mf2:MF2){

    let height = tree.get_height();
    let dt = tree.get_iter_mut();

    let mut c=ClosestCand::new(num);
    recc(A::new(),dt,&mf,&mf2,point,&mut c);
 
    for i in c.a{
        let j=unsafe{&mut *i.0}.get_mut();
        func(ColSingle{inner:j.1,rect:j.0});
    }


    struct ClosestCand<T:SweepTrait>{
        a:Vec<(*mut T,T::Num)>,
        num:usize
    }
    impl<T:SweepTrait> ClosestCand<T>{
        fn new(num:usize)->ClosestCand<T>{
            let a=Vec::with_capacity(num);
            ClosestCand{a,num}
        }

        fn consider(&mut self,a:(&mut T,T::Num)){
            let a=(a.0 as *mut T,a.1);

            if self.a.len()<self.num{
                println!("added");
                self.a.push(a);
                //TODO inefficient?
                self.a.sort_unstable_by(|a,b|a.1.cmp(&b.1));
            }else{
                if a.1<self.a[self.num-1].1{
                    self.a.push(a);
                    //TODO inefficient?
                    self.a.sort_unstable_by(|a,b|a.1.cmp(&b.1));
                    self.a.pop();
                }
            }
        }
        fn full_and_max_distance(&self)->Option<T::Num>{
            match self.a.get(self.num-1){
                Some(x)=>
                {
                    Some(x.1)
                },
                None=>{
                    None
                }
            }
        }
    }
    
    fn recc<'x,'a,
        A: AxisTrait,
        T: SweepTrait + 'x,
        C: CTreeIterator<Item = &'x mut NodeDyn<T>>,
        MF:Fn((T::Num,T::Num),&AABBox<T::Num>)->T::Num,
        MF2:Fn(T::Num,T::Num)->T::Num,
        >(axis:A,stuff:C,mf:&MF,mf2:&MF2,point:(T::Num,T::Num),res:&mut ClosestCand<T>){

        let (nn,rest)=stuff.next();

        //known at compile time.
        let pp=if axis.is_xaxis(){
            point.0
        }else{
            point.1
        };

        let div = nn.divider;
        
        match rest {
            Some((left, right)) => {


                let (first,other)=if (pp<div) {
                    (left,right)
                }else{
                    (right,left)
                };

                recc(axis.next(), first,mf,mf2,point,res);
               
                let traverse_other=match res.full_and_max_distance(){
                    Some(max)=>{
                        if mf2(pp,div)<max{
                            true
                        }else{
                            false
                        }
                    },
                    None=>{
                        true
                    }
                };

                if traverse_other{
                    recc(axis.next(),other,mf,mf2,point,res);
                }
            }
            _ => {
                
            }
        }

        let traverse_other=match res.full_and_max_distance(){
            Some(max)=>{
                if mf2(pp,div)<max{
                    true
                }else{
                    false
                }
            },
            None=>{
                true
            }
        };

        if traverse_other{
            for i in nn.range.iter_mut(){            
                let dis_sqr=mf(point,i.get().0);
                res.consider((i,dis_sqr));
            }
        }
    }
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


    //#[derive(Copy,Clone)]
    pub struct Wrapper<'a, T: SweepTrait, F: FnMut(ColSingle<T>, ColSingle<T>) + 'a>(
        UnsafeCell<&'a mut F>,
        PhantomData<T>,
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
        fn add(self,b:Self)->Self{
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

    let (_,bag)={
        let wrapper = Wrapper(UnsafeCell::new(&mut clos), PhantomData);


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
    println!("height={:?}",height);
    //TODO this value really should be able to be set by the user right?
    //highly dependant on the algorithm 
    const a:usize=6;

    let gg=if height<=a{
        0
    }else{
        height-a
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
    mut clos: F,
) -> (F,K::Bag) {
    let height = kdtree.get_height();
    let level = kdtree.get_level_desc();
    let dt = kdtree.get_iter_mut();
    let dt = compt::LevelIter::new(dt, level);
    let mut sweeper = Sweeper::new();

    let h = K::new(height);
    let bag = self::recurse(this_axis, par, &mut sweeper, dt, clos, h);
    bag
}

fn for_every_bijective_pair<A: AxisTrait, B: AxisTrait, F: Bleek>(
    this: &mut NodeDyn<F::T>,
    parent: &mut &mut NodeDyn<F::T>,
    sweeper: &mut Sweeper<F::T>,
    mut func: F,
) {
    //Evaluated at compile time
    if A::get() != B::get() {
        let r1 = Sweeper::get_section::<B>(&mut this.range, &parent.container_box);
        let r2 = Sweeper::get_section::<A>(&mut parent.range, &this.container_box);

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
            &mut parent.range,
            func,
        );
    }
}

fn rect_recurse<
    'x,
    A: AxisTrait,
    T: SweepTrait + 'x,
    C: CTreeIterator<Item = &'x mut NodeDyn<T>>,
    F: FnMut(ColSingle<T>),
>(
    this_axis: A,
    m: C,
    rect: &Rect<T::Num>,
    func: &mut F,
) {
    let (nn, rest) = m.next();
    {
        let sl = Sweeper::get_section::<A::Next>(&mut nn.range, rect.get_range2::<A::Next>());

        for i in sl {
            let a = i.get_mut();
            let a = ColSingle {
                rect: a.0,
                inner: a.1,
            };

            func(a);
        }
    }
    match rest {
        Some((left, right)) => {
            let div = nn.divider;

            let rr = rect.get_range2::<A>();

            if !(div < rr.start) {
                self::rect_recurse(this_axis.next(), left, rect, func);
            }
            if !(div > rr.end) {
                self::rect_recurse(this_axis.next(), right, rect, func);
            }
        }
        _ => {}
    }
}

pub fn for_all_intersect_rect<A: AxisTrait, T: SweepTrait, F: FnMut(ColSingle<T>)>(
    tree: &mut DynTree<A, T>,
    rect: &Rect<T::Num>,
    mut closure: F,
) {
    let mut f = |a: ColSingle<T>| {
        if rect.intersects_rect(&(a.rect).0) {
            closure(a);
        }
    };

    let ta = tree.get_iter_mut();
    self::rect_recurse(A::new(), ta, rect, &mut f);
}

pub fn for_all_in_rect<A: AxisTrait, T: SweepTrait, F: FnMut(ColSingle<T>)>(
    tree: &mut DynTree<A, T>,
    rect: &Rect<T::Num>,
    mut closure: F,
) {
    let mut f = |a: ColSingle<T>| {
        if rect.contains_rect(&(a.rect).0) {
            closure(a);
        }
    };

    let ta = tree.get_iter_mut();
    self::rect_recurse(A::new(), ta, rect, &mut f);
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
            let a2 = A::Next::get(); //self.axis.next();
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
