//!
//! # User Guide
//!
//! There are four flavors of the same fundamental knearest api provided in this module.
//! There is a naive version, and there is a version that uses the tree, and there are mutable versions of those 
//! that return mutable references.
//!
//! Along with a reference to the tree, the user provides the needed geometric functions by passing an implementation of Knearest.
//! The user provides a point, and the number of nearest objects to return. Then an iterator containing up to that number of units is returned. 
//! A unit is a distance plus one or bots. This is to handle solutions where there is a tie. There may be multiple nearest elements.
//! The first element returned is the closest, and the last the furtheset.
//! It is possible for the vec to be empty if the tree does not contain any bots. 
//! All bots are returned for ties since it is hard to define exactly which bot would be returned by this algorithm otherwise.
//! This also means that the orderding of the bots inside of a Unit has no meaning and could be returned in any order.
//! For trees that use floating point bounding boxes, ties will be extremely rare in a lot of cases, so each Unit
//! will likely only have one bot inside of it.
//!
//! # Safety
//!
//! There is no unsafe code in this module
//!
//!


use crate::inner_prelude::*;


///The geometric functions that the user must provide.
pub trait Knearest{
    type T:HasAabb<Num=Self::N>;
    type N:NumTrait;

    ///The type of number of minimize based off on.
    ///For example it can be distance or distance squared.
    type D:Ord+Copy+core::fmt::Debug;


    ///User defined expensive distance function. Here the user can return fine-grained distance
    ///of the shape contained in T instead of its bounding box.
    fn twod_check(&mut self, point:Vec2<Self::N>,bot:&Self::T)->Self::D;
    

    ///Return the distance between two objects
    //TODO only use this api?
    fn oned_check(&mut self,val1:Self::N,val2:Self::N)->Self::D;

    ///Create a range about the point n. This is used to
    ///limit the number of bots in a node that need to be checked.
    fn create_range(&mut self,point:Self::N,dis:Self::D)->[Self::N;2];
}



