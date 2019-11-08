//!
//! # User Guide
//!
//! There are four flavors of the same fundamental knearest api provided in this module.
//! There is a naive version, and there is a version that uses the tree, and there are mutable versions of those 
//! that return mutable references.
//!
//! Along with a reference to the tree, the user provides the needed geometric functions by passing an implementation of Knearest.
//! The user provides a point, and the number of nearest objects to return.
//! Then the equivalent to a Vec<(&mut T,N)> is returned where T is the element, and N is its distance.
//! Even if you are only looking for one closest element, becaise of ties, it is possible for many many bots to be returned.
//! If we only returned an arbitrary one, then that would make verification against the naive algorithm harder.
//! It is possible for the Vec returned to be empty if the tree does not contain any bots.
//! While ties are possible, the ordering in which the ties are returned is arbitrary and has no meaning.
//!
//! If the user looks for multiple nearest, then the Vec will return first, all the 1st closest ties,
//! then all the 2nd closest ties, etc.
//! 
//! Slice splitting functions are provided that will split up the Vec into slices over just the ties. 
//!


use crate::inner_prelude::*;
use core::cmp::Ordering;

///The geometric functions that the user must provide.
pub trait Knearest{
    type T:HasAabb<Num=Self::N>;
    type N:NumTrait;
    
    ///User defined expensive distance function. Here the user can return fine-grained distance
    ///of the shape contained in T instead of its bounding box.
    fn distance_to_bot(&self, point:Vec2<Self::N>,bot:&Self::T)->Self::N{
        self.distance_to_rect(point,bot.get())
    }

    fn distance_to_rect(&self,point:Vec2<Self::N>,rect:&Rect<Self::N>)->Self::N;

}




///Splits a mutable slice into multiple slices
///The splits occur where the predicate returns false.
pub struct SliceSplitMut<'a,T,F>{
    arr:Option<&'a mut [T]>,
    func:F
}

impl<'a,T,F:FnMut(&T,&T)->bool> SliceSplitMut<'a,T,F>
{
    pub fn new(arr:&'a mut [T],func:F)->SliceSplitMut<'a,T,F>
    {
        SliceSplitMut{arr:Some(arr),func}
    }
}

impl<'a,T,F:FnMut(&T,&T)->bool> Iterator for SliceSplitMut<'a,T,F>{
    type Item=&'a mut [T];
    fn next(&mut self)->Option<Self::Item>{
        let (last,arr)={
            let arr=self.arr.take()?;
            let i=arr.get(0)?;        
            let count=arr.iter().peeking_take_while(|a|(self.func)(a,i)).count();
            (count,arr)
        };
        let (first,rest)=arr.split_at_mut(last);
        self.arr=Some(rest);
        Some(first)
    }
}

///Splits a mutable slice into multiple slices
///The splits occur where the predicate returns false.
pub struct SliceSplit<'a,T,F>{
    arr:Option<&'a [T]>,
    func:F
}
impl<'a,T,F:FnMut(&T,&T)->bool> SliceSplit<'a,T,F>{
    pub fn new(arr:&'a [T],func:F)->SliceSplit<'a,T,F>{
        SliceSplit{arr:Some(arr),func}
    }
}
impl<'a,T,F:FnMut(&T,&T)->bool> Iterator for SliceSplit<'a,T,F>{
    type Item=&'a [T];
    fn next(&mut self)->Option<Self::Item>{
        let (last,arr)={
            let arr=self.arr.take()?;
            let i=arr.get(0)?;        
            let count=arr.iter().peeking_take_while(|a|(self.func)(a,i)).count();
            (count,arr)
        };
        let (first,rest)=arr.split_at(last);
        self.arr=Some(rest);
        Some(first)
    }
}





fn make_rect_from_range<A:AxisTrait,N:NumTrait>(axis:A,range:&Range<N>,rect:&Rect<N>)->Rect<N>{
    if axis.is_xaxis(){
        Rect{x:*range,y:rect.y}
    }else{
        Rect{x:rect.x,y:*range}
    }
}


fn range_side<N:NumTrait>(point:Vec2<N>,axis:impl axgeom::AxisTrait,range:&Range<N>)->Ordering{
    if axis.is_xaxis(){
        range.left_or_right_or_contain(&point.x)
    }else{
        range.left_or_right_or_contain(&point.y)
    }
}

/// Returned by k_nearest_mut
pub struct UnitMut<'a,T:HasAabb>{
    pub bot:ProtectedBBox<'a,T>,
    pub mag:T::Num
}


