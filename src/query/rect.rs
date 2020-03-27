//! Provides two basic functions: for_all_in_rect, for_all_intersect_rect that both have similar api.
//!
//!
//! for_all_in_rect allows the user to retreive references to all bots whose aabb's are strictly inside of the specified rectangle.
//! for_all_intersect_rect is similar, but will return all bots who are inside as well as all bots whose aabb's intersect the rect.
//! The user is allowed to hold on to the mutable references returned for as long as the tree itself is mutably borrowed.
//!

use crate::query::inner_prelude::*;

macro_rules! rect {
    ($iterator:ty,$colsingle:ty,$get_section:ident,$get_node:ident) => {
        fn rect_recurse<'a, A: Axis, N: Node, F: FnMut($colsingle)>(
            this_axis: A,
            m: $iterator,
            rect: &Rect<N::Num>,
            func: &mut F,
        ) {
            let (nn, rest) = m.next();
            let nn = nn.$get_node();
            match rest {
                Some([left, right]) => {
                    let div = match nn.div {
                        Some(b) => b,
                        None => return,
                    };

                    let sl =
                        $get_section(this_axis.next(), nn.bots, rect.get_range(this_axis.next()));

                    for i in sl {
                        func(i);
                    }
                    let rr = rect.get_range(this_axis);

                    if !(*div < rr.start) {
                        self::rect_recurse(this_axis.next(), left, rect, func);
                    }
                    if !(*div > rr.end) {
                        self::rect_recurse(this_axis.next(), right, rect, func);
                    }
                }
                None => {
                    let sl =
                        $get_section(this_axis.next(), nn.bots, rect.get_range(this_axis.next()));

                    for i in sl {
                        func(i);
                    }
                }
            }
        }
    };
}

pub fn naive_for_all_not_in_rect_mut<T: Aabb>(
    bots: PMut<[T]>,
    rect: &Rect<T::Num>,
    mut closure: impl FnMut(PMut<T>),
) {
    for b in bots.iter_mut() {
        if !rect.contains_rect(b.get()) {
            closure(b);
        }
    }
}

pub fn for_all_not_in_rect_mut<A: Axis, T: Aabb>(
    tree: &mut DinoTree<A, T>,
    rect: &Rect<T::Num>,
    closure: impl FnMut(PMut<T>),
) {
    fn rect_recurse<A: Axis, N: Node, F: FnMut(PMut<N::T>)>(
        axis: A,
        it: VistrMut<N>,
        rect: &Rect<N::Num>,
        mut closure: F,
    ) -> F {
        let (nn, rest) = it.next();
        let nn = nn.get_mut();
        //TODO exploit sorted property.
        for a in nn.bots.iter_mut() {
            if !rect.contains_rect(a.get()) {
                closure(a);
            }
        }

        match rest {
            Some([left, right]) => {
                let div = match nn.div {
                    Some(b) => b,
                    None => return closure,
                };

                match rect.get_range(axis).contains_ext(*div) {
                    core::cmp::Ordering::Less => rect_recurse(axis.next(), left, rect, closure),
                    core::cmp::Ordering::Greater => rect_recurse(axis.next(), right, rect, closure),
                    core::cmp::Ordering::Equal => {
                        let closure = rect_recurse(axis.next(), left, rect, closure);
                        rect_recurse(axis.next(), right, rect, closure)
                    }
                }
            }
            None => closure,
        }
    }
    rect_recurse(tree.axis(), tree.vistr_mut(), rect, closure);
}

pub use constant::*;
pub use mutable::*;

mod mutable {
    use super::*;
    use crate::query::colfind::oned::get_section_mut;

    rect!(VistrMut<N>, PMut<N::T>, get_section_mut, get_mut);
    pub fn for_all_intersect_rect_mut<A: Axis, T:Aabb>(
        tree: &mut DinoTree<A, T>,
        rect: &Rect<T::Num>,
        mut closure: impl FnMut(PMut<T>),
    ) {
        let mut f = |a: PMut<T>| {
            if rect.get_intersect_rect(a.get()).is_some() {
                closure(a);
            }
        };

        let axis = tree.axis();
        let ta = tree.vistr_mut();

        self::rect_recurse(axis, ta, rect, &mut f);
    }

