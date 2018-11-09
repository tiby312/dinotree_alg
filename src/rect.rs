//! Provides two basic functions: for_all_in_rect, for_all_intersect_rect that both have similar api like this:
//!
//! ```ignore
//! pub fn for_all_in_rect_mut<A: AxisTrait, T: HasAabb>(
//!        tree: &mut DinoTree<A,(),T>,
//!        rect: &Rect<T::Num>,
//!        mut closure: impl FnMut(&mut T),
//! );
//! ```
//!
//! for_all_in_rect allows the user to retreive references to all bots whose aabb's are strictly inside of the specified rectangle.
//! for_all_intersect_rect is similar, but will return all bots who are inside as well as all bots whose aabb's intersect the rect.
//! The user is allowed to hold on to the mutable references returned for as long as the tree itself is mutably borrowed.
//!
//! # Safety
//!
//! There is no unsafe code in this module.
//!
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
                    let &FullComp{div,cont:_}=match extra{
                        Some(b)=>b,
                        None=>return
                    };
                    let sl = $get_section(this_axis.next(),$get_ref!(nn.range), rect.get_range(this_axis.next()));

                    for i in sl {
                        func(i);
                    }
                    let rr = rect.get_range(this_axis);

                    if !(div < rr.left) {
                        self::rect_recurse(this_axis.next(), left, rect, func);
                    }
                    if !(div > rr.right) {
                        self::rect_recurse(this_axis.next(), right, rect, func);
                    }
                },
                None=>{
                    let sl = $get_section(this_axis.next(),$get_ref!(nn.range), rect.get_range(this_axis.next()));

                    for i in sl {
                        func(i);
                    }
                }
            }
        }
    }
}


pub fn for_all_not_in_rect_mut<A:AxisTrait,T:HasAabb>(tree:&mut DinoTree<A,(),T>,rect:&Rect<T::Num>,closure:impl FnMut(&mut T)){
    fn rect_recurse<A:AxisTrait,T:HasAabb,F:FnMut(&mut T)>(axis:A,it:VistrMut<(),T>,rect:&Rect<T::Num>,mut closure:F)->F{
        let (nn,rest)=it.next();
        
        //TODO exploit sorted property.
        for a in nn.range.iter_mut(){
            if !rect.contains_rect(a.get()){
                closure(a);
            }
        }
        
        match rest{
            Some((extra,left,right))=>{
                let &FullComp{div,cont:_}=match extra{
                    Some(b)=>b,
                    None=>return closure,
                };

                match rect.get_range(axis).left_or_right_or_contain(&div){
                    std::cmp::Ordering::Less=>{
                        rect_recurse(axis.next(),left,rect,closure)
                    },
                    std::cmp::Ordering::Greater=>{
                        rect_recurse(axis.next(),right,rect,closure)
                    },
                    std::cmp::Ordering::Equal=>{
                        let closure=rect_recurse(axis.next(),left,rect,closure);
                        rect_recurse(axis.next(),right,rect,closure)
                    }
                }
            },
            None=>{closure}
        }
        
    }
    rect_recurse(tree.axis(),tree.vistr_mut(),rect,closure);
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

    rect!(VistrMut<(),T>,&mut T,get_section_mut,get_range_mut);
    pub fn for_all_intersect_rect_mut<A: AxisTrait, T: HasAabb>(
        tree: &mut DinoTree<A,(),T>,
        rect: &Rect<T::Num>,
        mut closure: impl FnMut(&mut T),
    ) {
        let mut f = |a: &mut T| {
            if rect.get_intersect_rect(a.get()).is_some() {
                closure(a);
            }
        };

        let axis=tree.axis();
        let ta = tree.vistr_mut();

        self::rect_recurse(axis, ta, rect, &mut f);
    }

    pub fn naive_for_all_in_rect_mut<T: HasAabb>(
        bots: &mut [T],
        rect: &Rect<T::Num>,
        mut closure: impl FnMut(&mut T),
    ) {
        for b in bots.iter_mut(){
            if rect.contains_rect(b.get()){
                closure(b);
            }
        }

    }

    pub fn naive_for_all_intersect_rect_mut<T: HasAabb>(
        bots: &mut [T],
        rect: &Rect<T::Num>,
        mut closure: impl FnMut(&mut T),
    ) {
        for b in bots.iter_mut(){
            if rect.get_intersect_rect(b.get()).is_some(){
                closure(b);
            }
        }

    }
    pub fn for_all_in_rect_mut<A: AxisTrait, T: HasAabb>(
        tree: &mut DinoTree<A,(),T>,
        rect: &Rect<T::Num>,
        mut closure: impl FnMut(&mut T),
    ) {
        let mut f = |a: &mut T | {
            if rect.contains_rect(a.get()) {
                closure(a);
            }
        };
        let axis=tree.axis();
        let ta = tree.vistr_mut();
        self::rect_recurse(axis, ta, rect, &mut f);
    }

}
mod constant{

    use oned::get_section;
    use super::*;
    rect!(Vistr<(),T>,&T,get_section,get_range);
    //rect!(Vistr<(),T>,&T,oned::mod_const::Sweeper<T>,get_slice,make_colsingle);
    
    pub fn for_all_intersect_rect<A: AxisTrait, T: HasAabb>(
        tree: &DinoTree<A,(),T>,
        rect: &Rect<T::Num>,
        mut closure: impl FnMut(&T),
    ) {
        
        let mut f = |a: &T| {
            if rect.get_intersect_rect(a.get()).is_some() {
                closure(a);
            }
        };
        let axis=tree.axis();
        let ta = tree.vistr();
        self::rect_recurse(axis, ta, rect, &mut f);
    }

    pub fn for_all_in_rect<A: AxisTrait, T: HasAabb>(
        tree: &DinoTree<A,(),T>,
        rect: &Rect<T::Num>,
        mut closure: impl FnMut(&T),
    ) {
        
        let mut f = |a: &T| {
            if rect.contains_rect(a.get()) {
                closure(a);
            }
        };

        let axis=tree.axis();
        let ta = tree.vistr();
        self::rect_recurse(axis, ta, rect, &mut f);
    }

}