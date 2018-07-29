use inner_prelude::*;

pub trait Knearest{
    type T:HasAabb<Num=Self::N>;
    type N:NumTrait;
    type D:Ord+Copy+std::fmt::Debug;

    //Expensive check
    fn twod_check(&mut self, [Self::N;2],&Self::T)->Self::D;
    
    fn oned_check(&mut self,Self::N,Self::N)->Self::D;

    //create a range around n.
    fn create_range(&mut self,Self::N,Self::D)->[Self::N;2];

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

macro_rules! knearest_recc{
    ($iterator:ty,$ptr:ty,$ref:ty,$get_iter:ident,$nonleaf:ident)=>{
        
    
        struct ClosestCand<T:HasAabb,D:Ord+Copy>{
            a:SmallVec<[($ptr,D);32]>,
            num:usize
        }
        impl<T:HasAabb,D:Ord+Copy> ClosestCand<T,D>{

            //First is the closest
            fn into_sorted(self)->SmallVec<[($ptr,D);32]>{
                self.a
            }
            fn new(num:usize)->ClosestCand<T,D>{
                let a=SmallVec::with_capacity(num);
                ClosestCand{a,num}
            }

            fn consider(&mut self,a:($ref,D))->bool{
                let a=(a.0 as $ptr,a.1);

                if self.a.len()<self.num{
                    let arr=&mut self.a;
                    
                    for i in 0..arr.len(){
                        if a.1<arr[i].1{
                            arr.insert(i,a);
                            return true;
                        }
                    }

                    //only way we get here is if the above didnt return.
                    arr.push(a);
                    
                }else{
                    let arr=&mut self.a;
                    for i in 0..arr.len(){
                        if a.1<arr[i].1{
                            let v=arr.pop().unwrap();
                            arr.insert(i,a);

                            let max=arr.iter().max().unwrap();
                            assert!(max.1<v.1);
                            return true;
                        }
                    }
                }
                return false;

            }
            fn full_and_max_distance(&self)->Option<D>{
                use is_sorted::IsSorted;
                assert!(self.a.iter().map(|a|&a.1).is_sorted());
                match self.a.get(self.num-1){
                    Some(x)=>
                    {
                        Some(x.1)
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
                    if k.oned_check(pp,div)<max{
                        true
                    }else{
                        false
                    }
                },
                None=>{
                    true
                }
            }
        }
         
        fn recc<
            A: AxisTrait,
            K:Knearest,
            >(axis:A,stuff:LevelIter<$iterator>,knear:&mut K,point:[K::N;2],res:&mut ClosestCand<K::T,K::D>){

            let pp=*axgeom::AxisWrapRef(&point).get(axis);
            let ppother=*axgeom::AxisWrapRef(&point).get(axis.next());

            let ((_depth,nn),rest)=stuff.next();

            match rest{
                Some((extra,left,right))=>{
                    let &FullComp{div,cont}=match extra{
                        Some(b)=>b,
                        None=>return
                    };

                    let (first,second)=match pp.cmp(&div){
                        std::cmp::Ordering::Less=>{
                            (left,right)
                        },
                        std::cmp::Ordering::Greater=>{
                            (right,left)
                        },
                        std::cmp::Ordering::Equal=>{
                            //This case it doesnt really matter whether we traverse left or right first.
                            (left,right)
                        }
                    };

                    recc(axis.next(),first,knear,point,res);

                    if traverse_other(res,knear,pp,div){
                        recc(axis.next(),second,knear,point,res);
                    }
                    //Check again incase the other recursion took care of everything
                    //We are hoping that it is more likely that the closest points are found
                    //in decendant nodes instead of ancestor nodes.
                    //if traverse_other(res,knear,pp,div){
                    for bot in $get_iter!(nn.range){
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
                    for bot in $get_iter!(nn.range){
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
        A:AxisTrait,
        K:Knearest,
        >(tree:&'b DynTree<A,(),K::T>,point:[K::N;2],num:usize,mut knear: K)->KnearestResult<'b,K::T,K::D>{
        let axis=tree.get_axis();
        let dt = tree.get_iter().with_depth(Depth(0));

        let mut c=ClosestCand::new(num);


        recc(axis,dt,&mut knear,point,&mut c);
     

        let v:Vec<(&'b K::T,K::D)>=c.into_sorted().into_iter().map(|i|{
            let j:& K::T=unsafe{& *i.0};
            (j,i.1)
        }).collect();
        KnearestResult{it:v.into_iter()}
    }

    knearest_recc!(NdIter<(),K::T>,*const T,&T,get_range_iter,NonLeafDyn);

    pub fn naive<'b,K:Knearest>(bots:impl Iterator<Item=&'b K::T>,point:[K::N;2],num:usize,mut k:K)->KnearestResult<'b,K::T,K::D>{
        
        let mut closest=ClosestCand::new(num);

        for b in bots{
            let d=k.twod_check(point,b);

            match closest.full_and_max_distance(){
                Some(dis)=>{
                    if d>dis{
                        continue;
                    }
                },
                None=>{}
            }

            closest.consider((b,d));
        }


        let v:Vec<(&'b K::T,K::D)>=closest.into_sorted().into_iter().map(|i|{
            let j:& K::T=unsafe{&*i.0};
            (j,i.1)
        }).collect();
        KnearestResult{it:v.into_iter()}

    }

}

pub use self::mutable::naive_mut;
pub use self::mutable::k_nearest_mut;
mod mutable{
    use super::*;
    pub fn naive_mut<'b,K:Knearest>(bots:impl Iterator<Item=&'b mut K::T>,point:[K::N;2],num:usize,mut k:K)->KnearestResultMut<'b,K::T,K::D>{
        
        let mut closest=ClosestCand::new(num);

        for b in bots{
            let d=k.twod_check(point,b);

            match closest.full_and_max_distance(){
                Some(dis)=>{
                    if d>dis{
                        continue;
                    }
                },
                None=>{}
            }

            closest.consider((b,d));
        }


        let v:Vec<(&'b mut K::T,K::D)>=closest.into_sorted().into_iter().map(|i|{
            let j:&mut K::T=unsafe{&mut *i.0};
            (j,i.1)
        }).collect();
        KnearestResultMut{it:v.into_iter()}

    }

    knearest_recc!(NdIterMut<(),K::T>,*mut T,&mut T,get_mut_range_iter,NonLeafDynMut);

    pub fn k_nearest_mut<'b,
        A:AxisTrait,
        K:Knearest,
        >(tree:&'b mut DynTree<A,(),K::T>,point:[K::N;2],num:usize,mut knear: K)->KnearestResultMut<'b,K::T,K::D>{
        let axis=tree.get_axis();
        let dt = tree.get_iter_mut().with_depth(Depth(0));

        let mut c=ClosestCand::new(num);


        recc(axis,dt,&mut knear,point,&mut c);
     

        let v:Vec<(&'b mut K::T,K::D)>=c.into_sorted().into_iter().map(|i|{
            let j:&mut K::T=unsafe{&mut *i.0};
            (j,i.1)
        }).collect();
        KnearestResultMut{it:v.into_iter()}
    }
}


pub struct KnearestResult<'b,T:'b,F>{
    it:std::vec::IntoIter<(&'b T,F)>
}

impl<'b,T:'b,F> ExactSizeIterator for KnearestResult<'b,T,F>{}
unsafe impl<'b,T:'b,F> std::iter::TrustedLen for KnearestResult<'b,T,F>{}

impl<'b,T:'b,F> Iterator for KnearestResult<'b,T,F>{
    type Item=(&'b T,F);
    fn next(&mut self)->Option<Self::Item>{
        self.it.next()
    }
    fn size_hint(&self)->(usize,Option<usize>){
        self.it.size_hint()
    }
}

pub struct KnearestResultMut<'b,T:'b,F>{
    it:std::vec::IntoIter<(&'b mut T,F)>
}

impl<'b,T:'b,F> ExactSizeIterator for KnearestResultMut<'b,T,F>{}
unsafe impl<'b,T:'b,F> std::iter::TrustedLen for KnearestResultMut<'b,T,F>{}

impl<'b,T:'b,F> Iterator for KnearestResultMut<'b,T,F>{
    type Item=(&'b mut T,F);
    fn next(&mut self)->Option<Self::Item>{
        self.it.next()
    }
    fn size_hint(&self)->(usize,Option<usize>){
        self.it.size_hint()
    }
}

/*
pub fn k_nearest_mut<'b,
    A:AxisTrait,
    N:NumTrait,
    T,
    K:Knearest<N=N,T=BBox<N,T>>,
    >(tree:&'b mut DynTree<A,(),BBox<N,T>>,point:[K::N;2],num:usize,mut knear: K,mut func:impl FnMut(BBoxDet<'b,N,T>,K::D)){


    k_nearest_mut_unchecked(tree,point,num,knear,|a,b|{
        func(a.destruct(),b)
    });
}
*/