    pub fn naive_for_all_in_rect_mut<T: Aabb>(
        bots: PMut<[T]>,
        rect: &Rect<T::Num>,
        mut closure: impl FnMut(PMut<T>),
    ) {
        for b in bots.iter_mut() {
            if rect.contains_rect(b.get()) {
                closure(b);
            }
        }
    }

    pub fn naive_for_all_intersect_rect_mut<T: Aabb>(
        bots: PMut<[T]>,
        rect: &Rect<T::Num>,
        mut closure: impl FnMut(PMut<T>),
    ) {
        for b in bots.iter_mut() {
            if rect.get_intersect_rect(b.get()).is_some() {
                closure(b);
            }
        }
    }
    pub fn for_all_in_rect_mut<A: Axis, T: Aabb>(
        tree: &mut DinoTree<A, T>,
        rect: &Rect<T::Num>,
        mut closure: impl FnMut(PMut<T>),
    ) {
        let mut f = |a: PMut<T>| {
            if rect.contains_rect(a.get()) {
                closure(a);
            }
        };
        let axis = tree.axis();
        let ta = tree.vistr_mut();
        self::rect_recurse(axis, ta, rect, &mut f);
    }
}

mod constant {

    use super::*;
    use crate::query::colfind::oned::get_section;
    rect!(Vistr<'a, N>, &'a N::T, get_section, get);

    pub fn for_all_intersect_rect<'a, A: Axis, T: Aabb>(
        tree: &'a DinoTree<A, T>,
        rect: &Rect<T::Num>,
        mut closure: impl FnMut(&'a T),
    ) {
        let mut f = |a: &'a T| {
            if rect.get_intersect_rect(a.get()).is_some() {
                closure(a);
            }
        };
        let axis = tree.axis();
        let ta = tree.vistr();
        self::rect_recurse(axis, ta, rect, &mut f);
    }

    pub fn for_all_in_rect<'a, A: Axis, T: Aabb>(
        tree: &'a DinoTree<A, T>,
        rect: &Rect<T::Num>,
        mut closure: impl FnMut(&'a T),
    ) {
        let mut f = |a: &'a T| {
            if rect.contains_rect(a.get()) {
                closure(a);
            }
        };

        let axis = tree.axis();
        let ta = tree.vistr();
        self::rect_recurse(axis, ta, rect, &mut f);
    }
}

///Indicates that the user supplied a rectangle
///that intersects with a another one previously queries
///in the session.
#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub struct RectIntersectErr;

/// If we have two non intersecting rectangles, it is safe to return to the user two sets of mutable references
/// of the bots strictly inside each rectangle since it is impossible for a bot to belong to both sets.
///
/// # Safety
///
/// Unsafe code is used.  We unsafely convert the references returned by the rect query
/// closure to have a longer lifetime.
/// This allows the user to store mutable references of non intersecting rectangles at the same time.
/// If two requested rectangles intersect, an error is returned.
///
///Handles a multi rect mut "sessions" within which
///the user can query multiple non intersecting rectangles.
pub struct MultiRectMut<'a,'b:'a, A: Axis, T:Aabb> {
    tree: &'a mut DinoTree<'b,A, T>,
    rects: Vec<Rect<T::Num>>,
}

impl<'a,'b:'a, A: Axis,T:Aabb> MultiRectMut<'a,'b, A, T> {
    pub fn new(tree: &'a mut DinoTree<'b,A, T>) -> Self {
        MultiRectMut {
            tree,
            rects: Vec::new(),
        }
    }
    pub fn for_all_in_rect_mut(
        &mut self,
        rect: Rect<T::Num>,
        mut func: impl FnMut(PMut<'a, T>),
    ) -> Result<(), RectIntersectErr> {
        for r in self.rects.iter() {
            if rect.get_intersect_rect(r).is_some() {
                return Err(RectIntersectErr);
            }
        }

        self.rects.push(rect);

        for_all_in_rect_mut(self.tree, &rect, |bbox: PMut<T>| {
            //This is only safe to do because the user is unable to mutate the bounding box.
            let bbox: PMut<'a,T > = unsafe { core::mem::transmute(bbox) };
            func(bbox);
        });

        Ok(())
    }
}
