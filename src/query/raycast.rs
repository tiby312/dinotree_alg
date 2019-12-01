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

use crate::query::inner_prelude::*;
use core::cmp::Ordering;
use core::convert::TryFrom;

///A Ray.
#[derive(Debug, Copy, Clone)]
pub struct Ray<N> {
    pub point: Vec2<N>,
    pub dir: Vec2<N>,
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

impl<N:Num> Ray<N>{
    fn range_side(&self,axis:impl axgeom::Axis,range:&Range<N>)->Ordering{
        
        let v=if axis.is_xaxis(){
            self.point.x
        }else{
            self.point.y
        };

        range.contains_ext(v)
    }
}



pub enum RayCastResult<'a,T:Aabb>{
    Hit(Vec<PMut<'a,T>>,T::Num),
    NoHit
}
impl<'a,T:Aabb> RayCastResult<'a,T>{
    pub fn unwrap(self)->(Vec<PMut<'a,T>>,T::Num){
        match self{
            RayCastResult::Hit(a,b)=>(a,b),
            RayCastResult::NoHit=>panic!("Ray did not hit.")
        }
    }
}

///Describes if a ray hit a rectangle.
#[derive(Copy, Clone, Debug)]
pub enum RayIntersectResult<N> {
    Hit(N),
    NoHit
}

impl<N> RayIntersectResult<N>{

    pub fn inner_into<K:From<N>>(self)->RayIntersectResult<K>{
        use RayIntersectResult::*;
        match self{
            Hit(k)=>{
                Hit(K::from(k))
            },
            NoHit=>{
                NoHit
            }
        }
    }
    pub fn inner_try_into<K:TryFrom<N>>(self)->Result<RayIntersectResult<K>,K::Error>{
        use RayIntersectResult::*;
        match self{
            Hit(k)=>{
                match K::try_from(k){
                    Ok(k)=>{
                        Ok(Hit(k))
                    },
                    Err(k)=>{
                        Err(k)
                    }
                }
            },
            NoHit=>{
                Ok(NoHit)
            }
        }
    }
}

///This is the trait that defines raycast specific geometric functions that are needed by this raytracing algorithm.
///By containing all these functions in this trait, we can keep the trait bounds of the underlying Num to a minimum
///of only needing Ord.
pub trait RayTrait{
    type N:Num;
    type T:Aabb<Num=Self::N>;

    ///Returns the length of ray between its origin, and where it intersects the line provided.
    ///Returns none if the ray doesnt intersect it.
    ///We use this to further prune nodes.If the closest possible distance of a bot in a particular node is 
    ///bigger than what we've already seen, then we dont need to visit that node.
    //fn compute_distance_to_line<A:Axis>(&mut self,axis:A,line:Self::N)->Option<Self::N>;

    ///The expensive collision detection
    ///This is where the user can do expensive collision detection on the shape
    ///contains within it's bounding box.
    ///Its default implementation just calls compute_distance_to_rect()
    fn compute_distance_to_bot(&self,ray:&Ray<Self::N>,a:&Self::T)->RayIntersectResult<Self::N>{
        self.compute_distance_to_rect(ray,a.get())
    }

