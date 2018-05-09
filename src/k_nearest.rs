use inner_prelude::*;
use super::*;

/*
//test
mod stuff{
    use super::*;


    pub trait NodeTrait{
        type N:NumTrait;
        type T;
        fn get_div(&self)->Option<Self::N>;
        fn get_cont(&self)->Option<Range<Self::N>>;
        fn for_every<F:FnMut(Self::T)>(&mut self,func:F);
    }

    pub trait Callback{
        type T;
        type N:NumTrait;
        fn get(&self,a:Self::T)->&Rect<Self::N>;
        fn callback(&mut self,a:Self::T,a:Self::N);
    }


    struct Blag2<'a,T:SweepTrait+'a>(&'a mut NodeDyn<T>);

    impl<'a,T:SweepTrait+'a> NodeTrait for Blag2<'a,T>{
        type N=T::Num;
        type T=&'a mut T;
        fn get_div(&self)->Option<Self::N>{
            self.0.div
        }
        fn get_cont(&self)->Option<Range<Self::N>>{
            self.0.cont
        }
        fn for_every<F:FnMut(Self::T)>(&mut self,mut func:F){
            for bot in self.0.range.iter_mut(){
                func(bot);
            }
        }
    }

    struct Blag<'a,T:'a,F>((PhantomData<&'a T>,F));

    impl<'a,T:SweepTrait,F:FnMut(ColSingle<T>,T::Num)> Callback for Blag<'a,T,F>{
        type T=&'a mut T;
        type N=T::Num;
        fn get(&self,a:&'a mut T)->&Rect<T::Num>{
            &(a.get().0).0
        }
        fn callback(&mut self,a:&mut T,b:T::Num){
            let j=a.get_mut();
            let c=ColSingle{inner:j.1,rect:j.0};
            let dis=b;
            ((self.0).1)(c,dis)
        }
    }




    pub fn test<BB:NodeTrait,A:AxisTrait,C: CTreeIterator<Item = BB>,H:Callback<T=BB::T,N=BB::N>>(stuff:C,callback:H){
        let (mut nn,rest)=stuff.next();

        nn.for_every(|bot|{
            let r=callback.get(bot);
        });

    }
}*/


pub fn k_nearest<'b,
    A:AxisTrait,
    T:SweepTrait,
    F: FnMut(ColSingle<'b,T>,T::Num),
    MF:FnMut([T::Num;2],&AABBox<T::Num>)->T::Num,
    MF2:FnMut(T::Num,T::Num)->T::Num,
    >(tree:&'b mut DynTree<A,(),T>,point:[T::Num;2],num:usize,mut func:F,mut mf:MF,mut mf2:MF2){

    let dt = tree.get_iter_mut();

    let mut c=ClosestCand::new(num);
    recc(A::new(),dt,&mut mf,&mut mf2,point,&mut c);
 
    for i in c.into_sorted(){
        let j=unsafe{&mut *i.0}.get_mut();
        func(ColSingle{inner:j.1,rect:j.0},i.1);
    }


}

//TODO use the property that the trees are sorted somehow.


use self::cand::ClosestCand;
mod cand{
    use super::*;

    pub struct ClosestCand<T:SweepTrait>{
        a:SmallVec<[(*mut T,T::Num);32]>,
        num:usize
    }
    impl<T:SweepTrait> ClosestCand<T>{

        //First is the closest
        pub fn into_sorted(self)->SmallVec<[(*mut T,T::Num);32]>{
            self.a
        }
        pub fn new(num:usize)->ClosestCand<T>{
            let a=SmallVec::with_capacity(num);
            ClosestCand{a,num}
        }

        pub fn consider(&mut self,a:(&mut T,T::Num)){
            let a=(a.0 as *mut T,a.1);

            if self.a.len()<self.num{
                

                let arr=&mut self.a;
                if arr.len()==0{
                    arr.push(a);
                }else{
                    let mut inserted=false;
                    for i in 0..arr.len(){
                        if a.1<arr[i].1{
                            arr.insert(i,a);
                            inserted=true;
                            break;
                        }
                    }
                    if !inserted{
                        arr.push(a);
                    }

                }

            }else{
                let arr=&mut self.a;
                for i in 0..arr.len(){
                    if a.1<arr[i].1{
                        arr.pop();
                        arr.insert(i,a);
                        break;
                    }
                }
                
            }
        }
        pub fn full_and_max_distance(&self)->Option<T::Num>{
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
}


fn traverse_other<T:SweepTrait,MF2:FnMut(T::Num,T::Num)->T::Num>(res:&ClosestCand<T>,mf2:&mut MF2,pp:T::Num,div:T::Num)->bool{
    match res.full_and_max_distance(){
        Some(max)=>{
            if mf2(pp,div)<max{
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


fn recc<'x,'a,
    A: AxisTrait,
    T: SweepTrait + 'x,
    C: CTreeIterator<Item = &'x mut NodeDyn<(),T>>,
    MF:FnMut([T::Num;2],&AABBox<T::Num>)->T::Num,
    MF2:FnMut(T::Num,T::Num)->T::Num,
    >(axis:A,stuff:C,mf:&mut MF,mf2:&mut MF2,point:[T::Num;2],res:&mut ClosestCand<T>){

    let (nn,rest)=stuff.next();

    //known at compile time.
    let pp=if axis.is_xaxis(){
        point[0]
    }else{
        point[1]
    };

    match rest {
        Some((left, right)) => {
            let div=match nn.div{
                Some(div)=>{div},
                None=>{return;}
            };
    

            let (first,other)=if pp<div {
                (left,right)
            }else{
                (right,left)
            };

            recc(axis.next(), first,mf,mf2,point,res);
           
            if traverse_other(res,mf2,pp,div){
                recc(axis.next(),other,mf,mf2,point,res);
            }

            //Check again incase the other recursion took care of everything
            //We are hoping that it is more likely that the closest points are found
            //in decendant nodes instead of ancestor nodes.
            if traverse_other(res,mf2,pp,div){
                for i in nn.range.iter_mut(){            
                    let dis_sqr=mf(point,i.get().0);
                    res.consider((i,dis_sqr));
                }
            }

        }
        _ => {
            for i in nn.range.iter_mut(){            
                let dis_sqr=mf(point,i.get().0);
                res.consider((i,dis_sqr));
            }        
        }
    }
}
