//!
//! # User guide
//!
//! There are four flavors of the same fundamental raycast api provided in this module.
//! There is a naive version, and there is a version that uses the tree, and there are mutable versions of those 
//! that return mutable references.
//!
//! 
//! In addition to the tree, the user provides the geometric functions needed by passing an implementation of RayTrait.
//! The user must also provide a rectangle within which all objects that the user is interested in possibly
//! being hit by the raycast must include. 
//!
//! What is returned is the distance to where the ray cast stopped, plus a list of all bots at that distance. 
//! In most cases, only one object is returned, but in the cases where they are ties more can be returned. 
//! All possible solutions are returned since it would be hard to define which of the tied objects would be returned.
//! So the Option returns Some() if and only if the list returned has atleast one element in it.
//!
//! # Notes

//! At first the algorithm worked by splitting the ray into two where the ray intersected the divider.
//! So one ray would have the same origin point, and the other would have the point at which the ray
//! intersected the divder as the origin point. The problem with this is that there might not be a clean solution
//! to the new point of the second ray. The point that you compute may not lie exactly on a point along the ray. 
//!
//! With real numbers this isnt a problem. There would always be a solution. But real numbers don't exist
//! in the real world. Floating points will be close, but not perfect. If you are using integers, the corner case problems
//! are more apparent.
//! 
//! The solution instead was to never subdivide the ray. Its always the same. Instead, keep subdividing the area into rectangles.
//!
//! Why does the user have to provide a finite rectangle up front? The reason is implementation simplicity/performance.
//! By doing this, we don't have to special case the nodes along the outside of the tree.
//! We also don't have have to worry about overflow and underflow problems of providing a rectangle that 
//! just barely fits into the number type.
//! 
//! # Safety
//!
//! There is no unsafety in this module.
//!
//!

use crate::inner_prelude::*;
use smallvec::SmallVec;
use core::cmp::Ordering;
use core::convert::TryFrom;
use core::fmt::Debug;

///A Ray.
#[derive(Debug, Copy, Clone)]
pub struct Ray<N> {
    pub point: Vec2<N>,
    pub dir: Vec2<N>,
}

impl<N> Ray<N>{

    pub fn inner_into<B:From<N>>(self)->Ray<B>{
        let point=self.point.inner_into();
        let dir=self.dir.inner_into();
        Ray{point,dir}
    }
    pub fn inner_try_into<B:TryFrom<N>>(self)->Result<Ray<B>,B::Error>{
        let point=self.point.inner_try_into();
        let dir=self.dir.inner_try_into();
        match(point,dir){
            (Ok(point),Ok(dir))=>{
                Ok(Ray{point,dir})
            },
            (Err(e),Ok(_))=>{
                Err(e)
            },
            (Ok(_),Err(e))=>{
                Err(e)
            },
            (Err(e),Err(_))=>{
                Err(e)
            }
        }
    }
}
impl<N:NumTrait> Ray<N>{


    fn divider_side(&self,axis:impl axgeom::AxisTrait,div:&N)->Ordering{
        if axis.is_xaxis(){
            self.point.x.cmp(div)
        }else{
            self.point.y.cmp(div)
        }
    }

    fn range_side(&self,axis:impl axgeom::AxisTrait,range:&Range<N>)->Ordering{
        if axis.is_xaxis(){
            range.left_or_right_or_contain(&self.point.x)
        }else{
            range.left_or_right_or_contain(&self.point.y)
        }
    }

}



///Describes if a ray hit a rectangle.
#[derive(Copy, Clone, Debug)]
pub enum RayIntersectResult<N> {
    Hit(N),
    NoHit
}


///This is the trait that defines raycast specific geometric functions that are needed by this raytracing algorithm.
///By containing all these functions in this trait, we can keep the trait bounds of the underlying NumTrait to a minimum
///of only needing Ord.
pub trait RayTrait{
    type T:HasAabb<Num=Self::N>+Debug;
    type N:NumTrait;

    ///Returns the length of ray between its origin, and where it intersects the line provided.
    ///Returns none if the ray doesnt intersect it.
    ///We use this to further prune nodes.If the closest possible distance of a bot in a particular node is 
    ///bigger than what we've already seen, then we dont need to visit that node.
    //fn compute_distance_to_line<A:AxisTrait>(&mut self,axis:A,line:Self::N)->Option<Self::N>;

    ///The expensive collision detection
    ///This is where the user can do expensive collision detection on the shape
    ///contains within it's bounding box.
    ///Its default implementation just calles compute_distance_to_rect()
    fn compute_distance_to_bot(&mut self,ray:&Ray<Self::N>,a:&Self::T)->RayIntersectResult<Self::N>{
        self.compute_distance_to_rect(ray,a.get())
    }

