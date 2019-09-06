//! Provides two basic functions: for_all_in_rect, for_all_intersect_rect that both have similar api.
//!
//!
//! for_all_in_rect allows the user to retreive references to all bots whose aabb's are strictly inside of the specified rectangle.
//! for_all_intersect_rect is similar, but will return all bots who are inside as well as all bots whose aabb's intersect the rect.
//! The user is allowed to hold on to the mutable references returned for as long as the tree itself is mutably borrowed.
//!
//! # Safety
//!
//! There is no unsafe code in this module.
//!
use crate::inner_prelude::*;



macro_rules! rect{
    ($iterator:ty,$colsingle:ty,$get_section:ident)=>{     
        fn rect_recurse<
            A: AxisTrait,
            T: HasAabbMut,
            F: FnMut($colsingle),
        >(
            this_axis: A,
            m: $iterator,
            rect: &Rect<T::Num>,
            func: &mut F
        ) {
            

            let (nn,rest)=m.next();
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

                    if !(*div < rr.left) {
                        self::rect_recurse(this_axis.next(), left, rect, func);
                    }
                    if !(*div > rr.right) {
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


fn for_all_not_in_rect_mut<K:DinoTreeRefMutTrait>(tree:&mut K,rect:&Rect<K::Num>,closure:impl FnMut(BBoxRefMut<K::Num,K::Inner>)){
    fn rect_recurse<A:AxisTrait,T:HasAabbMut,F:FnMut(BBoxRefMut<T::Num,T::Inner>)>(axis:A,it:VistrMut<T>,rect:&Rect<T::Num>,mut closure:F)->F{
        let (nn,rest)=it.next();
        
        //TODO exploit sorted property.
        for a in nn.bots.iter_mut(){
            if !rect.contains_rect(a.rect){
                closure(a);
            }
        }
        
        match rest{
            Some([left,right])=>{
                
                let div=match nn.div{
                    Some(b)=>b,
                    None=>return closure,
                };

                match rect.get_range(axis).left_or_right_or_contain(div){
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





pub(crate) use self::mutable::for_all_intersect_rect_mut;
pub(crate) use self::mutable::for_all_in_rect_mut;
pub(crate) use self::constant::for_all_intersect_rect;
pub(crate) use self::constant::for_all_in_rect;

pub use self::mutable::naive_for_all_intersect_rect_mut;
pub use self::mutable::naive_for_all_in_rect_mut;



pub struct RectQueryBuilder<'a,K:DinoTreeRefTrait>{
    tree:&'a mut K,
    rect:Rect<K::Num>
}
impl<'a,K:DinoTreeRefTrait>  RectQueryBuilder<'a,K>{
    pub fn new(tree:&'a mut K,rect:Rect<K::Num>)->Self{
        RectQueryBuilder{tree,rect}
    }
    pub fn for_all_in(&self,mut closure: impl Fn(BBoxRef<K::Num,K::Inner>)){
        constant::for_all_in_rect(self.tree,&self.rect,closure);
    }
    pub fn for_all_intersect(&self,mut closure: impl Fn(BBoxRef<K::Num,K::Inner>)){
        constant::for_all_intersect_rect(self.tree,&self.rect,closure);
    }

}
impl<'a,K:DinoTreeRefMutTrait>  RectQueryBuilder<'a,K>{
    pub fn for_all_in_mut(&mut self, mut closure: impl FnMut(BBoxRefMut<K::Num,K::Inner>)){
        mutable::for_all_in_rect_mut(self.tree,&self.rect,closure);
    }
    pub fn for_all_intersect_mut(&mut self, mut closure: impl FnMut(BBoxRefMut<K::Num,K::Inner>)){
        mutable::for_all_intersect_rect_mut(self.tree,&self.rect,closure);
    }
    pub fn for_all_not_in_mut(&mut self, mut closure: impl FnMut(BBoxRefMut<K::Num,K::Inner>)){
        for_all_not_in_rect_mut(self.tree,&self.rect,closure);
    }
}



mod mutable{
    use crate::colfind::oned::get_section_mut;
    use super::*;

    rect!(VistrMut<T>,BBoxRefMut<T::Num,T::Inner>,get_section_mut);
    pub fn for_all_intersect_rect_mut<K:DinoTreeRefMutTrait>(
        tree: &mut K,
        rect: &Rect<K::Num>,
        mut closure: impl FnMut(BBoxRefMut<K::Num,K::Inner>),
    ) {
        let mut f = |a: BBoxRefMut<K::Num,K::Inner>| {
            if rect.get_intersect_rect(a.rect).is_some() {
                closure(a);
            }
        };

        let axis=tree.axis();
        let ta = tree.vistr_mut();

        self::rect_recurse(axis, ta, rect, &mut f);
    }

    pub fn naive_for_all_in_rect_mut<T: HasAabbMut>(
        bots: &mut ElemSlice<T>,
        rect: &Rect<T::Num>,
        mut closure: impl FnMut(BBoxRefMut<T::Num,T::Inner>),
    ) {
        for b in bots.iter_mut(){
            if rect.contains_rect(b.rect){
                closure(b);
            }
        }

    }

    pub fn naive_for_all_intersect_rect_mut<T: HasAabbMut>(
        bots: &mut ElemSlice<T>,
        rect: &Rect<T::Num>,
        mut closure: impl FnMut(BBoxRefMut<T::Num,T::Inner>),
    ) {
        for b in bots.iter_mut(){
            if rect.get_intersect_rect(b.rect).is_some(){
                closure(b);
            }
        }

    }
    pub fn for_all_in_rect_mut<K:DinoTreeRefMutTrait>(
        mut tree: &mut K,
        rect: &Rect<K::Num>,
        mut closure: impl FnMut(BBoxRefMut<K::Num,K::Inner>),
    ) {
        let mut f = |a: BBoxRefMut<K::Num,K::Inner> | {
            if rect.contains_rect(a.rect) {
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
    rect!(Vistr<T>,BBoxRef<T::Num,T::Inner>,get_section);
    
    pub fn for_all_intersect_rect<K:DinoTreeRefTrait>(
        tree:&K,
        rect: &Rect<K::Num>,
        mut closure: impl FnMut(BBoxRef<K::Num,K::Inner>),
    ) {
        
        let mut f = |a: BBoxRef<K::Num,K::Inner>| {
            if rect.get_intersect_rect(a.rect).is_some() {
                closure(a);
            }
        };
        let axis=tree.axis();
        let ta = tree.vistr();
        self::rect_recurse(axis, ta, rect, &mut f);
    }

    pub fn for_all_in_rect<K:DinoTreeRefTrait>(
        tree:&K,
        rect: &Rect<K::Num>,
        mut closure: impl FnMut(BBoxRef<K::Num,K::Inner>),
    ) {
        
        let mut f = |a: BBoxRef<K::Num,K::Inner>| {
            if rect.contains_rect(a.rect) {
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
pub struct MultiRectMut<'a,K:DinoTreeRefMutTrait> {
    tree:&'a mut K,
    rects: Vec<Rect<K::Num>>,
}

impl<'a,K:DinoTreeRefMutTrait> MultiRectMut<'a,K>{
    pub fn new(tree:&'a mut K)->Self{
        MultiRectMut{tree,rects:Vec::new()}
    }
    pub fn for_all_in_rect_mut(&mut self,rect:Rect<K::Num>,mut func:impl FnMut(BBoxRefMut<'a,K::Num,K::Inner>))->Result<(),RectIntersectErr>{
        for r in self.rects.iter(){
            if rect.get_intersect_rect(r).is_some(){
                return Err(RectIntersectErr);
            }
        }

        self.rects.push(rect);

        for_all_in_rect_mut(self.tree,&rect,|bbox:BBoxRefMut<K::Num,K::Inner>|{
            //This is only safe to do because the user is unable to mutate the bounding box.
            let bbox:BBoxRefMut<'a,K::Num,K::Inner>=unsafe{core::mem::transmute(bbox)};
            func(bbox);
        });

        Ok(())
    }
}

