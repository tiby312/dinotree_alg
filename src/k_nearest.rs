use inner_prelude::*;
use super::*;


pub fn k_nearest<
    A:AxisTrait,
    T:SweepTrait,
    F: FnMut(ColSingle<T>),
    MF:Fn((T::Num,T::Num),&AABBox<T::Num>)->T::Num,
    MF2:Fn(T::Num,T::Num)->T::Num,
    >(tree:&mut DynTree<A,T>,point:(T::Num,T::Num),num:usize,mut func:F,mf:MF,mf2:MF2){

    let dt = tree.get_iter_mut();

    let mut c=ClosestCand::new(num);
    recc(A::new(),dt,&mf,&mf2,point,&mut c);
 
    for i in c.into_sorted(){
        let j=unsafe{&mut *i.0}.get_mut();
        func(ColSingle{inner:j.1,rect:j.0});
    }


}


use self::cand::ClosestCand;
mod cand{
    use super::*;

    pub struct ClosestCand<T:SweepTrait>{
        a:Vec<(*mut T,T::Num)>,
        num:usize
    }
    impl<T:SweepTrait> ClosestCand<T>{

        //First is the closest
        pub fn into_sorted(self)->Vec<(*mut T,T::Num)>{
            self.a
        }
        pub fn new(num:usize)->ClosestCand<T>{
            let a=Vec::with_capacity(num);
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
            /* check sorted
            {
                let mut c=arr.first().unwrap();
                for i in arr
                [1..].iter(){
                    assert!(i.1>=c.1,"{:?}",(i.1,c.1));
                    c=i;
                }
            }
            */
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

fn recc<'x,'a,
    A: AxisTrait,
    T: SweepTrait + 'x,
    C: CTreeIterator<Item = &'x mut NodeDyn<T>>,
    MF:Fn((T::Num,T::Num),&AABBox<T::Num>)->T::Num,
    MF2:Fn(T::Num,T::Num)->T::Num,
    >(axis:A,stuff:C,mf:&MF,mf2:&MF2,point:(T::Num,T::Num),res:&mut ClosestCand<T>){

    let (nn,rest)=stuff.next();

    //known at compile time.
    let pp=if axis.is_xaxis(){
        point.0
    }else{
        point.1
    };

    
    match rest {
        Some((left, right)) => {
            let div = nn.div.unwrap();
    

            let (first,other)=if pp<div {
                (left,right)
            }else{
                (right,left)
            };

            recc(axis.next(), first,mf,mf2,point,res);
           
            let traverse_other=match res.full_and_max_distance(){
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
            };

            if traverse_other{
                recc(axis.next(),other,mf,mf2,point,res);
            }
        }
        _ => {
            
        }
    }

    let traverse_other=match res.full_and_max_distance(){
        Some(max)=>{
            match nn.div{
                Some(div)=>{
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
        },
        None=>{
            true
        }
    };

    if traverse_other{
        for i in nn.range.iter_mut(){            
            let dis_sqr=mf(point,i.get().0);
            res.consider((i,dis_sqr));
        }
    }

}