    ///Returns true if the ray intersects with this rectangle.
    ///This function allows as to prune which nodes to visit.
    fn compute_distance_to_rect(&mut self,ray:&Ray<Self::N>,a:&Rect<Self::N>)->RayIntersectResult<Self::N>;
}


fn new_smallvec<T>(a:T)->SmallVec<[T;2]>{
    //Cannot use smallvec! creation macro since T does not implement Copy.
    let mut b=SmallVec::new();
    b.push(a);
    b
}


fn make_rect_from_range<A:AxisTrait,N:NumTrait>(axis:A,range:&Range<N>,rect:&Rect<N>)->Rect<N>{
    if axis.is_xaxis(){
        Rect{x:*range,y:rect.y}
    }else{
        Rect{x:rect.x,y:*range}
    }
}





macro_rules! raycast{
    ($iterator:ty,$ptr:ty,$ref:ty,$get_iter:ident,$nonleaf:ident,$ref_lifetime:ty)=>{
        pub use std::dbg;

        fn should_handle_rect<R:RayTrait>(closest:&mut Closest<R::T>,rect:&Rect<R::N>,ray:&Ray<R::N>,rtrait:&mut R)->bool{
            match rtrait.compute_distance_to_rect(ray,rect){
                RayIntersectResult::Hit(val)=>{

                    match closest.get_dis(){
                        Some(dis)=>{
                            if val<=dis{
                                return true;
                            }        
                        },
                        None=>{
                            return true;
                            //recc(axis_next,second.0,rtrait,second.1,closest);
                        }
                    }   
                    
                },
                RayIntersectResult::NoHit=>{

                }
            }

            return false;
        } 

        #[derive(Debug)]
        struct Closest<'a,T:HasAabb+Debug+'a>{
            closest:Option<(SmallVec<[$ref_lifetime;2]>,T::Num)>
        }
        impl<'a,T:HasAabb+Debug+'a> Closest<'a,T>{
            fn consider<R:RayTrait<T=T,N=T::Num>>(&mut self,ray:&Ray<T::Num>,b:$ref_lifetime,raytrait:&mut R){

                let x=match raytrait.compute_distance_to_bot(ray,b){
                    RayIntersectResult::Hit(val)=>{
                        val
                    },
                    RayIntersectResult::NoHit=>{
                        return;
                    },
                };

                let diff=match self.closest.take(){
                    Some(mut dis)=>{
                        match x.cmp(&dis.1){
                            Ordering::Greater=>{
                                dis
                                //do nothing.
                            },
                            Ordering::Less=>{
                                
                                //TODO clear instead of remaking vec??
                                (new_smallvec(b),x)
                            },
                            Ordering::Equal=>{
                                dis.0.push(b);
                                dis
                            }
                        }
                    },
                    None=>{  
                        (new_smallvec(b),x)
                    }
                };
                self.closest=Some(diff);

            }

            fn get_dis(&self)->Option<T::Num>{
                match &self.closest{
                    Some(x)=>{
                        Some(x.1)
                    },
                    None=>{
                        None
                    }
                }
            }
        }


        //Returns the first object that touches the ray.
        fn recc<'a,
            N:NumTrait+'a,
            A: AxisTrait,
            T: HasAabb<Num=N>+Debug+'a,
            R: RayTrait<T=T,N=N>
            >(axis:A,stuff:LevelIter<$iterator>,rtrait:&mut R,rect:Rect<N>,ray:&Ray<N>,closest:&mut Closest<'a,T>){

            //dbg!(rect,ray,&closest);

            let ((_depth,nn),rest)=stuff.next();
            match rest{
                Some([left,right])=>{
                    let axis_next=axis.next();

                    let div=match nn.div{
                        Some(b)=>b,
                        None=>return
                    };

                    let (rleft,rright) = rect.subdivide(axis,*div);


                    let range = match nn.cont{
                        Some(range)=>{
                            range
                        },
                        None=>{
                            let (first,second)=match ray.divider_side(axis,div){
                                Ordering::Less=>{
                                    ((rleft,left),(rright,right))
                                },
                                _=>{
                                    ((rright,right),(rleft,left))
                                }
                            };
                            
                            if should_handle_rect(closest,&first.0,ray,rtrait){
                                recc(axis_next,first.1,rtrait,first.0,ray,closest);
                            }
                            if should_handle_rect(closest,&second.0,ray,rtrait){
                                recc(axis_next,second.1,rtrait,second.0,ray,closest);   
                            }

                            return;
                        }
                    };


                    
                    let rmiddle=make_rect_from_range(axis,range,&rect);


                    match ray.range_side(axis,range){
                        Ordering::Less=>{
                            if should_handle_rect(closest,&rleft,ray,rtrait){
                                recc(axis_next,left,rtrait,rleft,ray,closest);
                            }
                           

                            if should_handle_rect(closest,&rmiddle,ray,rtrait){
                                for b in $get_iter!(nn.bots){
                                    closest.consider(ray,b,rtrait);
                                }
                            }

                            if should_handle_rect(closest,&rright,ray,rtrait){
                                recc(axis_next,right,rtrait,rright,ray,closest);
                            }
                        },
                        Ordering::Greater=>{
                            
                            if should_handle_rect(closest,&rright,ray,rtrait){
                                recc(axis_next,right,rtrait,rright,ray,closest);
                            }
                            
                            if should_handle_rect(closest,&rmiddle,ray,rtrait){
                                for b in $get_iter!(nn.bots){
                                    closest.consider(ray,b,rtrait);
                                }
                            }

                            if should_handle_rect(closest,&rleft,ray,rtrait){
                                recc(axis_next,left,rtrait,rleft,ray,closest);
                            }
                        },
                        Ordering::Equal=>{
                                    
                            if should_handle_rect(closest,&rmiddle,ray,rtrait){
                                for b in $get_iter!(nn.bots){
                                    closest.consider(ray,b,rtrait);
                                }
                            }

                            if should_handle_rect(closest,&rleft,ray,rtrait){
                                recc(axis_next,left,rtrait,rleft,ray,closest);
                            }
                            
                            if should_handle_rect(closest,&rright,ray,rtrait){
                                recc(axis_next,right,rtrait,rright,ray,closest);
                            }
                        }
                    }

                },
                None=>{
                    //TODO remove this check
                    //if should_handle_rect(closest,&rect,ray,rtrait){
                        //Can't do better here since for leafs, cont is none.
                        for b in $get_iter!(nn.bots){
                            closest.consider(ray,b,rtrait);
                        } 
                    //}
                }
            }
        }
    }
}


