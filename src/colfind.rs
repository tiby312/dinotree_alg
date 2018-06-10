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



//go_down!(,);






macro_rules! anchor{
    ($slice:ty,$node:ty,$get_slice:ident)=>{
        mod anchor{
            use super::*;
            pub struct DestructuredAnchor<'a,T:HasAabb+'a,AnchorAxis:AxisTrait+'a>{
                pub cont:&'a Range<T::Num>,
                _div:&'a T::Num,
                pub range:$slice,
                _p:PhantomData<AnchorAxis>
            }
            pub enum ErrEnum{
                NoBots,
                NoChildrenOrBots
            }
            impl<'a,T:HasAabb+'a,AnchorAxis:AxisTrait+'a> DestructuredAnchor<'a,T,AnchorAxis>{

                pub fn new(nd:$node)->Result<DestructuredAnchor<'a,T,AnchorAxis>,ErrEnum>{
                    let cont=match &nd.cont{
                        &Some(ref x)=>{x},
                        &None=>return Err(ErrEnum::NoBots)
                    };
                    let div=match &nd.div{
                        &Some(ref x)=>{x},
                        &None=>return Err(ErrEnum::NoChildrenOrBots)
                    };
                    
                    let range=$get_slice!(nd.range);
                    Ok(DestructuredAnchor{_p:PhantomData,cont,_div:div,range})
                }
            }
        }
        
    }
}