fn new_smallvec<T>(a:T)->SmallVec<[T;2]>{
    let mut b=SmallVec::new();
    b.push(a);
    b
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


/// Returned by k_nearest
pub struct Unit<'a,T:'a,D>{ //:Ord+Copy
    pub bots:SmallVec<[&'a T;2]>,
    pub mag:D
}
/// Returned by k_nearest_mut
pub struct UnitMut<'a,T:'a,D>{
    pub bots:SmallVec<[&'a mut T;2]>,
    pub mag:D
}
macro_rules! unit_create{
    ($a:expr,$b:expr)=>{{
        Unit{bots:$a,mag:$b}
    }}
}

macro_rules! unit_mut_create{
    ($a:expr,$b:expr)=>{{
        UnitMut{bots:$a,mag:$b}
    }}
}

macro_rules! knearest_recc{
    ($iterator:ty,$ptr:ty,$ref:ty,$get_iter:ident,$nonleaf:ident,$ref_lifetime:ty,$unit:ty,$unit_create:ident)=>{
        
        struct ClosestCand<'a,T:HasAabb+'a,D:Ord+Copy>{
            a:Vec<$unit>,
            num:usize
        }
        impl<'a,T:HasAabb+'a,D:Ord+Copy> ClosestCand<'a,T,D>{

            //First is the closest
            fn into_sorted(self)->Vec<$unit>{
                self.a
            }
            fn new(num:usize)->ClosestCand<'a,T,D>{
                let a=Vec::with_capacity(num);
                ClosestCand{a,num}
            }

            fn consider(&mut self,a:($ref_lifetime,D))->bool{
                //let a=(a.0 as $ptr,a.1);
                let curr_bot=a.0;
                let curr_dis=a.1;

                if self.a.len()<self.num{
                    let arr=&mut self.a;
                    
                    for i in 0..arr.len(){
                        if curr_dis<arr[i].mag{
                            let unit=$unit_create!(new_smallvec(curr_bot),curr_dis);// Unit{bots:new_smallvec(curr_bot),dis:curr_dis};
                            arr.insert(i,unit);
                            return true;
                        }else if curr_dis==arr[i].mag{
                            arr[i].bots.push(curr_bot);
                            return true;
                        }
                    }

                    //only way we get here is if the above didnt return.
                    let unit=$unit_create!(new_smallvec(curr_bot),curr_dis);//Unit{bots:new_smallvec(curr_bot),dis:curr_dis};
                    arr.push(unit);
                    
                }else{
                    let arr=&mut self.a;
                    for i in 0..arr.len(){
                        if curr_dis<arr[i].mag{
                            let v=arr.pop().unwrap();
                            let unit=$unit_create!(new_smallvec(curr_bot),curr_dis);//Unit{bots:new_smallvec(curr_bot),dis:curr_dis};
                            arr.insert(i,unit);
                        

                            let max=arr.iter().map(|a|a.mag).max().unwrap();
                            assert!(max<v.mag);
                            return true;
                        }else if curr_dis==arr[i].mag{
                            arr[i].bots.push(curr_bot);
                            return true;
                        }
                    }
                }
                return false;
            }

            fn full_and_max_distance(&self)->Option<D>{
                use is_sorted::IsSorted;
                assert!(IsSorted::is_sorted(&mut self.a.iter().map(|a|a.mag)));
                match self.a.get(self.num-1){
                    Some(x)=>
                    {
                        Some(x.mag)
                    },
                    None=>{
                        None
                    }
                }
            }
        }
        
        fn traverse_other<K:Knearest>(res:&ClosestCand<K::T,K::D>,k:&mut K,pp:K::N,div:K::N)->bool{
            match res.full_and_max_distance(){
                Some(max)=>{
                    k.oned_check(pp,div)<max
                },
                None=>{
                    true
                }
            }
        }

        fn recc<'a,
            N:NumTrait+'a,
            T:HasAabb<Num=N>+'a,
            A: AxisTrait,
            K:Knearest<T=T,N=N>,
            >(axis:A,stuff:LevelIter<$iterator>,knear:&mut K,point:Vec2<K::N>,res:&mut ClosestCand<'a,K::T,K::D>){

            //let pp=*axgeom::AxisWrapRef(&point).get(axis);
            //let ppother=*axgeom::AxisWrapRef(&point).get(axis.next());
            let pp=*point.get_axis(axis);
            let ppother=*point.get_axis(axis.next());

            let ((_depth,nn),rest)=stuff.next();

            match rest{
                Some([left,right])=>{
                    let div=match nn.div{
                        Some(b)=>b,
                        None=>return
                    };

                    let (first,second)=match pp.cmp(div){
                        core::cmp::Ordering::Less=>{
                            (left,right)
                        },
                        core::cmp::Ordering::Greater=>{
                            (right,left)
                        },
                        core::cmp::Ordering::Equal=>{
                            //This case it doesnt really matter whether we traverse left or right first.
                            (left,right)
                        }
                    };

                    recc(axis.next(),first,knear,point,res);

                    if traverse_other(res,knear,pp,*div){
                        recc(axis.next(),second,knear,point,res);
                    }
                    //Check again incase the other recursion took care of everything
                    //We are hoping that it is more likely that the closest points are found
                    //in decendant nodes instead of ancestor nodes.
                    //if traverse_other(res,knear,pp,div){
                    for bot in $get_iter!(nn.bots){
                        match res.full_and_max_distance(){
                            Some(dis)=>{
                                
                                //Used for both x and y.
                                //Think of this is a bounding box around the point that grows
                                let [leftr,rightr]=knear.create_range(ppother,dis);

                                let [leftbot,rightbot]={
                                    [bot.get().get_range(axis.next()).left,bot.get().get_range(axis.next()).right]
                                };
                                
                                if leftbot>rightr{
                                    //All the bots after this will also be too far away.
                                    //because the bots are sorted in ascending order.
                                    break;
                                }else if rightbot>=leftr{
                                    let dis_sqr=knear.twod_check(point,bot);
                                    res.consider((bot,dis_sqr));
                                
                                }
                            },
                            None=>{
                                let dis_sqr=knear.twod_check(point,bot);
                                res.consider((bot,dis_sqr));
                            
                            }
                        }                           
                    }
                },
                None=>{
                    
                    for bot in $get_iter!(nn.bots){
                        match res.full_and_max_distance(){
                            Some(dis)=>{
                                
                                //TODO use leftr??
                                let [_leftr,rightr]=knear.create_range(ppother,dis);

                                let [leftbot,_rightbot]={
                                    [bot.get().get_range(axis.next()).left,bot.get().get_range(axis.next()).right]
                                };
                                
                                if leftbot>rightr{
                                    //All the bots after this will also be too far away.
                                    //because the bots are sorted in ascending order.
                                    break;
                                }else{
                                    let dis_sqr=knear.twod_check(point,bot);
                                    res.consider((bot,dis_sqr));
                                } 
                            },
                            None=>{
                                let dis_sqr=knear.twod_check(point,bot);
                                res.consider((bot,dis_sqr));
                            
                            }
                        }                          
                    }
                }
            }
        }
    }
}


///The dinotree's NumTrait does not inherit any kind of arithmetic traits.
///This showcases that the tree construction and pair finding collision algorithms
///do not involves any arithmetic. 
///However, when finding the nearest neighbor, we need to do some calculations to
///compute distance between points. So instead of giving the NumTrait arithmetic and thus
///add uneeded bounds for general use of this tree, the user must provide functions for arithmetic
///specifically for this function.
///The user can also specify what the minimum distance function is minizing based off of. For example
///minimizing based off the square distance will give you the same answer as minimizing based off 
///of the distant. 
///The callback function will be called on the closest object, then the second closest, and so on up 
///until k.
///User can also this way choose whether to use manhatan distance or not.

///Its important to distinguish the fact that there is no danger of any of the references returned being the same.
///The closest is guarenteed to be distinct from the second closest. That is not to say they they don't overlap in 2d space.




pub use self::con::naive;
pub use self::con::k_nearest;
mod con{
    use super::*;
    pub fn k_nearest<'b,
        //T:HasAabb,
        //A:AxisTrait+'b,
        V:DinoTreeRefTrait,
        K:Knearest<T=V::Item,N=V::Num>,
        >(tree:&'b V,point:Vec2<V::Num>,num:usize,mut knear: K)->NearestResult<'b,V::Item,K::D>{
        let axis=tree.axis();
        let dt = tree.vistr().with_depth(Depth(0));

        let mut c=ClosestCand::new(num);


        recc(axis,dt,&mut knear,point,&mut c);
     
        NearestResult{inner:c.into_sorted().into_iter()}
    }

    knearest_recc!(Vistr<'a,K::T>,*const T,&T,get_range_iter,NonLeafDyn,&'a T,Unit<'a,T,D>,unit_create);

    pub fn naive<'b,K:Knearest>(bots:impl Iterator<Item=&'b K::T>,point:Vec2<K::N>,num:usize,mut k:K)->Vec<Unit<'b,K::T,K::D>>{
        
        let mut closest=ClosestCand::new(num);

        for b in bots{
            let d=k.twod_check(point,b);

            if let Some(dis)=closest.full_and_max_distance(){    
                if d>dis{
                    continue;
                }
            }

            closest.consider((b,d));
        }

        closest.into_sorted()
    }

}

pub use self::mutable::naive_mut;
pub use self::mutable::k_nearest_mut;
mod mutable{
    use super::*;
    pub fn naive_mut<'b,K:Knearest>(bots:impl Iterator<Item=&'b mut K::T>,point:Vec2<K::N>,num:usize,mut k:K)->Vec<UnitMut<'b,K::T,K::D>>{
        
        let mut closest=ClosestCand::new(num);

        for b in bots{
            let d=k.twod_check(point,b);

            if let Some(dis)= closest.full_and_max_distance(){
                if d>dis{
                    continue;
                }
            }

            closest.consider((b,d));
        }

        closest.into_sorted()

    }


    knearest_recc!(VistrMut<'a,K::T>,*mut T,&mut T,get_mut_range_iter,NonLeafDynMut,&'a mut T,UnitMut<'a,T,D>,unit_mut_create);

    pub fn k_nearest_mut<'b,
        V:DinoTreeRefMutTrait,
        K:Knearest<N=V::Num,T=V::Item>,
        >(tree:&'b mut V,point:Vec2<V::Num>,num:usize,mut knear: K)->NearestResultMut<'b,V::Item,K::D>{ //Vec<UnitMut<'b,T,K::D>>
        let axis=tree.axis();
        let dt = tree.vistr_mut().with_depth(Depth(0));

        let mut c=ClosestCand::new(num);

        recc(axis,dt,&mut knear,point,&mut c);
     
        NearestResultMut{inner:c.into_sorted().into_iter()}
    }

}



///Returns the closest to the furthest unit found.
pub struct NearestResult<'a,T,D>{
    inner:alloc::vec::IntoIter<Unit<'a,T,D>>
}
impl<'a,T,D> Iterator for NearestResult<'a,T,D>{
    type Item=Unit<'a,T,D>;
    fn next(&mut self)->Option<Self::Item>{
        self.inner.next()
    }
    fn size_hint(&self)->(usize,Option<usize>){
        self.inner.size_hint()
    }
}

impl<'a,T,D> core::iter::FusedIterator for NearestResult<'a,T,D>{}
impl<'a,T,D> core::iter::ExactSizeIterator for NearestResult<'a,T,D>{}
//unsafe impl<'a,T,D> std::iter::TrustedLen for NearestResult<'a,T,D>{}


///Returns the closest to the furthest unit found.
pub struct NearestResultMut<'a,T,D>{
    inner:alloc::vec::IntoIter<UnitMut<'a,T,D>>
}
impl<'a,T,D> Iterator for NearestResultMut<'a,T,D>{
    type Item=UnitMut<'a,T,D>;
    fn next(&mut self)->Option<Self::Item>{
        self.inner.next()
    }

    fn size_hint(&self)->(usize,Option<usize>){
        self.inner.size_hint()
    }
}
impl<'a,T,D> core::iter::FusedIterator for NearestResultMut<'a,T,D>{}
impl<'a,T,D> core::iter::ExactSizeIterator for NearestResultMut<'a,T,D>{}
//unsafe impl<'a,T,D> std::iter::TrustedLen for NearestResultMut<'a,T,D>{}
