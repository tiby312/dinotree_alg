use inner_prelude::*;
use super::*;


pub fn raycast<
    A:AxisTrait,
    T:SweepTrait,
    MF:Fn(ColSingle<T>)->Option<T::Num>, //called to test if this object touches the ray. if it does, return distance to start of ray
    MF2:FnMut(ColSingle<T>,T::Num),  //called for the first thing that touched the ray
    >(tree:&mut DynTree<A,T>,point:(T::Num,T::Num),dir:(T::Num,T::Num),mut func:MF,mut mf2:MF2){


    let dt = tree.get_iter_mut();

    let ray=Ray{point,dir};
    let mut closest=Closest{closest:None};
    recc(A::new(),dt,&func,&ray,&mut closest);

    match closest.closest{
        Some(x)=>{
            let bb=unsafe{&mut *x.0};
            let rr=bb.get_mut();
            let cc=ColSingle{inner:rr.1,rect:rr.0};
            mf2(cc,x.1);
        },
        None=>{

        }
    }
}



struct Closest<T:SweepTrait>{
    closest:Option<(*mut T,T::Num)>
}
impl<T:SweepTrait> Closest<T>{
    fn consider<MF:Fn(ColSingle<T>)->Option<T::Num>>(&mut self,bots:&mut [T],func:MF){

        for b in bots{
            let val={
                let (a,bb)=b.get_mut();
                let cc=ColSingle{inner:bb,rect:a};
                func(cc)
            };

            if let Some(x)=val{
                match self.closest{
                    Some(dis)=>{
                        if x<dis.1{
                            self.closest=Some((b,x));
                        }
                    },
                    None=>{
                        self.closest=Some((b,x));
                    }
                }
            }
        } 
    }

    fn is_empty(&self)->bool{
        self.closest.is_none()
    }
    fn get_dis(&self)->Option<T::Num>{
        match self.closest{
            Some(x)=>{
                Some(x.1)
            },
            None=>{
                None
            }
        }
    }
}


use self::ray::Ray;
mod ray{
    use super::*;
    pub struct Ray<N:NumTrait>{
        pub point:(N,N),
        pub dir:(N,N)
    }

    //TODO have a line struct.
    //When ray is split into two, return a ray and a line.

    pub enum Val<X>{
        BothTouch((X,X)),
        OneTouch(X),
        //NoTouch Impossibe????
    }

    impl<N:NumTrait> Ray<N>{

        pub fn intersects_divider<A:AxisTrait,X>(&self,axis:A,div:N,left:X,right:X)->Val<X>{
            //test code
            //Val::OneTouch(right)
            unimplemented!();
        }

        //Returns the closest possible distance between the ray, 
        //and the two parallel lines.
        pub fn closest_distance_to_cyclinder<A:AxisTrait>(&self,axis:A,cont:Range<N>)->N{
            unimplemented!();
        }
    }
}


//Returns the first object that touches the ray.
fn recc<'x,'a,
    A: AxisTrait,
    T: SweepTrait + 'x,
    C: CTreeIterator<Item = &'x mut NodeDyn<T>>,
    MF:Fn(ColSingle<T>)->Option<T::Num>, //User returns distance to ray origin if it collides with ray
    >(axis:A,stuff:C,func:&MF,ray:&Ray<T::Num>,closest:&mut Closest<T>){


    let (nn,rest)=stuff.next();


    match rest {
        Some((left, right)) => {
    
            let div=match nn.div{
                Some(div)=>div,
                None=>{
                    return  //There is nothing to consider in this node or any decendants.
                }
            };

            match ray.intersects_divider(axis.next(),div,left,right){
                ray::Val::BothTouch((first,second))=>{
                    //Its more likely that we'll find the closest bot in a children, than this node.
                    //This is because only the bots that intersect with this node are kept here.
                    //Many more bots exist in the lower you o in the tree.
                    //So because of this, lets recurse first, before we check this node.
                    recc(axis.next(),first,func,ray,closest);

                    //Only bother considering the bots in this node,
                    match closest.get_dis(){
                        Some(dis)=>{
                            match nn.cont{
                                Some(cont)=>{
                                    if ray.closest_distance_to_cyclinder(axis.next(),cont)<dis{
                                        closest.consider(&mut nn.range,func);
                                    }        
                                },
                                None=>{
                                    //this node does not have any nodes, so dont need to consider
                                    //anything in this node.
                                }
                            }
                        },
                        None=>{
                            closest.consider(&mut nn.range,func);  
                        }
                    }
                    

                    //So only in the case where we literally could not find a single bot 
                    //that intersected the ray, do we recurse the side of the node that is
                    //further away from the ray's origin.
                    if closest.is_empty(){
                        recc(axis.next(),second,func,ray,closest);
                    }
                },
                ray::Val::OneTouch(first)=>{
                    recc(axis.next(),first,func,ray,closest);
                },
            }
        }
        _ => {
            //If we are a leaf node, there are so little bots in here,
            //that lets just consider them all no matter what.
            closest.consider(&mut nn.range,func);
        }
    }

}

/*
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
*/