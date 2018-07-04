use inner_prelude::*;


macro_rules! rect{
    ($iterator:ty,$colsingle:ty,$get_section:ident,$leafdyn:ident)=>{     
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
            
            //use dinotree_inner::
            match compt::CTreeIteratorEx::next(m){
                compt::LeafEx::Leaf(leaf)=>{
                    let sl = $get_section(this_axis.next(),leaf.range, rect.as_axis().get(this_axis.next()));

                    for i in sl {
                        func(i);
                    }
                },
                compt::LeafEx::NonLeaf((nonleaf,left,right))=>{
                    match nonleaf{
                        $leafdyn::NoBotsHereOrBelow=>{
                            return;
                        },
                        $leafdyn::Bots(bots,cont,div)=>{
                            let sl = $get_section(this_axis.next(),bots, rect.as_axis().get(this_axis.next()));

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
                        }
                    }
                }
            }
            
            /*
            {
                let sl = $get_section(this_axis.next(),$get_range!(nn.range), rect.as_axis().get(this_axis.next()));

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
                        self::rect_recurse(this_axis.next(), left, rect, func);
                    }
                    if !(div > rr.right) {
                        self::rect_recurse(this_axis.next(), right, rect, func);
                    }
                }
                _ => {}
            }
            */
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

    rect!(NdIterMut<(),T>,&mut T,get_section_mut,NonLeafDynMut);
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
    rect!(NdIter<(),T>,&T,get_section,NonLeafDyn);
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