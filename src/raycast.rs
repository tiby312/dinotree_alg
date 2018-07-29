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


    fn split_ray<A:AxisTrait>(&mut self,axis:A,ray1:&Ray<Self::N>,fo:Self::N)->Option<(Ray<Self::N>,Ray<Self::N>)>;

}




macro_rules! raycast{
    ($iterator:ty,$ptr:ty,$ref:ty,$get_iter:ident,$nonleaf:ident)=>{
        
        struct Closest<T:HasAabb>{
            //closest:Option<($ptr,T::Num)>
            closest:Option<(SmallVec<[$ptr;4]>,T::Num)>
        }
        impl<T:HasAabb> Closest<T>{
            fn consider<R:RayTrait<T=T,N=T::Num>>(&mut self,b:$ref,raytrait:&mut R){

                if let Some(x)=raytrait.compute_distance_bot(b){
                    let diff=match self.closest.take(){
                        Some(mut dis)=>{
                            match x.cmp(&dis.1){
                                std::cmp::Ordering::Greater=>{
                                    dis
                                    //do nothing.
                                },
                                std::cmp::Ordering::Less=>{
                                    let mut s=SmallVec::new();
                                    s.push(b as $ptr);
                                    //TODO clear instead of remaking vec??
                                    (s,x)
                                },
                                std::cmp::Ordering::Equal=>{
                                    dis.0.push(b as $ptr);
                                    dis
                                }
                            }
                        },
                        None=>{
                            let mut s=SmallVec::new();
                            s.push(b as $ptr);     
                            (s,x)
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
        fn recc<'x,'a,
            N:NumTrait,
            A: AxisTrait,
            T: HasAabb<Num=N>,
            R: RayTrait<T=T,N=N>
            >(axis:A,stuff:LevelIter<$iterator>,rtrait:&mut R,ray:Ray<N>,closest:&mut Closest<T>){

            let ((_depth,nn),rest)=stuff.next();
            match rest{
                Some((extra,left,right))=>{
                    let &FullComp{div,cont}=match extra{
                        Some(b)=>b,
                        None=>return
                    };
                    {
                        let ray_point=*axgeom::AxisWrapRef(&ray.point).get(axis);

                        let axis_next=axis.next();

                        match ray_point.cmp(&div){
                            std::cmp::Ordering::Less=>{
                                match rtrait.split_ray(axis,&ray,div){
                                    Some((ray_closer,ray_further))=>{
                                        recc(axis_next,left,rtrait,ray_closer,closest);
                                        recc(axis_next,right,rtrait,ray_closer,closest);
                                        
                                        //dop(axis,right,rtrait,ray_further,closest,div);
                                    },
                                    None=>{
                                        //The ray isnt long enough to touch the divider.
                                        //So just recurse the one side.
                                        recc(axis_next,left,rtrait,ray,closest);
                                        
                                        //recc(axis_next,right,rtrait,ray,closest);
                                        
                                    }
                                }
                            },
                            std::cmp::Ordering::Greater=>{
                                match rtrait.split_ray(axis,&ray,div){
                                    Some((ray_closer,ray_further))=>{
                                        recc(axis_next,right,rtrait,ray_closer,closest);
                                        recc(axis_next,left,rtrait,ray_closer,closest);
                                        
                                        //dop(axis,left,rtrait,ray_further,closest,div);
                                    },
                                    None=>{
                                        recc(axis_next,right,rtrait,ray,closest);

                                        //recc(axis_next,left,rtrait,ray,closest);
                                        
                                    }
                                }
                            },
                            std::cmp::Ordering::Equal=>{ //We might potentially recurse the wrong way unless we recurse both, so recurse both
                                recc(axis_next,left,rtrait,ray,closest);
                                recc(axis_next,right,rtrait,ray,closest);   
                            }
                        };             
                    };


                    //Possibly recurse this side if the closest possible ray distance for a bot in this side
                    //of the divider is less than the current closest ray distance found.
                    fn dop<
                        N:NumTrait,
                        A: AxisTrait,
                        T: HasAabb<Num=N>,
                        R: RayTrait<T=T,N=N>
                        >(axis:A,nd:LevelIter<$iterator>,rtrait:&mut R,ray:Ray<N>,closest:&mut Closest<T>,div:N){
                        match closest.get_dis(){
                            Some(dis)=>{
                                match rtrait.compute_distance_to_line(axis,div){
                                    Some(dis2)=>{
                                        if dis2<dis{
                                            recc(axis.next(),nd,rtrait,ray,closest);
                                        }else{
                                            //We get to skip here
                                        }
                                    },
                                    None=>{
                                        //Ray doesnt intersect other side
                                    }
                                }
                            },
                            None=>{
                                recc(axis.next(),nd,rtrait,ray,closest);
                            }
                        }
                    }


                    //Check the bots in this node only after recursing children.
                    //We recurse first since this way we might find out we dont need to recurse the bots in this node
                    //since its more likely that we will find the closest bot in a child node
            
                    let ff=[cont.left,cont.right];

                    let ray_point=*axgeom::AxisWrapRef(&ray.point).get(axis);

                    //TODO figure out correct inequalities
                    let handle_middle=if ray_point>=ff[0] && ray_point<=ff[1]{
                        true
                    }else{

                        let ray_point_wrap=axgeom::AxisWrapRef(&ray.point);
                        let closer_line=if *ray_point_wrap.get(axis)<div{
                            ff[0]
                        }else{
                            ff[1]
                        };

                        match closest.get_dis(){
                            Some(dis)=>{
                                match rtrait.compute_distance_to_line(axis,closer_line){
                                    Some(dis2)=>{
                                        if dis2<dis{
                                            true
                                        }else{
                                            false
                                        }
                                    },
                                    None=>{
                                        true
                                    }
                                }
                            },
                            None=>{
                                true
                            }
                        }
                    };


                    if /*handle_middle*/true{
                        match rtrait.compute_intersection_range(axis,ff){
                            Some((a,b))=>{
                                for bot in $get_iter!(nn.range){
                                    /*
                                    let rang=*bot.get().get_range(axis.next());
                                    if rang.left>b{
                                        break;
                                    }
                                    
                                    if rang.right>=a{
                                        closest.consider(bot,rtrait);
                                    }
                                    */
                                    closest.consider(bot,rtrait);
                                }
                            },
                            None=>{
                                //Do nothing
                                //TODO REMOVE THIS
                                for bot in $get_iter!(nn.range){
                                    closest.consider(bot,rtrait);
                                }
                            }
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


pub struct RaycastResult<'b,T:'b,F>{
    it:std::vec::IntoIter<(&'b T,F)>
}

impl<'b,T:'b,F> ExactSizeIterator for RaycastResult<'b,T,F>{}
unsafe impl<'b,T:'b,F> std::iter::TrustedLen for RaycastResult<'b,T,F>{}

impl<'b,T:'b,F> Iterator for RaycastResult<'b,T,F>{
    type Item=(&'b T,F);
    fn next(&mut self)->Option<Self::Item>{
        self.it.next()
    }
    fn size_hint(&self)->(usize,Option<usize>){
        self.it.size_hint()
    }
}

pub struct RaycastResultMut<'b,T:'b,F>{
    it:std::vec::IntoIter<(&'b mut T,F)>
}

impl<'b,T:'b,F> ExactSizeIterator for RaycastResultMut<'b,T,F>{}
unsafe impl<'b,T:'b,F> std::iter::TrustedLen for RaycastResultMut<'b,T,F>{}

impl<'b,T:'b,F> Iterator for RaycastResultMut<'b,T,F>{
    type Item=(&'b mut T,F);
    fn next(&mut self)->Option<Self::Item>{
        self.it.next()
    }
    fn size_hint(&self)->(usize,Option<usize>){
        self.it.size_hint()
    }
}


mod mutable{
    use super::*;
    raycast!(NdIterMut<(),T>,*mut T,&mut T,get_mut_range_iter,NonLeafDynMut);

    pub fn naive_mut<
        'a,A:AxisTrait,
        T:HasAabb,
        R:RayTrait<T=T,N=T::Num>
        >(bots:&'a mut [T],mut rtrait:R)->RaycastResultMut<'a,T,T::Num>{

        let mut closest=Closest{closest:None};

        for b in bots.iter_mut(){
            closest.consider(b,&mut rtrait);
        }

        
        match closest.closest{
            Some(x)=>{

                let v:Vec<(&'a mut T,T::Num)>=x.0.iter().map(|&a|(unsafe{&mut *a},x.1)).collect();
                RaycastResultMut{it:v.into_iter()}
            },
            None=>{
                let v=Vec::new();
                RaycastResultMut{it:v.into_iter()}
                
            }
        }  

    }
    pub fn raycast_mut<
        'a,A:AxisTrait,
        T:HasAabb,
        R:RayTrait<T=T,N=T::Num>
        >(tree:&'a mut DynTree<A,(),T>,ray:Ray<T::Num>,mut rtrait:R)->RaycastResultMut<'a,T,T::Num>{
        
        let axis=tree.get_axis();
        let dt = tree.get_iter_mut().with_depth(Depth(0));


        let mut closest=Closest{closest:None};
        recc(axis,dt,&mut rtrait,ray,&mut closest);

        match closest.closest{
            Some(x)=>{

                let v:Vec<(&'a mut T,T::Num)>=x.0.iter().map(|&a|(unsafe{&mut *a},x.1)).collect();
                RaycastResultMut{it:v.into_iter()}
            },
            None=>{
                let v=Vec::new();
                RaycastResultMut{it:v.into_iter()}
                
            }
        }    
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
        >(bots:impl Iterator<Item=&'a T>,mut rtrait:R)->RaycastResult<'a,T,T::Num>{

        let mut closest=Closest{closest:None};

        for b in bots{
            closest.consider(b,&mut rtrait);
        }

        match closest.closest{
            Some(x)=>{

                let v:Vec<(&'a T,T::Num)>=x.0.iter().map(|&a|(unsafe{& *a},x.1)).collect();
                RaycastResult{it:v.into_iter()}
            },
            None=>{
                let v=Vec::new();
                RaycastResult{it:v.into_iter()}
                
            }
        }  

    }
    raycast!(NdIter<(),T>,*const T,&T,get_range_iter,NonLeafDyn);
    pub fn raycast<
        'a,A:AxisTrait,
        T:HasAabb,
        R:RayTrait<T=T,N=T::Num>
        >(tree:&'a DynTree<A,(),T>,ray:Ray<T::Num>,mut rtrait:R)->RaycastResult<'a,T,T::Num>{
        
        let axis=tree.get_axis();
        let dt = tree.get_iter().with_depth(Depth(0));


        let mut closest=Closest{closest:None};
        recc(axis,dt,&mut rtrait,ray,&mut closest);

        match closest.closest{
            Some(x)=>{

                let v:Vec<(&'a T,T::Num)>=x.0.iter().map(|&a|(unsafe{& *a},x.1)).collect();
                RaycastResult{it:v.into_iter()}
            },
            None=>{
                let v=Vec::new();
                RaycastResult{it:v.into_iter()}
                
            }
        }
 
    }
}

