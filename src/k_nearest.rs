use inner_prelude::*;
use super::*;


//TODO use the property that the trees are sorted somehow.


use self::cand::ClosestCand;
mod cand{
    use super::*;

    pub struct ClosestCand<T:SweepTrait,D:Ord+Copy>{
        a:SmallVec<[(*mut T,D);32]>,
        num:usize
    }
    impl<T:SweepTrait,D:Ord+Copy> ClosestCand<T,D>{

        //First is the closest
        pub fn into_sorted(self)->SmallVec<[(*mut T,D);32]>{
            self.a
        }
        pub fn new(num:usize)->ClosestCand<T,D>{
            let a=SmallVec::with_capacity(num);
            ClosestCand{a,num}
        }

        pub fn consider(&mut self,a:(&mut T,D))->bool{
            let a=(a.0 as *mut T,a.1);

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
                        arr.pop();
                        arr.insert(i,a);
                        return true;
                    }
                }
            }
            return false;

        }
        pub fn full_and_max_distance(&self)->Option<D>{
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
//(x*x)  (y*y)
//


pub trait Knearest{
    type T:SweepTrait<Num=Self::N>;
    type N:NumTrait;
    type D:Ord+Copy+std::fmt::Debug;
    fn twod_check(&mut self, [Self::N;2],&AABBox<Self::N>)->Self::D;
    fn oned_check(&mut self,Self::N,Self::N)->Self::D;

    //create a range around n.
    fn create_range(&mut self,Self::N,Self::D)->[Self::N;2];
    //fn handle(&mut self,ColSingle<'b,Self::T>,Self::D);
}


pub fn k_nearest<'b,
    A:AxisTrait,
    K:Knearest,
    >(tree:&'b mut DynTree<A,(),K::T>,point:[K::N;2],num:usize,mut knear: K,mut func:impl FnMut(ColSingle<'b,K::T>,K::D))
        where K::N:'b{
    let axis=A::new();
    let dt = tree.get_iter_mut().with_depth(Depth(0));

    let mut c=ClosestCand::new(num);
    recc(axis,dt,&mut knear,point,&mut c);
 
    for i in c.into_sorted(){
        let j:&mut K::T=unsafe{&mut *i.0};

        let j=j.get_mut();
        //let j=unsafe{&mut *i.0}.get_mut();
        //let j:(&AABBox<<K::T as SweepTrait>::Num>,&mut <K::T as SweepTrait>::Inner)=unsafe{&mut *i.0}.get_mut();
        //let j:(&AABBox<<K::T as SweepTrait>::Num>,&'b mut <K::T as SweepTrait>::Inner)=unsafe{std::mem::transmute(j)};
        func(ColSingle{inner:j.1,rect:j.0},i.1);
        //knear.handle(ColSingle{inner:j.1,rect:j.0},i.1);
    }


}
fn recc<
    A: AxisTrait,
    K:Knearest,
    >(axis:A,stuff:LevelIter<NdIterMut<(),K::T>>,knear:&mut K,point:[K::N;2],res:&mut ClosestCand<K::T,K::D>){

    let ((depth,nn),rest)=stuff.next();

    //known at compile time.
    let pp=if axis.is_xaxis(){
        point[0]
    }else{
        point[1]
    };

    let ppother=if axis.is_xaxis(){
        point[1]
    }else{
        point[0]
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

            recc(axis.next(), first,knear,point,res);
           
            if traverse_other(res,knear,pp,div){
                recc(axis.next(),other,knear,point,res);
            }

            //Check again incase the other recursion took care of everything
            //We are hoping that it is more likely that the closest points are found
            //in decendant nodes instead of ancestor nodes.
            //if traverse_other(res,knear,pp,div){
            {  
                let mut bb=nn.range.iter_mut().peekable();
                
                {//Skip over all the bots that dont arnt inside the range.
                    match res.full_and_max_distance(){
                        Some(dis)=>{    
                            let [leftr,rightr]=knear.create_range(ppother,dis);
                            /*
                            if depth.0==0{
                                println!("leftr,right,dis={:?}",(leftr,rightr,dis));
                            }
                            */
                            //println!("left,rightr={:?}",(leftr,rightr));
                            
                            loop{
                                let skip={
                                    let bot=match bb.peek(){
                                        Some(bot)=>{bot},
                                        None=>{break}
                                    };

                                    let [leftbot,rightbot]={
                                        [(bot.get().0).0.get_range2::<A::Next>().left(),(bot.get().0).0.get_range2::<A::Next>().right()]
                                    };

                                    if rightbot>=leftr{
                                        break;
                                    }
                                };

                                bb.next();
                            }
                        },
                        None=>{

                        }
                    }
                }
                
                /*
                for bot in bb{
                    let dis_sqr=knear.twod_check(point,bot.get().0);
                    res.consider((bot,dis_sqr));
                }
                */
                
                
                {
                    for bot in bb{
                        match res.full_and_max_distance(){
                            Some(dis)=>{

                                let [leftr,rightr]=knear.create_range(ppother,dis);

                                let [leftbot,rightbot]={
                                    [(bot.get().0).0.get_range2::<A::Next>().left(),(bot.get().0).0.get_range2::<A::Next>().right()]
                                };
                                
                                if leftbot>rightr{
                                    //All the bots after this will also be too far away.
                                    //because the bots are sorted in ascending order.
                                    break;
                                }else{
                                    let dis_sqr=knear.twod_check(point,bot.get().0);
                                    res.consider((bot,dis_sqr));
                                }
                            },
                            None=>{
                                let dis_sqr=knear.twod_check(point,bot.get().0);
                                res.consider((bot,dis_sqr));
                            
                            }
                        }                          
                    }
                }
                
                
                
            
            }

        }
        _ => {
            //If we are a child, just handle everything.
            for i in nn.range.iter_mut(){            
                let dis_sqr=knear.twod_check(point,i.get().0);
                res.consider((i,dis_sqr));
            }        
        }
    }
}
