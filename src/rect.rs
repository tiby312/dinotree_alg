//! Provides two basic functions: for_all_in_rect, for_all_intersect_rect that both have similar api.
//!
//!
//! for_all_in_rect allows the user to retreive references to all bots whose aabb's are strictly inside of the specified rectangle.
//! for_all_intersect_rect is similar, but will return all bots who are inside as well as all bots whose aabb's intersect the rect.
//! The user is allowed to hold on to the mutable references returned for as long as the tree itself is mutably borrowed.
//!

use crate::inner_prelude::*;



macro_rules! rect{
    ($iterator:ty,$colsingle:ty,$get_section:ident,$get_node:ident)=>{     
        fn rect_recurse<
            A: AxisTrait,
            //T: HasAabb,
            N:NodeTrait,
            F: FnMut($colsingle),
        >(
            this_axis: A,
            m: $iterator,
            rect: &Rect<N::Num>,
            func: &mut F
        ) {
            

            let (nn,rest)=m.next();
            let nn=nn.$get_node();
            match rest{
                Some([left,right])=>{

                    let div=match nn.div{
                        Some(b)=>b,
                        None=>return
                    };

                    let sl = $get_section(this_axis.next(),nn.bots,rect.get_range(this_axis.next()));

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
                },
                None=>{
                    let sl = $get_section(this_axis.next(),nn.bots, rect.get_range(this_axis.next()));

                    for i in sl {
                        func(i);
                    }
                }
            }
        }
    }
}



pub fn naive_for_all_not_in_rect_mut<T:HasAabb>(bots:&mut [T],rect:&Rect<T::Num>,mut closure:impl FnMut(ProtectedBBox<T>)){
    let bots = ProtectedBBoxSlice::new(bots);

    for b in bots.iter_mut(){
        if !rect.contains_rect(b.get()){
            closure(b);
        }
    }
}

