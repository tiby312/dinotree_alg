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
use core::cmp::Ordering;
use core::convert::TryFrom;

use axgeom::primitive_from::PrimitiveFrom;

///A Ray.
#[derive(Debug, Copy, Clone)]
pub struct Ray<N> {
    pub point: Vec2<N>,
    pub dir: Vec2<N>,
}

impl<N:Copy> Ray<N>{
    #[inline(always)]
    pub fn inner_as<B:PrimitiveFrom<N>>(&self)->Ray<B>{
        Ray{point:self.point.inner_as(),dir:self.point.inner_as()}
    }
}
impl<N> Ray<N>{
    

    #[inline(always)]
    pub fn inner_into<B:From<N>>(self)->Ray<B>{
        let point=self.point.inner_into();
        let dir=self.dir.inner_into();
        Ray{point,dir}
    }
    #[inline(always)]
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
    type N:NumTrait;
    type Inner;

    ///Returns the length of ray between its origin, and where it intersects the line provided.
    ///Returns none if the ray doesnt intersect it.
    ///We use this to further prune nodes.If the closest possible distance of a bot in a particular node is 
    ///bigger than what we've already seen, then we dont need to visit that node.
    //fn compute_distance_to_line<A:AxisTrait>(&mut self,axis:A,line:Self::N)->Option<Self::N>;

    ///The expensive collision detection
    ///This is where the user can do expensive collision detection on the shape
    ///contains within it's bounding box.
    ///Its default implementation just calles compute_distance_to_rect()
    fn compute_distance_to_bot(&self,ray:&Ray<Self::N>,a:BBoxRefMut<Self::N,Self::Inner>)->RayIntersectResult<Self::N>{
        self.compute_distance_to_rect(ray,a.rect)
    }

    ///Returns true if the ray intersects with this rectangle.
    ///This function allows as to prune which nodes to visit.
    fn compute_distance_to_rect(&self,ray:&Ray<Self::N>,a:&Rect<Self::N>)->RayIntersectResult<Self::N>;
}



fn vec_make<T>(a:T)->Vec<T>{
    //Cannot use smallvec! creation macro since T does not implement Copy.
    let mut b=Vec::with_capacity(1);
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



     
struct Closest<'a,N:NumTrait,T>{
    closest:Option<(Vec<BBoxRefMut<'a,N,T>>,N)>
}
impl<'a,N:NumTrait,T> Closest<'a,N,T>{
    fn consider<R:RayTrait<N=N,Inner=T>>(&mut self,ray:&Ray<N>,mut b:BBoxRefMut<'a,N,T>,raytrait:&mut R){

        let x=match raytrait.compute_distance_to_bot(ray,b.as_mut()){
            RayIntersectResult::Hit(val)=>{
                val
            },
            RayIntersectResult::NoHit=>{
                return;
            },
        };

        match self.closest.as_mut(){
            Some(mut dis)=>{
                match x.cmp(&dis.1){
                    Ordering::Greater=>{
                        //dis
                        //do nothing.
                    },
                    Ordering::Less=>{
                        dis.0.clear();
                        dis.0.push(b);
                        dis.1=x;
                    },
                    Ordering::Equal=>{
                        dis.0.push(b);
                    }
                }
            },
            None=>{
                self.closest=Some((vec_make(b),x))  
            }
        };
    }

    fn get_dis(&self)->Option<N>{
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


struct Blap<'a,R:RayTrait>{
    rtrait:R,
    ray:Ray<R::N>,
    closest:Closest<'a,R::N,R::Inner>
}
impl<'a,R:RayTrait> Blap<'a,R>{
    fn should_handle_rect(&mut self,rect:&Rect<R::N>)->bool{
        match self.rtrait.compute_distance_to_rect(&self.ray,rect){
            RayIntersectResult::Hit(val)=>{

                match self.closest.get_dis(){
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
}

//Returns the first object that touches the ray.
fn recc<'a,
    N:NumTrait,
    A: AxisTrait,
    T: HasAabbMut<Num=N>,
    R: RayTrait<N=N,Inner=T::Inner>
    >(axis:A,stuff:LevelIter<VistrMut<'a,T>>,rect:Rect<N>,blap:&mut Blap<'a,R>){

    let ((_depth,nn),rest)=stuff.next();
    match rest{
        Some([left,right])=>{
            let axis_next=axis.next();

            let div=match nn.div{
                Some(b)=>b,
                None=>return
            };

            let (rleft,rright) = rect.subdivide(axis,*div);


            let range = &match nn.cont{
                Some(range)=>{
                    *range
                },
                None=>{
                    Range{left:*div,right:*div}
                }
            };


            
            let rmiddle=make_rect_from_range(axis,range,&rect);


            match blap.ray.range_side(axis,range){
                Ordering::Less=>{
                    if blap.should_handle_rect(&rleft){
                        recc(axis_next,left,rleft,blap);
                    }
                   

                    if blap.should_handle_rect(&rmiddle){
                        for b in nn.bots.iter_mut(){
                            blap.closest.consider(&blap.ray,b,&mut blap.rtrait);
                        }
                    }

                    if blap.should_handle_rect(&rright){
                        recc(axis_next,right,rright,blap);
                    }
                },
                Ordering::Greater=>{
                    
                    if blap.should_handle_rect(&rright){
                        recc(axis_next,right,rright,blap);
                    }
                    
                    if blap.should_handle_rect(&rmiddle){
                        for b in nn.bots.iter_mut(){
                            blap.closest.consider(&blap.ray,b,&mut blap.rtrait);
                        }
                    }

                    if blap.should_handle_rect(&rleft){
                        recc(axis_next,left,rleft,blap);
                    }
                },
                Ordering::Equal=>{
                            
                    if blap.should_handle_rect(&rmiddle){
                        for b in nn.bots.iter_mut(){
                            blap.closest.consider(&blap.ray,b,&mut blap.rtrait);
                        }
                    }

                    if blap.should_handle_rect(&rleft){
                        recc(axis_next,left,rleft,blap);
                    }
                    
                    if blap.should_handle_rect(&rright){
                        recc(axis_next,right,rright,blap);
                    }
                }
            }

        },
        None=>{
            //Can't do better here since for leafs, cont is none.
            for b in nn.bots.iter_mut(){
                blap.closest.consider(&blap.ray,b,&mut blap.rtrait);
            } 
        
        }
    }
}


pub use self::mutable::naive_mut;
pub use self::mutable::raycast_mut;



mod mutable{
    use super::*;

    pub fn naive_mut<
        'a,
        T:HasAabbMut,
        >(bots:ElemSliceMut<'a,T>,ray:Ray<T::Num>,mut rtrait:impl RayTrait<N=T::Num,Inner=T::Inner>)->Option<(Vec<BBoxRefMut<'a,T::Num,T::Inner>>,T::Num)>{

        let mut closest=Closest{closest:None};

        for b in bots.iter_mut(){
            closest.consider(&ray,b,&mut rtrait);
        }

        closest.closest
    }
    pub fn raycast_mut<
        'a,   
        K:DinoTreeRefMutTrait
        >(tree:&'a mut K,rect:Rect<K::Num>,ray:Ray<K::Num>,rtrait:impl RayTrait<N=K::Num,Inner=K::Inner>)->Option<(Vec<BBoxRefMut<'a,K::Num,K::Inner>>,K::Num)>{
        
        let axis=tree.axis();
        let dt = tree.vistr_mut().with_depth(Depth(0));


        let closest=Closest{closest:None};
        let mut blap=Blap{rtrait,ray,closest};
        recc(axis,dt,rect,&mut blap);

        blap.closest.closest
    }
}