struct ClosestCand<'a,T:HasAabb>{
    //Can have multiple bots with the same mag. So the length could be bigger than num.
    bots:Vec<UnitMut<'a,T>>,
    //The current number of different distances in the vec
    curr_num:usize,
    //The max number of different distances.
    num:usize
}
impl<'a,T:HasAabb> ClosestCand<'a,T>{

    //First is the closest
    fn into_sorted(self)->Vec<UnitMut<'a,T>>{
        self.bots
    }
    fn new(num:usize)->ClosestCand<'a,T>{
        let bots=Vec::with_capacity(num);
        ClosestCand{bots,num,curr_num:0}
    }

    fn consider(&mut self,a:(ProtectedBBox<'a,T>,T::Num))->bool{
        //let a=(a.0 as $ptr,a.1);
        let curr_bot=a.0;
        let curr_dis=a.1;

        if self.curr_num<self.num{
            let arr=&mut self.bots;
            
            for i in 0..arr.len(){
                if curr_dis<arr[i].mag{
                    let unit= UnitMut{bot:curr_bot,mag:curr_dis};//$unit_create!(curr_bot,curr_dis);
                    arr.insert(i,unit);
                    self.curr_num+=1;
                    return true;
                }
            }
            //only way we get here is if the above didnt return.
            let unit=UnitMut{bot:curr_bot,mag:curr_dis};
            self.curr_num+=1;
            arr.push(unit);
        }else{
            let arr=&mut self.bots;
            for i in 0..arr.len(){
                if curr_dis<arr[i].mag{
                    let v=arr.pop().unwrap();
                    loop{
                        
                        if arr[arr.len()-1].mag==v.mag{
                            arr.pop().unwrap();
                        }else{
                            break;
                        }
                    }
                    let unit=UnitMut{bot:curr_bot,mag:curr_dis};//$unit_create!(curr_bot,curr_dis);
                    arr.insert(i,unit);
                
                    let max=arr.iter().map(|a|a.mag).max().unwrap();
                    assert!(max<v.mag);
                    return true;
                }else if curr_dis==arr[i].mag{
                    let unit=UnitMut{bot:curr_bot,mag:curr_dis};//$unit_create!(curr_bot,curr_dis);
                    arr.insert(i,unit);
                    return true;
                }
            }
        }

        false
    }

    fn full_and_max_distance(&self)->Option<T::Num>{
        use is_sorted::IsSorted;
        assert!(IsSorted::is_sorted(&mut self.bots.iter().map(|a|a.mag)));
        if self.curr_num==self.num{
            self.bots.last().map(|a|a.mag)
        }else{
            None
        }
    }
}

struct Blap<'a:'b,'b,K:Knearest>{
    knear:&'b mut K,
    point:Vec2<K::N>,
    closest:ClosestCand<'a,K::T>
}

impl<'a:'b,'b,K:Knearest> Blap<'a,'b,K>{
    fn should_traverse_rect(&self,rect:&Rect<K::N>)->bool{
        if let Some(dis) = self.closest.full_and_max_distance(){
            self.knear.distance_to_rect(self.point,rect)<dis
        }else{
            true
        }
    }
}