    ///Returns true if the ray intersects with this rectangle.
    ///This function allows as to prune which nodes to visit.
    fn compute_distance_to_rect(&self,ray:&Ray<Self::N>,a:&Rect<Self::N>)->RayIntersectResult<Self::N>;
}



//TODO use this.
pub struct RaycastSimple<T:Aabb,F>{
    _p:PhantomData<T>,
    pub func:F
}

impl<T:Aabb,F> RaycastSimple<T,F>
    where F:Fn(&Ray<T::Num>,&Rect<T::Num>) -> RayIntersectResult<T::Num>{

    pub fn new(func:F)->RaycastSimple<T,F>{
        RaycastSimple{_p:PhantomData,func}
    }
}
impl<T:Aabb,F> RayTrait for RaycastSimple<T,F>
    where F:Fn(&Ray<T::Num>,&Rect<T::Num>) -> RayIntersectResult<T::Num>{
    type T=T;
    type N=T::Num;

    fn compute_distance_to_rect(&self,ray:&Ray<Self::N>,a:&Rect<Self::N>)->RayIntersectResult<Self::N>{
        (self.func)(ray,a)
    }
}







fn make_rect_from_range<A:Axis,N:Num>(axis:A,range:&Range<N>,rect:&Rect<N>)->Rect<N>{
    if axis.is_xaxis(){
        Rect{x:*range,y:rect.y}
    }else{
        Rect{x:rect.x,y:*range}
    }
}



     
struct Closest<'a,T:Aabb>{
    closest:Option<(Vec<PMut<'a,T>>,T::Num)>
}
impl<'a,T:Aabb> Closest<'a,T>{
    fn consider<R:RayTrait<N=T::Num,T=T>>(&mut self,ray:&Ray<T::Num>, b:PMut<'a,T>,raytrait:&mut R){

        let x=match raytrait.compute_distance_to_bot(ray,b.as_ref()){
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
                self.closest=Some((vec![b],x))  
            }
        };
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


struct Blap<'a:'b,'b,R:RayTrait>{
    rtrait:&'b mut R,
    ray:Ray<R::N>,
    closest:Closest<'a,R::T>
}
impl<'a:'b,'b,R:RayTrait> Blap<'a,'b,R>{
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
                    }
                }   
                
            },
            RayIntersectResult::NoHit=>{

            }
        }
        false
    } 
}

//Returns the first object that touches the ray.
fn recc<'a:'b,'b,
    A: Axis,
    //T: Aabb,
    N:Node,
    R: RayTrait<N=N::Num,T=N::T>
    >(axis:A,stuff:LevelIter<VistrMut<'a,N>>,rect:Rect<N::Num>,blap:&mut Blap<'a,'b,R>){

    let ((_depth,nn),rest)=stuff.next();
    let nn=nn.get_mut();
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
                    Range{start:*div,end:*div}
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
                            blap.closest.consider(&blap.ray,b,blap.rtrait);
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
                            blap.closest.consider(&blap.ray,b,blap.rtrait);
                        }
                    }


                    if blap.should_handle_rect(&rleft){
                        recc(axis_next,left,rleft,blap);
                    }
                },
                Ordering::Equal=>{
                            
                    if blap.should_handle_rect(&rmiddle){
                        for b in nn.bots.iter_mut(){
                            blap.closest.consider(&blap.ray,b,blap.rtrait);
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
                blap.closest.consider(&blap.ray,b,blap.rtrait);
            } 

        
        }
    }
}


pub use self::mutable::raycast_naive_mut;
pub use self::mutable::raycast_mut;



mod mutable{
    use super::*;

    pub fn raycast_naive_mut<
        'a,
        T:Aabb,
        >(bots:PMut<'a,[T]>,ray:Ray<T::Num>,rtrait:&mut impl RayTrait<N=T::Num,T=T>)->RayCastResult<'a,T>{
        let mut closest=Closest{closest:None};

        for b in bots.iter_mut(){
            closest.consider(&ray,b,rtrait);
        }


        match closest.closest{
            Some((a,b))=>{
                RayCastResult::Hit(a,b)
            },
            None=>{
                RayCastResult::NoHit
            }
        }
    }

    pub fn raycast_mut<
        'a,   
        A:Axis,
        N:Node
        >(tree:&'a mut DinoTree<A,N>,rect:Rect<N::Num>,ray:Ray<N::Num>,rtrait:&mut impl RayTrait<N=N::Num,T=N::T>)->RayCastResult<'a,N::T>{
        
        let axis=tree.axis();
        let dt = tree.vistr_mut().with_depth(Depth(0));


        let closest=Closest{closest:None};
        let mut blap=Blap{rtrait,ray,closest};
        recc(axis,dt,rect,&mut blap);

        match blap.closest.closest{
            Some((a,b))=>{
                RayCastResult::Hit(a,b)
            },
            None=>{
                RayCastResult::NoHit
            }
        }
    }
}