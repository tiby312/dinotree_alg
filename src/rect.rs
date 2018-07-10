use inner_prelude::*;


macro_rules! get_range{
    ($range:expr)=>{{
        &$range
    }}
}


macro_rules! get_range_mut{
    ($range:expr)=>{{
        &mut $range
    }}
}


macro_rules! rect{
    ($iterator:ty,$colsingle:ty,$get_section:ident,$get_ref:ident)=>{     
        fn rect_recurse<
            A: AxisTrait,
            T: HasAabb,
            F: FnMut($colsingle),
        >(
            this_axis: A,
            m: $iterator,
            rect: &Rect<T::Num>,
            func: &mut F
        ) {
            

            let (nn,rest)=m.next();
            match rest{
                Some((extra,left,right))=>{
                    let FullComp{div,cont}=match extra{
                        Some(b)=>b,
                        None=>return
                    };
                    let sl = $get_section(this_axis.next(),$get_ref!(nn.range), rect.as_axis().get(this_axis.next()));

                    for i in sl {
                        func(i);
                    }
                    let rr = rect.as_axis().get(this_axis);

                    if !(div < rr.left) {
                        self::rect_recurse(this_axis.next(), left, rect, func);
                    }
                    if !(div > rr.right) {
                        self::rect_recurse(this_axis.next(), right, rect, func);
                    }
                },
                None=>{
                    let sl = $get_section(this_axis.next(),$get_ref!(nn.range), rect.as_axis().get(this_axis.next()));

                    for i in sl {
                        func(i);
                    }
                }
            }
        }
    }
}



//TODO test the intersect ones
pub use self::mutable::for_all_intersect_rect_mut;
pub use self::mutable::for_all_in_rect_mut;
pub use self::mutable::naive_for_all_intersect_rect_mut;
pub use self::mutable::naive_for_all_in_rect_mut;


pub use self::constant::for_all_intersect_rect;
pub use self::constant::for_all_in_rect;


mod mutable{
    use oned::get_section_mut;
    use super::*;

    rect!(NdIterMut<(),T>,&mut T,get_section_mut,get_range_mut);
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

        self::rect_recurse(axis, ta, rect, &mut f);
    }

    pub fn naive_for_all_in_rect_mut<T: HasAabb, F: FnMut(&mut T)>(
        bots: &mut [T],
        rect: &Rect<T::Num>,
        mut closure: F,
    ) {
        for b in bots.iter_mut(){
            if rect.contains_rect(b.get()){
                closure(b);
            }
        }

    }

    pub fn naive_for_all_intersect_rect_mut<T: HasAabb, F: FnMut(&mut T)>(
        bots: &mut [T],
        rect: &Rect<T::Num>,
        mut closure: F,
    ) {
        for b in bots.iter_mut(){
            if rect.get_intersect_rect(b.get()).is_some(){
                closure(b);
            }
        }

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
        self::rect_recurse(axis, ta, rect, &mut f);
    }

}
mod constant{

    use oned::get_section;
    use super::*;
    rect!(NdIter<(),T>,&T,get_section,get_range);
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
        self::rect_recurse(axis, ta, rect, &mut f);
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
        self::rect_recurse(axis, ta, rect, &mut f);
    }

}