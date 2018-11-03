//!
//! # User guide
//!
//! There are four flavors of the same fundamental raycast api provided in this module.
//! There is a naive version, and there is a version that uses the tree, and there are mutable versions of those 
//! that return mutable references.
//!
//! They all look something like this:
//!
//! ```ignore
//! pub fn raycast_mut<'a,A:AxisTrait,T:HasAabb>(
//!              tree:&'a mut DinoTree<A,(),T>,
//!              rect:Rect<T::Num>,
//!              mut rtrait:impl RayTrait<T=T,N=T::Num>)
//!       ->Option<(SmallVec<[&'a mut T;2]>,T::Num)>{
//!    
//! ```
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

use inner_prelude::*;
use smallvec::SmallVec;




///This is the trait that defines raycast specific geometric functions that are needed by this raytracing algorithm.
///By containing all these functions in this trait, we can keep the trait bounds of the underlying NumTrait to a minimum
///of only needing Ord.
pub trait RayTrait{
    type T:HasAabb<Num=Self::N>;
    type N:NumTrait;

    ///Returns the length of ray between its origin, and where it intersects the line provided.
    ///Returns none if the ray doesnt intersect it.
    ///We use this to further prune nodes.If the closest possible distance of a bot in a particular node is 
    ///bigger than what we've already seen, then we dont need to visit that node.
    fn compute_distance_to_line<A:AxisTrait>(&mut self,axis:A,line:Self::N)->Option<Self::N>;

    ///The expensive collision detection
    ///This is where the user can do expensive collision detection on the shape
    ///contains within it's bounding box.
    fn compute_distance_bot(&mut self,&Self::T)->Option<Self::N>;


    ///Returns true if the ray intersects with this rectangle.
    ///This function allows as to prune which nodes to visit.
    fn intersects_rect(&self,&Rect<Self::N>)->bool;

    ///Return the ordering of the divider relative to the ray's origin point.
    ///So if, for the particular axis, the point is less than the divider,
    ///return Less. If they are equal, return equal. If the popint is greater than the divider,
    ///return greater.
    ///This function allows us to determine which children to recurse. We want to recurse
    ///towards the origin of the ray since we want to find things that are closer to the ray first.
    fn divider_side(&self,axis:impl AxisTrait,div:&Self::N)->std::cmp::Ordering;

}


fn new_smallvec<T>(a:T)->SmallVec<[T;2]>{
    //Cannot use smallvec! creation macro since T does not implement Copy.
    
    let mut b=SmallVec::new();
    b.push(a);
    b
}