macro_rules! go_down{
    ($sweeper:ty,$anchor:ty,$iterator:ty)=>{

        fn go_down<
            A: AxisTrait, //this axis
            B: AxisTrait, //parent axis
            X: HasAabb ,
            F: ColMulti<T = X>
        >(
            this_axis: A,
            parent_axis: B,
            sweeper: $sweeper,
            anchor: $anchor,
            m: $iterator,
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
                            if !(div < anchor.cont.start) {
                                self::go_down(this_axis.next(), parent_axis, sweeper, anchor, left, func,depth.next_down());
                            };
                            if !(div > anchor.cont.end) {
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

    }
}


macro_rules! recurse{
    ($sweeper:ty,$anchor:ty,$iterator:ty,$get_ref:ident,$create_wrap:ident,$create_sweep:ident)=>{


        fn recurse<
            A: AxisTrait,
            JJ: par::Joiner,
            X: HasAabb + Send ,//+$sync
            F: ColMulti<T = X>+Send,
            K: TreeTimerTrait
        >(
            this_axis: A,
            par: JJ,
            sweeper:$sweeper,// &mut oned::Sweeper<F::T>,
            m: $iterator, //NdIterMut<(),X>
            mut clos: F,
            mut timer_log: K,
            level:Depth
        ) -> (F,K::Bag) {
            timer_log.start();

            let (nn, rest) = m.next();

            let k = match rest {
                None => {
                    sweeper.find_2d::<A::Next, _>($get_ref!(nn.range), ColMultiWrapper(&mut clos));

                    (clos,timer_log.leaf_finish())
                },
                Some((mut left, mut right)) => {

                    match anchor::DestructuredAnchor::<X,A>::new(nn){
                        Ok(mut nn)=>{
                            sweeper.find_2d::<A::Next, _>(nn.range, ColMultiWrapper(&mut clos));

                            //let left = compt::WrapGen::new(&mut left);
                            //let right = compt::WrapGen::new(&mut right);
                            //left.create_wrap();
                            let left=$create_wrap!(left);//.create_wrap_mut();
                            let right=$create_wrap!(right);//.create_wrap_mut();

                            self::go_down(this_axis.next(), this_axis, sweeper, $get_ref!(nn), left, &mut clos,level.next_down());
                            self::go_down(this_axis.next(), this_axis, sweeper, $get_ref!(nn), right, &mut clos,level.next_down());
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
                            let mut sweeper = $create_sweep!();
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
    }
}




macro_rules! colfind{
    ($bleek:ident,$anchor:ty,$sweeper:ty,$node:ty,$ref:ty,$get_slice:ident)=>{

        pub trait ColMulti:Sized {
            type T: HasAabb;
            fn collide(&mut self, a: $ref, b: $ref);
            fn div(self)->(Self,Self);
            fn add(self,b:Self)->Self;
        }

        struct ColMultiWrapper<'a, C: ColMulti + 'a>(pub &'a mut C);


            



        fn for_every_bijective_pair<A: AxisTrait, B: AxisTrait, F:$bleek,L:LeafTracker>(
            this: $node,
            parent: $anchor,
            sweeper: $sweeper, 
            mut func: F,
            leaf_tracker:L
        ) {
            //Can be evaluated at compile time
            if A::get() != B::get() {

                let (parent_box,parent_bots)=(parent.cont,$get_slice!(parent.range));

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
                    parent.range,
                    func,
                );
            }
        }

    }
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




pub fn query<A:AxisTrait,T:HasAabb>(tree:&DynTree<A,(),T>,mut func:impl FnMut(&T,&T)){

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
        pub struct Wrapper<'a, T: HasAabb, F: FnMut(&T, &T) + 'a>(
            pub &'a mut F,
            pub PhantomData<T>,
        );

        impl<'a, T: HasAabb, F: FnMut(&T, &T) + 'a> Clone for Wrapper<'a, T, F> {
            fn clone(&self) -> Wrapper<'a, T, F> {
                unreachable!()
            }
        }

        impl<'a, T: HasAabb, F: FnMut(&T, &T) + 'a> self::constant::ColMulti for Wrapper<'a, T, F> {
            type T = Wrap<T>;

            fn collide(&mut self, a: &Wrap<T>, b: &Wrap<T>) {
                self.0(&a.0,&b.0);
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
        unsafe impl<'a, T: HasAabb, F: FnMut(& T, & T) + 'a> Send
            for Wrapper<'a, T, F>
        {
        }
        unsafe impl<'a, T: HasAabb, F: FnMut(& T, & T) + 'a> Sync
            for Wrapper<'a, T, F>
        {
        }
    }

    let wrap=wrap::Wrapper(&mut func,PhantomData);

    let tree:&DynTree<A,(),wrap::Wrap<T>>=unsafe{std::mem::transmute(tree)};
    self::constant::for_every_col_pair::<_,_, _, _, TreeTimerEmpty>(
        A::new(),
        par::Sequential,
        tree,
        wrap,
    );
}

pub fn query_mut<A:AxisTrait,T:HasAabb>(tree:&mut DynTree<A,(),T>,mut func:impl FnMut(&mut T,&mut T)){

    //TODO condense this using macros
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

        impl<'a, T: HasAabb, F: FnMut(&mut T, &mut T) + 'a> self::mutable::ColMulti for Wrapper<'a, T, F> {
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
    self::mutable::for_every_col_pair_mut::<_,_, _, _, TreeTimerEmpty>(
        A::new(),
        par::Sequential,
        tree,
        wrap,
    );
    
}



pub fn query_par<A:AxisTrait,T:HasAabb+Send>(tree:&DynTree<A,(),T>,func:impl Fn(&T,&T)+Copy+Send){

    let c1=move |_:&mut (),a:&T,b:&T|{
        func(a,b);
    };

    let c2=|_:()|((),());
    let c3=|_:(),_:()|();

    let clos = self::constant::closure_struct::ColMultiStruct{aa
        :(),a:c1,f2:c2,f3:c3,_p:PhantomData};



    const DEPTH_SEQ:usize=4;

    let height=tree.get_height();
    let gg=if height<=DEPTH_SEQ{
        Depth(0)
    }else{
        Depth(height-DEPTH_SEQ)
    };

    self::constant::for_every_col_pair::<_,_, _, _, TreeTimerEmpty>(
        A::new(),
        par::Parallel::new(gg),
        tree,
        clos,
    );    
}

/*

pub fn query_mut_par<A:AxisTrait,N:NumTrait,T:Send>(tree:&mut DynTree<A,(),BBox<N,T>>,mut func:impl Fn(BBoxDet<N,T>,BBoxDet<N,T>)+Copy+Send){
    query_mut_par_unchecked(tree,move |a,b|{
        func(a.destruct(),b.destruct())
    });
}
*/

pub fn query_par_mut<A:AxisTrait,T:HasAabb+Send>(tree:&mut DynTree<A,(),T>,func:impl Fn(&mut T,&mut T)+Copy+Send){

    let c1=move |_:&mut (),a:&mut T,b:&mut T|{
        func(a,b);
    };

    let c2=|_:()|((),());
    let c3=|_:(),_:()|();

    let clos = self::mutable::closure_struct::ColMultiStruct{aa
        :(),a:c1,f2:c2,f3:c3,_p:PhantomData};



    const DEPTH_SEQ:usize=4;

    let height=tree.get_height();
    let gg=if height<=DEPTH_SEQ{
        Depth(0)
    }else{
        Depth(height-DEPTH_SEQ)
    };

    self::mutable::for_every_col_pair_mut::<_,_, _, _, TreeTimerEmpty>(
        A::new(),
        par::Parallel::new(gg),
        tree,
        clos,
    );        
}

/*
pub fn for_every_col_pair<
    'a,
    A: AxisTrait,
    T: HasAabb,
>(
    kdtree: &DynTree<'a,A,(), T>,
    func:impl Fn(&T,&T),
    )->(F,K::Bag){

    unimplemented!();
}
*/
/*


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
*/


mod mutable{
    use super::*;

    pub fn for_every_col_pair_mut<
        A: AxisTrait,
        JJ: par::Joiner,
        T: HasAabb+Send,
        F: ColMulti<T = T>+Send,
        K: TreeTimerTrait,
    >(
        this_axis: A,
        par: JJ,
        kdtree: &mut DynTree<A,(), T>,
        clos: F,
    ) -> (F,K::Bag) {
        let height = kdtree.get_height();
        let dt = kdtree.get_iter_mut();
        let mut sweeper = oned::mod_mut::Sweeper::new();

        let h = K::new(height);
        let bag = self::recurse(this_axis, par, &mut sweeper, dt, clos, h,Depth(0));
        bag
    }

    //TODO condense this using macros
    pub mod closure_struct {
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
    


    impl<'a, C: ColMulti + 'a> oned::mod_mut::Bleek for ColMultiWrapper<'a, C> {
        type T = C::T;
        fn collide(&mut self, a:&mut Self::T, b: &mut Self::T) {
            self.0.collide(a, b);
        }
    }


    macro_rules! get_mut_slice{
        ($range:expr)=>{{
            &mut $range
        }}
    }



    macro_rules! create_wrap_mut{
        ($e:expr)=>{
            $e.create_wrap_mut()
        }
    }

    macro_rules! create_sweep{
        ()=>{
            oned::mod_mut::Sweeper::new();
        }
    }
    anchor!(&'a mut [T],&'a mut NodeDyn<(),T>,get_mut_slice);


    go_down!(&mut oned::mod_mut::Sweeper<F::T>,&mut anchor::DestructuredAnchor<X,B>,NdIterMut<(),X>);

    recurse!(&mut oned::mod_mut::Sweeper<F::T>,&mut anchor::DestructuredAnchor<X,B>,NdIterMut<(),X>,get_mut_slice,create_wrap_mut,create_sweep);

    use oned::mod_mut::Bleek;
    colfind!( Bleek,&mut anchor::DestructuredAnchor<F::T,B>,&mut oned::mod_mut::Sweeper<F::T>,&mut NodeDyn<(),F::T>,&mut Self::T,get_mut_slice);

}

mod constant{
    use super::*;


    pub mod closure_struct {
        use super::*;

        pub struct ColMultiStruct<
            A:Send,
            T: HasAabb,
            F: Fn(&mut A,&T, &T) + Send + Copy,
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
            F: Fn(&mut A,&T, &T) + Send + Copy,
            F2:Fn(A)->(A,A)+Copy,
            F3:Fn(A,A)->A+Copy
        > ColMulti for ColMultiStruct<A,T,  F,F2,F3>
        {
            type T = T;
        
            fn collide(&mut self,a: &T, b: &T) {
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
    macro_rules! get_slice{
        ($range:expr)=>{{
            & $range
        }}
    }



    macro_rules! create_wrap{
        ($e:expr)=>{
            $e.create_wrap()
        }
    }

    macro_rules! create_sweep{
        ()=>{
            oned::mod_const::Sweeper::new();
        }
    }

    impl<'a, C: ColMulti + 'a> oned::mod_const::Bleek for ColMultiWrapper<'a, C> {
        type T = C::T;
        fn collide(&mut self, a:&Self::T, b: &Self::T) {
            self.0.collide(a, b);
        }
    }

    
    pub fn for_every_col_pair<
        A: AxisTrait,
        JJ: par::Joiner,
        T: HasAabb+Send,
        F: ColMulti<T = T>+Send,
        K: TreeTimerTrait,
    >(
        this_axis: A,
        par: JJ,
        kdtree: &DynTree<A,(), T>,
        clos: F,
    ) -> (F,K::Bag) {
        let height = kdtree.get_height();
        let dt = kdtree.get_iter();
        let mut sweeper = oned::mod_const::Sweeper::new();

        let h = K::new(height);
        let bag = self::recurse(this_axis, par, &mut sweeper, dt, clos, h,Depth(0));
        bag
    }


    recurse!(&mut oned::mod_const::Sweeper<F::T>,&anchor::DestructuredAnchor<X,B>,NdIter<(),X>,get_slice,create_wrap,create_sweep);
    
    go_down!(&mut oned::mod_const::Sweeper<F::T>,&anchor::DestructuredAnchor<X,B>,NdIter<(),X>);

    anchor!(&'a [T],&'a NodeDyn<(),T>,get_slice);

    use oned::mod_const::Bleek;
    colfind!( Bleek,&anchor::DestructuredAnchor<F::T,B>,&mut oned::mod_const::Sweeper<F::T>,&NodeDyn<(),F::T>,& Self::T,get_slice);

}