//!
//! # Safety
//!
//! There is no unsafe code in this module.
//!
use crate::inner_prelude::*;
use crate::rect::*;

///Find all intersecting pairs between the elements in this dinotree, and the specified elements.
///No intersecting pairs within each group are looked for, only those between the two groups.
///For best performance the group that this tree is built around should be the bigger of the two groups.
///Since the dividers of the tree are used to divide and conquer the problem.
///If the other group is bigger, consider building the DinoTree around that group instead, and
///leave this group has a list of bots.
///
///Currently this is implemented naively using for_all_intersect_rect_mut().
///But using the api, it is possible to build up a tree using the current trees dividers
///to exploit the divide and conquer properties of this problem.
///The two trees could be recursed at the same time to break up the problem.
pub fn intersect_with_mut<K:DinoTreeRefMutTrait+Send,X:Copy+Send>(
    mut tree:K,
    b: &mut [X],
    mut aabb_create:impl FnMut(&X)->axgeom::Rect<K::Num>,
    func: impl Fn(BBoxRefMut<K::Num,K::Inner>,BBoxRefMut<K::Num,K::Inner>)+Copy+Send,
) {

    //TODO instead of create just a list of BBox, construct a tree using the dividors of the current tree.
    //This way we can paralleliz this function.
    let mut b2:Vec<_>=b.iter_mut().map(|a|BBoxRefMut::new(aabb_create(&a),a)).collect();

    for mut i in ElemSlice::from_slice_mut(&mut b2).iter_mut() {
        let rect=*i.get();
        for_all_intersect_rect_mut(&mut tree,&rect, |a| {
            func(a,i.as_mut());
        });
    }

    /*
    for (i,j) in b2.iter().zip_eq(b.iter_mut()){
        *j=i.inner;
    }
    */
}