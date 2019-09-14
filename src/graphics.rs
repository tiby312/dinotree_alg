use crate::inner_prelude::*;

///Trait user must implement.
pub trait DividerDrawer{
    type N:NumTrait;
    fn draw_divider<A:AxisTrait>(&mut self,axis:A,div:Self::N,cont:[Self::N;2],length:[Self::N;2],depth:usize);
}



///Calls the user supplied function on each divider.
///Since the leaves do not have dividers, it is not called for the leaves.
pub fn draw<A:AxisTrait,N:NodeTrait,D:DividerDrawer<N=N::Num>>(
    gentree: &DinoTree<A,N>,
    dr:&mut D,
    rect:&Rect<N::Num>
) {
    fn recc<A:AxisTrait,N:NodeTrait,D:DividerDrawer<N=N::Num>>
        (axis:A,stuff:LevelIter<Vistr<N>>,dr:&mut D,rect:&Rect<N::Num>){

        let ((depth,nn),rest)=stuff.next();
        let nn=nn.get();
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