macro_rules! get_range_iter{
    ($range:expr)=>{{
        $range.iter()
    }}
}


macro_rules! get_mut_range_iter{
    ($range:expr)=>{{
        $range.iter_mut()
    }}
}


pub use self::mutable::naive_mut;
pub use self::mutable::raycast_mut;



mod mutable{
    use super::*;
    raycast!(VistrMut<'a,T>,*mut T,&mut T,get_mut_range_iter,NonLeafDynMut,&'a mut T);

    pub fn naive_mut<
        'a,A:AxisTrait,
        T:HasAabb+Debug,
        >(bots:&'a mut [T],ray:&Ray<T::Num>,mut rtrait:impl RayTrait<T=T,N=T::Num>)->Option<(SmallVec<[&'a mut T;2]>,T::Num)>{

        let mut closest=Closest{closest:None};

        for b in bots.iter_mut(){
            closest.consider(ray,b,&mut rtrait);
        }

        closest.closest
    }
    //RaycastResultMut<'a,T,T::Num>
    pub fn raycast_mut<
        'a,   
        K:DinoTreeRefMutTrait
        >(tree:&'a mut K,rect:Rect<K::Num>,ray:&Ray<K::Num>,mut rtrait:impl RayTrait<T=K::Item,N=K::Num>)->Option<(SmallVec<[&'a mut K::Item;2]>,K::Num)>
            where <K as dinotree::DinoTreeRefTrait>::Item: std::fmt::Debug{
        
        let axis=tree.axis();
        let dt = tree.vistr_mut().with_depth(Depth(0));


        let mut closest=Closest{closest:None};
        recc(axis,dt,&mut rtrait,rect,ray,&mut closest);

        closest.closest
    }
}

pub use self::cons::naive;
pub use self::cons::raycast;
mod cons{
    use super::*;
    pub fn naive<
        'a,
        T:HasAabb+Debug,
        >(bots:impl Iterator<Item=&'a T>,ray:&Ray<T::Num>,mut rtrait:impl RayTrait<T=T,N=T::Num>)->Option<(SmallVec<[&'a T;2]>,T::Num)>{

        let mut closest=Closest{closest:None};

        for b in bots{
            closest.consider(ray,b,&mut rtrait);
        }
        closest.closest
    }

    raycast!(Vistr<'a,T>,*const T,&T,get_range_iter,NonLeafDyn,&'a T);
    pub fn raycast<
        'a,
        //'a,A:AxisTrait+'a,
        //T:HasAabb,
        K:DinoTreeRefTrait
        >(tree:&'a K,rect:Rect<K::Num>,ray:&Ray<K::Num>,mut rtrait:impl RayTrait<T=K::Item,N=K::Num>)->Option<(SmallVec<[&'a K::Item;2]>,K::Num)>
            where <K as dinotree::DinoTreeRefTrait>::Item: std::fmt::Debug{
        

        let axis=tree.axis();
        let dt = tree.vistr().with_depth(Depth(0));


        let mut closest=Closest{closest:None};
        recc(axis,dt,&mut rtrait,rect,ray,&mut closest);
        closest.closest
    }
}