macro_rules! raycast{
    ($iterator:ty,$ptr:ty,$ref:ty,$get_iter:ident,$nonleaf:ident,$ref_lifetime:ty)=>{
        
        struct Closest<'a,T:HasAabb+'a>{
            closest:Option<(SmallVec<[$ref_lifetime;2]>,T::Num)>
        }
        impl<'a,T:HasAabb+'a> Closest<'a,T>{
            fn consider<R:RayTrait<T=T,N=T::Num>>(&mut self,b:$ref_lifetime,raytrait:&mut R){

                if let Some(x)=raytrait.compute_distance_bot(b){
                    let diff=match self.closest.take(){
                        Some(mut dis)=>{
                            match x.cmp(&dis.1){
                                std::cmp::Ordering::Greater=>{
                                    dis
                                    //do nothing.
                                },
                                std::cmp::Ordering::Less=>{
                                    
                                    //TODO clear instead of remaking vec??
                                    (new_smallvec(b),x)
                                },
                                std::cmp::Ordering::Equal=>{
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
            T: HasAabb<Num=N>+'a,
            R: RayTrait<T=T,N=N>
            >(axis:A,stuff:LevelIter<$iterator>,rtrait:&mut R,rect:Rect<N>,closest:&mut Closest<'a,T>){

            let ((_depth,nn),rest)=stuff.next();
            match rest{
                Some((extra,left,right))=>{
                    let &FullComp{div,cont:_}=match extra{
                        Some(b)=>b,
                        None=>return
                    };
                    
                    let (rleft,rright) = rect.subdivide(axis,div);

                    let axis_next=axis.next();

                    let (first,second)=match rtrait.divider_side(axis,&div){
                        std::cmp::Ordering::Less=>{
                            ((left,rleft),(right,rright))
                        },
                        std::cmp::Ordering::Greater=>{

                            ((right,rright),(left,rleft))
                        },
                        std::cmp::Ordering::Equal=>{ //We might potentially recurse the wrong way unless we recurse both, so recurse both
                            ((left,rleft),(right,rright))
                        }
                    };

                    if rtrait.intersects_rect(&first.1){
                        recc(axis_next,first.0,rtrait,first.1,closest);
                    }


                    if rtrait.intersects_rect(&second.1){
                        
                        match closest.get_dis(){
                            Some(dis)=>{
                                match rtrait.compute_distance_to_line(axis,div){
                                    Some(dis2)=>{
                                        if dis2<=dis{
                                            recc(axis_next,second.0,rtrait,second.1,closest);
                                        }else{
                                            //We get to skip here
                                        }
                                    },
                                    None=>{
                                        //Ray doesnt intersect other side??
                                    }
                                }
                            },
                            None=>{
                                recc(axis_next,second.0,rtrait,second.1,closest);
                            }
                        }
                        
                        //recc(axis_next,second.0,rtrait,second.1,closest);
                          
                    }      
                
                    //Check the bots in this node only after recursing children.
                    //We recurse first since this way we might find out we dont need to recurse the bots in this node
                    //since its more likely that we will find the closest bot in a child node
                    
                    //TODO prune bots here?
                    for b in $get_iter!(nn.range){
                        closest.consider(b,rtrait);
                    }
                
                    
                },
                None=>{
                    //Can't do better here since for leafs, cont is none.
                    for b in $get_iter!(nn.range){
                        closest.consider(b,rtrait);
                    } 
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
    raycast!(VistrMut<'a,(),T>,*mut T,&mut T,get_mut_range_iter,NonLeafDynMut,&'a mut T);

    pub fn naive_mut<
        'a,A:AxisTrait,
        T:HasAabb,
        >(bots:&'a mut [T],mut rtrait:impl RayTrait<T=T,N=T::Num>)->Option<(SmallVec<[&'a mut T;2]>,T::Num)>{

        let mut closest=Closest{closest:None};

        for b in bots.iter_mut(){
            closest.consider(b,&mut rtrait);
        }

        closest.closest
    }
    //RaycastResultMut<'a,T,T::Num>
    pub fn raycast_mut<
        'a,A:AxisTrait,
        T:HasAabb,
        >(tree:&'a mut DinoTree<A,(),T>,rect:Rect<T::Num>,mut rtrait:impl RayTrait<T=T,N=T::Num>)->Option<(SmallVec<[&'a mut T;2]>,T::Num)>{
        
        let axis=tree.axis();
        let dt = tree.vistr_mut().with_depth(Depth(0));


        let mut closest=Closest{closest:None};
        recc(axis,dt,&mut rtrait,rect,&mut closest);

        closest.closest
    }
}

pub use self::cons::naive;
pub use self::cons::raycast;
mod cons{
    use super::*;
    pub fn naive<
        'a,
        T:HasAabb,
        >(bots:impl Iterator<Item=&'a T>,mut rtrait:impl RayTrait<T=T,N=T::Num>)->Option<(SmallVec<[&'a T;2]>,T::Num)>{

        let mut closest=Closest{closest:None};

        for b in bots{
            closest.consider(b,&mut rtrait);
        }
        closest.closest
    }

    raycast!(Vistr<'a,(),T>,*const T,&T,get_range_iter,NonLeafDyn,&'a T);
    pub fn raycast<
        'a,A:AxisTrait,
        T:HasAabb,
        >(tree:&'a DinoTree<A,(),T>,rect:Rect<T::Num>,mut rtrait:impl RayTrait<T=T,N=T::Num>)->Option<(SmallVec<[&'a T;2]>,T::Num)>{
        
        let axis=tree.axis();
        let dt = tree.vistr().with_depth(Depth(0));


        let mut closest=Closest{closest:None};
        recc(axis,dt,&mut rtrait,rect,&mut closest);
        closest.closest
    }
}

