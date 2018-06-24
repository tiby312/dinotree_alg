use inner_prelude::*;




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
                let sl = sweeper.get_section(this_axis.next(),$get_range!(nn.range), rect.as_axis().get(this_axis.next()));

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

                    let rr = rect.as_axis().get(this_axis);

                    if !(div < rr.left) {
                        self::rect_recurse(this_axis.next(), left, rect, func,sweeper);
                    }
                    if !(div > rr.right) {
                        self::rect_recurse(this_axis.next(), right, rect, func,sweeper);
                    }
                }
                _ => {}
            }
        }
    }
}



//TODO test the intersect ones
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
            if rect.get_intersect_rect(a.get()).is_some() {
                closure(a);
            }
        };

        let axis=tree.get_axis();
        let ta = tree.get_iter_mut();

        let mut sweeper=oned::mod_mut::Sweeper::new();
        self::rect_recurse(axis, ta, rect, &mut f,&mut sweeper);
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
        let axis=tree.get_axis();
        let ta = tree.get_iter_mut();
        let mut sweeper=oned::mod_mut::Sweeper::new();
        self::rect_recurse(axis, ta, rect, &mut f,&mut sweeper);
    }

}
mod constant{
    use super::*;
    rect!(NdIter<(),T>,&T,oned::mod_const::Sweeper<T>,get_slice);
    //rect!(NdIter<(),T>,&T,oned::mod_const::Sweeper<T>,get_slice,make_colsingle);
    
    pub fn for_all_intersect_rect<A: AxisTrait, T: HasAabb>(
        tree: &DynTree<A,(),T>,
        rect: &Rect<T::Num>,
        mut closure: impl FnMut(&T),
    ) {
        
        let mut f = |a: &T| {
            if rect.get_intersect_rect(a.get()).is_some() {
                closure(a);
            }
        };
        let axis=tree.get_axis();
        let ta = tree.get_iter();
        let mut sweeper=oned::mod_const::Sweeper::new();
        self::rect_recurse(axis, ta, rect, &mut f,&mut sweeper);
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

        let axis=tree.get_axis();
        let ta = tree.get_iter();
        let mut sweeper=oned::mod_const::Sweeper::new();
        self::rect_recurse(axis, ta, rect, &mut f,&mut sweeper);
    }

}