pub fn for_all_not_in_rect_mut<A:AxisTrait,N:NodeTrait>(tree:&mut DinoTree<A,N>,rect:&Rect<N::Num>,closure:impl FnMut(ProtectedBBox<N::T>)){
    fn rect_recurse<A:AxisTrait,N:NodeTrait,F:FnMut(ProtectedBBox<N::T>)>(axis:A,it:VistrMut<N>,rect:&Rect<N::Num>,mut closure:F)->F{
        let (nn,rest)=it.next();
        let nn=nn.get_mut();
        //TODO exploit sorted property.
        for a in nn.bots.iter_mut(){
            if !rect.contains_rect(a.get()){
                closure(a);
            }
        }
        
        match rest{
            Some([left,right])=>{
                
                let div=match nn.div{
                    Some(b)=>b,
                    None=>return closure,
                };

                match rect.get_range(axis).contains_ext(*div){
                    core::cmp::Ordering::Less=>{
                        rect_recurse(axis.next(),left,rect,closure)
                    },
                    core::cmp::Ordering::Greater=>{
                        rect_recurse(axis.next(),right,rect,closure)
                    },
                    core::cmp::Ordering::Equal=>{
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




pub use self::mutable::naive_for_all_intersect_rect_mut;
pub use self::mutable::naive_for_all_in_rect_mut;
pub use constant::*;
pub use mutable::*;

/*
pub struct RectQueryBuilder<'a,K:DinoTreeRefTrait>{
    tree:&'a K,
    rect:Rect<K::Num>
}
impl<'a,K:DinoTreeRefTrait>  RectQueryBuilder<'a,K>{
    pub fn new(tree:&'a K,rect:Rect<K::Num>)->Self{
        RectQueryBuilder{tree,rect}
    }
    pub fn for_all_in(&self,closure: impl FnMut(&K::Item)){
        constant::for_all_in_rect(self.tree,&self.rect,closure);
    }
    pub fn for_all_intersect(&self,closure: impl FnMut(&K::Item)){
        constant::for_all_intersect_rect(self.tree,&self.rect,closure);
    }

}
pub struct RectQueryMutBuilder<'a,K:DinoTreeRefMutTrait>{
    tree:&'a mut K,
    rect:Rect<K::Num>
}

impl<'a,K:DinoTreeRefMutTrait>  RectQueryMutBuilder<'a,K>{
    pub fn new(tree:&'a mut K,rect:Rect<K::Num>)->Self{
        RectQueryMutBuilder{tree,rect}
    }
    pub fn for_all_in_mut(&mut self, closure: impl FnMut(ProtectedBBox<K::Item>)){
        mutable::for_all_in_rect_mut(self.tree,&self.rect,closure);
    }
    pub fn for_all_intersect_mut(&mut self, closure: impl FnMut(ProtectedBBox<K::Item>)){
        mutable::for_all_intersect_rect_mut(self.tree,&self.rect,closure);
    }
    pub fn for_all_not_in_mut(&mut self, closure: impl FnMut(ProtectedBBox<K::Item>)){
        for_all_not_in_rect_mut(self.tree,&self.rect,closure);
    }
}

impl<'a,K:DinoTreeRefMutTrait> core::convert::From<RectQueryMutBuilder<'a,K>> for RectQueryBuilder<'a,K>{
    fn from(a:RectQueryMutBuilder<'a,K>)->RectQueryBuilder<'a,K>{
        RectQueryBuilder{tree:a.tree,rect:a.rect}
    }
}
*/


mod mutable{
    use crate::colfind::oned::get_section_mut;
    use super::*;

    rect!(VistrMut<N>,ProtectedBBox<N::T>,get_section_mut,get_mut);
    pub fn for_all_intersect_rect_mut<A:AxisTrait,N:NodeTrait>(
        tree: &mut DinoTree<A,N>,
        rect: &Rect<N::Num>,
        mut closure: impl FnMut(ProtectedBBox<N::T>),
    ) {
        let mut f = |a: ProtectedBBox<N::T>| {
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
        mut closure: impl FnMut(ProtectedBBox<T>),
    ) {
        let bots = ProtectedBBoxSlice::new(bots);

        for b in bots.iter_mut(){
            if rect.contains_rect(b.get()){
                closure(b);
            }
        }

    }

    pub fn naive_for_all_intersect_rect_mut<T: HasAabb>(
        bots: &mut [T],
        rect: &Rect<T::Num>,
        mut closure: impl FnMut(ProtectedBBox<T>),
    ) {
        let bots = ProtectedBBoxSlice::new(bots);
        for b in bots.iter_mut(){
            if rect.get_intersect_rect(b.get()).is_some(){
                closure(b);
            }
        }

    }
    pub fn for_all_in_rect_mut<A:AxisTrait,N:NodeTrait>(
        tree: &mut DinoTree<A,N>,
        rect: &Rect<N::Num>,
        mut closure: impl FnMut(ProtectedBBox<N::T>),
    ) {
        let mut f = |a: ProtectedBBox<N::T> | {
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

    use crate::colfind::oned::get_section;
    use super::*;
    rect!(Vistr<N>,&N::T,get_section,get);
    
    pub fn for_all_intersect_rect<A:AxisTrait,N:NodeTrait>(
        tree:&DinoTree<A,N>,
        rect: &Rect<N::Num>,
        mut closure: impl FnMut(&N::T),
    ) {
        
        let mut f = |a: &N::T| {
            if rect.get_intersect_rect(a.get()).is_some() {
                closure(a);
            }
        };
        let axis=tree.axis();
        let ta = tree.vistr();
        self::rect_recurse(axis, ta, rect, &mut f);
    }

    pub fn for_all_in_rect<A:AxisTrait,N:NodeTrait>(
        tree:&DinoTree<A,N>,
        rect: &Rect<N::Num>,
        mut closure: impl FnMut(&N::T),
    ) {
        
        let mut f = |a: &N::T| {
            if rect.contains_rect(a.get()) {
                closure(a);
            }
        };

        let axis=tree.axis();
        let ta = tree.vistr();
        self::rect_recurse(axis, ta, rect, &mut f);
    }

}




///Indicates that the user supplied a rectangle
///that intersects with a another one previously queries
///in the session.
#[derive(Debug)]
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
pub struct MultiRectMut<'a,A:AxisTrait,N:NodeTrait> {
    tree:&'a mut DinoTree<A,N>,
    rects: Vec<Rect<N::Num>>,
}

impl<'a,A:AxisTrait,N:NodeTrait> MultiRectMut<'a,A,N>{
    pub fn new(tree:&'a mut DinoTree<A,N>)->Self{
        MultiRectMut{tree,rects:Vec::new()}
    }
    pub fn for_all_in_rect_mut(&mut self,rect:Rect<N::Num>,mut func:impl FnMut(ProtectedBBox<'a,N::T>))->Result<(),RectIntersectErr>{
        for r in self.rects.iter(){
            if rect.get_intersect_rect(r).is_some(){
                return Err(RectIntersectErr);
            }
        }

        self.rects.push(rect);

        for_all_in_rect_mut(self.tree,&rect,|bbox:ProtectedBBox<N::T>|{
            //This is only safe to do because the user is unable to mutate the bounding box.
            let bbox:ProtectedBBox<'a,N::T>=unsafe{core::mem::transmute(bbox)};
            func(bbox);
        });

        Ok(())
    }
}


