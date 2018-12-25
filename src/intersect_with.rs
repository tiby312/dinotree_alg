//!
//! # Safety
//!
//! There is no unsafe code in this module.
//!
use crate::inner_prelude::*;
use crate::rect::*;

/*
///Find all intersecting pairs between the elements in this dinotree, and the specified elements.
///No intersecting pairs within each group are looked for, only those between the two groups.
///For best performance the group that this tree is built around should be the bigger of the two groups.
pub fn intersect_with_mut<A:axgeom::AxisTrait,T:HasAabb,X:HasAabb<Num=T::Num>>(
    tree:&mut DinoTree<A,(),T>,
    b: &mut [X],
    mut func: impl FnMut(&mut T, &mut X),
) {
    //todo find better algorithn?
    //todo do this before axis specialization?
    //ideally you'd bin the new group using the existing dividers and query that.
    
    for i in b.iter_mut() {
        let rect=*i.get();
        for_all_intersect_rect_mut(tree,&rect, |a: &mut T| {
            func(a,i);
        });
    }
}
*/


///Find all intersecting pairs between the elements in this dinotree, and the specified elements.
///No intersecting pairs within each group are looked for, only those between the two groups.
///For best performance the group that this tree is built around should be the bigger of the two groups.
///Since the dividers of the tree are used to divide and conquer the problem.
///If the other group is bigger, consider building the DinoTree around that group instead, and
///leave this group has a list of bots.
///
///TODO:
///
///Currently this is implemented naively using for_all_intersect_rect_mut().
///But using the api, it is possible to build up a tree using the current trees dividers
///to exploirt the divide and conquer properties of this problem.
///This would be a bit tricky to implement since the tree heights might be different.
///But without changing this api, better performance can be achieved.
pub fn intersect_with_mut<A:axgeom::AxisTrait,T:HasAabb+Send,X:Copy+Send>(
    mut tree:DinoTreeRefMut<A,T>,
    b: &mut [X],
    mut aabb_create:impl FnMut(&X)->axgeom::Rect<T::Num>,
    func: impl Fn(&mut T, &mut BBox<T::Num,X>)+Copy+Send,
) {

    //TODO instead of create just a list of BBox, construct a tree using the dividors of the current tree.
    //This way we can paralleliz this function.


    let mut b2:Vec<_>=unsafe {
        b.iter_mut().map(|a|BBox::new(aabb_create(a),*a)).collect()
    };

    for i in b2.iter_mut() {
        let rect=*i.get();
        for_all_intersect_rect_mut(tree.as_ref_mut(),&rect, |a: &mut T| {
            func(a,i);
        });
    }


    for (i,j) in b2.iter().zip_eq(b.iter_mut()){
        *j=i.inner;
    }
}