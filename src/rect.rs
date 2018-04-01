use inner_prelude::*;
use super::*;
use oned::Bleek;

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
            let div=match nn.div{
                Some(div)=>div,
                None=>return
            };
            //let div = nn.div.unwrap();

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