fn recc<'a:'b,'b,
    N:NodeTrait,
    A: AxisTrait,
    K:Knearest<N=N::Num,T=N::T>,
    >(axis:A,stuff:LevelIter<VistrMut<'a,N>>,rect:Rect<K::N>,blap:&mut Blap<'a,'b,K>){

    let ((_depth, nn),rest)=stuff.next();
    let nn=nn.get_mut();
    match rest{
        Some([left,right])=>{
            let div=match nn.div{
                Some(b)=>b,
                None=>return
            };

            let (rleft,rright) = rect.subdivide(axis,*div);


            let range=&match nn.cont{
                Some(cont)=>{
                    *cont
                },
                None=>{
                    Range{left:*div,right:*div}
                }
            };

            let rmiddle=make_rect_from_range(axis,range,&rect);


            match range_side(blap.point,axis,range){
                Ordering::Less=>{
                    if blap.should_traverse_rect(&rleft){
                        recc(axis.next(),left,rleft,blap);
                    }

                    if blap.should_traverse_rect(&rmiddle){
                        for bot in nn.bots.iter_mut()
                        {
                            let dis_sqr=blap.knear.distance_to_bot(blap.point,bot.as_ref());
                            blap.closest.consider((bot,dis_sqr));
                        }
                    }

                    if blap.should_traverse_rect(&rright){
                        recc(axis.next(),right,rright,blap);
                    }
                },
                Ordering::Greater=>{

                    if blap.should_traverse_rect(&rright){
                        recc(axis.next(),right,rright,blap);
                    }

                    if blap.should_traverse_rect(&rmiddle){
                        for bot in nn.bots.iter_mut()
                        {
                            let dis_sqr=blap.knear.distance_to_bot(blap.point,bot.as_ref());
                            blap.closest.consider((bot,dis_sqr));
                        }
                    }
                    if blap.should_traverse_rect(&rleft){
                        recc(axis.next(),left,rleft,blap);
                    }

                },
                Ordering::Equal=>{
                    if blap.should_traverse_rect(&rmiddle){
                        for bot in nn.bots.iter_mut(){
                            let dis_sqr=blap.knear.distance_to_bot(blap.point,bot.as_ref());
                            blap.closest.consider((bot,dis_sqr));
                        }
                    }
                    if blap.should_traverse_rect(&rright){
                        recc(axis.next(),right,rright,blap);
                    }
                    if blap.should_traverse_rect(&rleft){
                        recc(axis.next(),left,rleft,blap);
                    }
                }
            }
        },
        None=>{
            for bot in nn.bots.iter_mut(){
                let dis_sqr=blap.knear.distance_to_bot(blap.point,bot.as_ref());
                blap.closest.consider((bot,dis_sqr));
            }
        }
    }
}
//    }
//}


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



/*
pub use self::con::naive;
pub use self::con::k_nearest;
mod con{
    use super::*;
    pub fn k_nearest<'b,
        V:DinoTreeRefTrait,
        >(tree:&'b V,point:Vec2<V::Num>,num:usize,knear: impl Knearest<T=V::Item,N=V::Num>,rect:Rect<V::Num>)->Vec<Unit<'b,V::Item,V::Num>>{
        let axis=tree.axis();
        let dt = tree.vistr().with_depth(Depth(0));

        let closest=ClosestCand::new(num);
        let mut blap=Blap{knear,point,closest};
        recc(axis,dt,rect,&mut blap);
    
        blap.closest.into_sorted()
    }

    knearest_recc!(Vistr<'a,K::T>,*const T,&T,get_range_iter,NonLeafDyn,&'a T,Unit<'a,T,D>,unit_create);

    pub fn naive<'b,K:Knearest>(bots:impl Iterator<Item=&'b K::T>,point:Vec2<K::N>,num:usize,k:K)->Vec<Unit<'b,K::T,K::N>>{
        
        let mut closest=ClosestCand::new(num);

        for b in bots{
            //TODO check aabb first
            let d=k.distance_to_bot(point,b);

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
*/

pub use self::mutable::naive_mut;
pub use self::mutable::k_nearest_mut;
mod mutable{
    use super::*;
    
    pub fn naive_mut<'a,K:Knearest<T=T,N=T::Num>,T:HasAabb>(bots:ProtectedBBoxSlice<'a,T>,point:Vec2<K::N>,num:usize,k:&mut K)->Vec<UnitMut<'a,K::T>>{
        //let bots=ProtectedBBoxSlice::new(bots);

        let mut closest=ClosestCand::new(num);

        for b in bots.iter_mut(){

            let d=k.distance_to_bot(point,b.as_ref());

            if let Some(dis)= closest.full_and_max_distance(){
                if d>dis{
                    continue;
                }
            }

            closest.consider((b,d));
        }

        closest.into_sorted()
    }
    

    pub fn k_nearest_mut<'a,
        A:AxisTrait,
        N:NodeTrait,
        >(tree:&'a mut DinoTree<A,N>,point:Vec2<N::Num>,num:usize,knear: &mut impl Knearest<N=N::Num,T=N::T>,rect:Rect<N::Num>)->Vec<UnitMut<'a,N::T>>{
        
        let axis=tree.axis();
        
        let dt = tree.vistr_mut().with_depth(Depth(0));

        let closest=ClosestCand::new(num);
        
        let mut blap=Blap{knear,point,closest};
        
        recc(axis,dt,rect,&mut blap);
     
        blap.closest.into_sorted()
    }

}

