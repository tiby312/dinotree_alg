

//!
//! # Unsafety
//! 
//! There is no unsafe code.
//!
use inner_prelude::*;

///Trait user must implement.
pub trait DividerDrawer{
    type N:NumTrait;
    fn draw_divider<A:AxisTrait>(&mut self,axis:A,div:Self::N,cont:[Self::N;2],length:[Self::N;2],depth:usize);
}

///Calls the user supplied function on each divider.
///Since the leaves do not have dividers, it is not called for the leaves.
pub fn draw<A:AxisTrait,T: HasAabb,D:DividerDrawer<N=T::Num>>(
    gentree: &DynTree<A,(),T>,
    dr:&mut D,
    rect:&Rect<T::Num>
) {
    fn recc<A:AxisTrait,T:HasAabb,D:DividerDrawer<N=T::Num>>
        (axis:A,stuff:LevelIter<NdIter<(),T>>,dr:&mut D,rect:&Rect<T::Num>){

        let ((depth,_nn),rest)=stuff.next();

        match rest{
            Some((extra,left,right))=>{
                let &FullComp{div,cont}=match extra{
                    Some(d)=>d,
                    None=>return
                };

                let cont=[cont.left,cont.right];
                let rr=rect.get_range(axis.next());
                dr.draw_divider::<A>(axis,div,cont,[rr.left,rr.right],depth.0);


                let (a,b)=rect.subdivide(axis,div);

                recc(axis.next(),left,dr,&a);
                recc(axis.next(),right,dr,&b);
            },
            None=>{

            }
        }
    }

    recc(gentree.get_axis(),gentree.get_iter().with_depth(Depth(0)),dr,rect);
    
}
