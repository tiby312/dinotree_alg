//!
//! # Safety
//!
//! There is no unsafe code in this module.
//!
use inner_prelude::*;
use rect::*;

///Find all intersecting pairs between the elements in this dinotree, and the specified elements.
///No intersecting pairs within each group are looked for, only those between the two groups.
///For best performance the group that this tree is built around should be the bigger of the two groups.

pub fn intersect_with_mut<A:axgeom::AxisTrait,T:HasAabb,X:HasAabb<Num=T::Num>>(
    tree:&mut DynTree<A,(),T>,
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