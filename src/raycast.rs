use inner_prelude::*;
use smallvec::SmallVec;

#[derive(Debug,Copy,Clone)]
pub struct Ray<N>{
    pub point:[N;2],
    pub dir:[N;2],
    pub tlen:N,
}




pub trait RayTrait{
    type T:HasAabb<Num=Self::N>;
    type N:NumTrait;

    //Returns the y range of the fat line that needs to be checked
    fn compute_intersection_range<A:AxisTrait>(&mut self,axis:A,fat_line:[Self::N;2])->Option<(Self::N,Self::N)>;

    //Returns distance from ray origin to the line.
    fn compute_distance_to_line<A:AxisTrait>(&mut self,axis:A,line:Self::N)->Option<Self::N>;

    //The expensive collision detection
    fn compute_distance_bot(&mut self,&Self::T)->Option<Self::N>;

    fn intersects_rect(&self,&Rect<Self::N>)->bool;
    fn divider_side(&self,axis:impl AxisTrait,div:&Self::N)->std::cmp::Ordering;
}


fn new_smallvec<T>(a:T)->SmallVec<[T;2]>{
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
            A: AxisTrait+'a,
            T: HasAabb<Num=N>+'a,
            R: RayTrait<T=T,N=N>
            >(axis:A,stuff:LevelIter<$iterator>,rtrait:&mut R,rect:Rect<N>,closest:&mut Closest<'a,T>){

            let ((_depth,nn),rest)=stuff.next();
            match rest{
                Some((extra,left,right))=>{
                    let &FullComp{div,cont}=match extra{
                        Some(b)=>b,
                        None=>return
                    };
                    
                    let (rleft,rright) = rect.subdivide(axis,div);

                    let axis_next=axis.next();

                    let (first,second)=match rtrait.divider_side(axis,&div){
                        std::cmp::Ordering::Less=>{

                            ((right,rright),(left,rleft))
                        },
                        std::cmp::Ordering::Greater=>{

                            ((left,rleft),(right,rright))
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
                    }      
                
                    //Check the bots in this node only after recursing children.
                    //We recurse first since this way we might find out we dont need to recurse the bots in this node
                    //since its more likely that we will find the closest bot in a child node
                    
                    let ff=[cont.left,cont.right];
                    
                    
                    match rtrait.compute_intersection_range(axis,ff){
                        Some((a,b))=>{
                            for bot in $get_iter!(nn.range){
                                let rang=*bot.get().get_range(axis.next());
                                if rang.left>b{
                                    break;
                                }
                                if rang.right>=a{
                                    closest.consider(bot,rtrait);
                                }
                            }
                        },
                        None=>{
                            //Do nothing
                        }
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
    raycast!(NdIterMut<'a,(),T>,*mut T,&mut T,get_mut_range_iter,NonLeafDynMut,&'a mut T);

    pub fn naive_mut<
        'a,A:AxisTrait,
        T:HasAabb,
        R:RayTrait<T=T,N=T::Num>
        >(bots:&'a mut [T],mut rtrait:R)->Option<(SmallVec<[&'a mut T;2]>,T::Num)>{

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
        R:RayTrait<T=T,N=T::Num>
        >(tree:&'a mut DynTree<A,(),T>,rect:Rect<T::Num>,mut rtrait:R)->Option<(SmallVec<[&'a mut T;2]>,T::Num)>{
        
        let axis=tree.get_axis();
        let dt = tree.get_iter_mut().with_depth(Depth(0));


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
        R:RayTrait<T=T,N=T::Num>
        >(bots:impl Iterator<Item=&'a T>,mut rtrait:R)->Option<(SmallVec<[&'a T;2]>,T::Num)>{

        let mut closest=Closest{closest:None};

        for b in bots{
            closest.consider(b,&mut rtrait);
        }
        closest.closest
    }

    raycast!(NdIter<'a,(),T>,*const T,&T,get_range_iter,NonLeafDyn,&'a T);
    pub fn raycast<
        'a,A:AxisTrait,
        T:HasAabb,
        R:RayTrait<T=T,N=T::Num>
        >(tree:&'a DynTree<A,(),T>,rect:Rect<T::Num>,mut rtrait:R)->Option<(SmallVec<[&'a T;2]>,T::Num)>{
        
        let axis=tree.get_axis();
        let dt = tree.get_iter().with_depth(Depth(0));


        let mut closest=Closest{closest:None};
        recc(axis,dt,&mut rtrait,rect,&mut closest);
        closest.closest
    }
}

