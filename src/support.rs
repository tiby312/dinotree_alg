use inner_prelude::*;
//use ordered_float::NotNaN;
//use dinotree_inner::support::*;


use ordered_float::NotNaN;


pub use dinotree_inner::support::NumWrapper;
pub use dinotree_inner::support::Numf32;
pub use dinotree_inner::support::Numf64;
pub use dinotree_inner::support::Numisize;
pub use dinotree_inner::support::Numusize;
pub use dinotree_inner::support::BBox;
pub use dinotree_inner::support::DefaultDepthLevel;

//use oned::Blee;
///Find all bots that collide along the specified axis only between two rectangles.
///So the bots may not actually collide in 2d space, but collide alone the x or y axis.
///This is useful when implementing "wrap around" behavior of bots that pass over a rectangular border.
pub fn collide_two_rect_parallel<
    A:AxisTrait,
    Num:NumTrait,
    T:SweepTrait<Num=Num>,
    F:FnMut(ColSingle<T>,ColSingle<T>)>(
        tree:&mut DinoTree<T>,rect1:& AABBox<T::Num>,rect2:& AABBox<T::Num>,mut func:F)
{
    
    struct Ba<'z,J:SweepTrait+Send+'z>(ColSingle<'z,J>);
    impl<'z,J:SweepTrait+Send+'z> SweepTrait for Ba<'z,J>{
        type Inner=J::Inner;
        type Num=J::Num;

        ///Destructure into the bounding box and mutable parts.
        fn get_mut<'a>(&'a mut self)->(&'a AABBox<J::Num>,&'a mut Self::Inner){
            let r=&mut self.0;
            (r.0,r.1)
        }

        ///Destructue into the bounding box and inner part.
        fn get<'a>(&'a self)->(&'a AABBox<J::Num>,&'a Self::Inner){
            let r=&self.0;
            (r.0,r.1)
        }
        
    }
    let mut rects=tree.rects();
    
    let mut buffer1=Vec::new();
    rects.for_all_in_rect(rect1,|a:ColSingle<T>|buffer1.push(Ba(a)));
    
    let mut buffer2=Vec::new();
    rects.for_all_in_rect(rect2,|a:ColSingle<T>|buffer2.push(Ba(a)));
    
    let cols:(&mut [Ba<T>],&mut [Ba<T>])=(&mut buffer1,&mut buffer2);

    {
        sweeper_update::<_,A,par::Parallel>(cols.0);
        sweeper_update::<_,A,par::Parallel >(cols.1);

        use std::marker::PhantomData;
        use oned::Bleek;
        struct Bo<T:SweepTrait,F:FnMut(ColSingle<T>,ColSingle<T>)>(
            F,
            PhantomData<T>
        );

        impl<T:SweepTrait,F:FnMut(ColSingle<T>,ColSingle<T>)> Bleek for Bo<T,F>{
            type T=T;
            fn collide(&mut self,a:ColSingle<Self::T>,b:ColSingle<Self::T>){
                self.0(a,b);
            }
        }
        
        let func2=|a:ColSingle<Ba<T>>,b:ColSingle<Ba<T>>|{
            //let c=ColPair{a:(cc.a.0,cc.a.1),b:(cc.b.0,cc.b.1)};
            let a=ColSingle(a.0,a.1);
            let b=ColSingle(b.0,b.1);
            func(a,b);
        };
        
        let mut sweeper=Sweeper::new();

        let b=Bo(func2,PhantomData);
        sweeper.find_bijective_parallel::<A,_>(cols,b);
    }
}