//!
//! # Unsafety
//! 
//! There is no unsafe code.
//!
use crate::inner_prelude::*;

///Trait user must implement.
pub trait DividerDrawer{
    type N:NumTrait;
    fn draw_divider<A:AxisTrait>(&mut self,axis:A,div:Self::N,cont:[Self::N;2],length:[Self::N;2],depth:usize);
}

///Calls the user supplied function on each divider.
///Since the leaves do not have dividers, it is not called for the leaves.
pub fn draw<K:DinoTreeRefTrait,D:DividerDrawer<N=K::Num>>(
    gentree: K,
    dr:&mut D,
    rect:&Rect<K::Num>
) {
    fn recc<A:AxisTrait,T:HasAabbMut,D:DividerDrawer<N=T::Num>>
        (axis:A,stuff:LevelIter<Vistr<T>>,dr:&mut D,rect:&Rect<T::Num>){

        let ((depth,nn),rest)=stuff.next();

        if let Some([left,right]) = rest{
            
            let div=match nn.div{
                Some(d)=>d,
                None=>return
            };

            let cont=match nn.cont{
                Some(d)=>d,
                None=>return
            };

            let cont=[cont.left,cont.right];
            let rr=rect.get_range(axis.next());
            dr.draw_divider::<A>(axis,*div,cont,[rr.left,rr.right],depth.0);


            let (a,b)=rect.subdivide(axis,*div);

            recc(axis.next(),left,dr,&a);
            recc(axis.next(),right,dr,&b);
        
        }
    }

    recc(gentree.axis(),gentree.vistr().with_depth(Depth(0)),dr,rect);
    
}
