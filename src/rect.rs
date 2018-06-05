use inner_prelude::*;


use dinotree_inner::*;


macro_rules! get_mut_slice{
    ($range:expr)=>{{
        &mut $range
    }}
}
macro_rules! get_slice{
    ($range:expr)=>{{
        & $range
    }}
}

macro_rules! rect{
    ($iterator:ty,$colsingle:ty,$sweeper:ty,$get_range:ident)=>{     
        fn rect_recurse<
            A: AxisTrait,
            T: HasAabb,
            F: FnMut($colsingle),
        >(
            this_axis: A,
            m: $iterator,
            rect: &Rect<T::Num>,
            func: &mut F,
            sweeper:&mut $sweeper
        ) {
            let (nn, rest) = m.next();
            {
                let sl = sweeper.get_section::<A::Next>($get_range!(nn.range), rect.get_range2::<A::Next>());

                for i in sl {
                    func(i);
                }
            }
            match rest {
                Some((left, right)) => {
                    let div=match nn.div{
                        Some(div)=>div,
                        None=>return
                    };

                    let rr = rect.get_range2::<A>();

                    if !(div < rr.start) {
                        self::rect_recurse(this_axis.next(), left, rect, func,sweeper);
                    }
                    if !(div > rr.end) {
                        self::rect_recurse(this_axis.next(), right, rect, func,sweeper);
                    }
                }
                _ => {}
            }
        }
    }
}


pub use self::mutable::for_all_intersect_rect_mut;
pub use self::mutable::for_all_in_rect_mut;

pub use self::constant::for_all_intersect_rect;
pub use self::constant::for_all_in_rect;


mod mutable{
    use super::*;
    rect!(NdIterMut<(),T>,&mut T,oned::mod_mut::Sweeper<T>,get_mut_slice);
    pub fn for_all_intersect_rect_mut<A: AxisTrait, T: HasAabb>(
        tree: &mut DynTree<A,(),T>,
        rect: &Rect<T::Num>,
        mut closure: impl FnMut(&mut T),
    ) {

        let mut f = |a: &mut T| {
            if rect.intersects_rect(a.get()) {
                closure(a);
            }
        };

        let ta = tree.get_iter_mut();

        let mut sweeper=oned::mod_mut::Sweeper::new();
        self::rect_recurse(A::new(), ta, rect, &mut f,&mut sweeper);
    }

    pub fn for_all_in_rect_mut<A: AxisTrait, T: HasAabb, F: FnMut(&mut T)>(
        tree: &mut DynTree<A,(),T>,
        rect: &Rect<T::Num>,
        mut closure: F,
    ) {
        let mut f = |a: &mut T | {
            if rect.contains_rect(a.get()) {
                closure(a);
            }
        };

        let ta = tree.get_iter_mut();
        let mut sweeper=oned::mod_mut::Sweeper::new();
        self::rect_recurse(A::new(), ta, rect, &mut f,&mut sweeper);
    }

}
mod constant{
    use super::*;
    //rect!(NdIter<(),T>,&T,oned::mod_const::Sweeper<T>,get_slice,make_colsingle);
    
    pub fn for_all_intersect_rect<A: AxisTrait, T: HasAabb>(
        tree: &DynTree<A,(),T>,
        rect: &Rect<T::Num>,
        mut closure: impl FnMut(&T),
    ) {

        let mut f = |a: &T| {
            if rect.intersects_rect(a.get()) {
                closure(a);
            }
        };

        let ta = tree.get_iter();

        unimplemented!();
        //let mut sweeper=oned::mod_const::Sweeper::new();
        //self::rect_recurse(A::new(), ta, rect, &mut f,&mut sweeper);
    }

    pub fn for_all_in_rect<A: AxisTrait, T: HasAabb>(
        tree: &mut DynTree<A,(),T>,
        rect: &Rect<T::Num>,
        mut closure: impl FnMut(&T),
    ) {
        let mut f = |a: &T| {
            if rect.contains_rect(a.get()) {
                closure(a);
            }
        };

        let ta = tree.get_iter();

        unimplemented!();
        //let mut sweeper=oned::mod_const::Sweeper::new();
        //self::rect_recurse(A::new(), ta, rect, &mut f,&mut sweeper);
    }

}