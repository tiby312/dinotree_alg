use inner_prelude::*;
use super::*;


//TODO use the property that the trees are sorted somehow.


pub fn raycast<
    'a,A:AxisTrait,
    T:SweepTrait,
    MF:FnMut(ColSingle<T>)->Option<T::Num>, //called to test if this object touches the ray. if it does, return distance to start of ray
    MFFast:FnMut(&AABBox<T::Num>)->Option<T::Num>,
    >(tree:&'a mut DynTree<A,(),T>,point:[T::Num;2],dir:[T::Num;2],mut func:MF,mut func_fast:MFFast,rect:AABBox<T::Num>)->Option<(ColSingle<'a,T>,T::Num)>{
    let axis=A::new();
    let ray=&Ray{point,dir};
    let dt = tree.get_iter_mut();

    let mut closest=Closest{closest:None};
    recc(axis,dt,&mut func,&mut func_fast,ray,&mut closest,rect);

    match closest.closest{
        Some(x)=>{
            let bb=unsafe{&mut *x.0};
            let rr=bb.get_mut();
            let cc=ColSingle{inner:rr.1,rect:rr.0};
            Some((cc,x.1))
            //mf2(cc,x.1);
        },
        None=>{
            None
        }
    }
}



struct Closest<T:SweepTrait>{
    closest:Option<(*mut T,T::Num)>
}
impl<T:SweepTrait> Closest<T>{
    fn consider<MF:FnMut(ColSingle<T>)->Option<T::Num>>(&mut self,bots:&mut [T],func:&mut MF){

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

    fn check_and_do<MFFast:FnMut(&AABBox<T::Num>)->Option<T::Num>>(&self,rect:&AABBox<T::Num>,func:&mut MFFast)->bool{
        match func(rect){
            Some(closest_possible)=>{
                match self.get_dis(){
                    Some(dis)=>{
                        if closest_possible<dis{
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
                false
            }
        }
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


fn subdivide<A:AxisTrait,N:NumTrait>(r:&AABBox<N>,_axis:A,div:N)->(AABBox<N>,AABBox<N>){
    let (a,b)=r.0.subdivide(div,A::get());
    (AABBox(a),AABBox(b))
}


fn create_middile_box<A:AxisTrait,N:NumTrait>(r:&AABBox<N>,_axis:A,cont:Range<N>)->AABBox<N>{
    let mut r=r.clone();
    *r.0.get_range_mut(A::get())=cont;
    r
}

use self::ray::Ray;
pub mod ray{
    use super::*;

    #[derive(Copy,Clone)]
    pub struct Ray<N:NumTrait>{
        pub point:[N;2],
        pub dir:[N;2],
    }
}
/*
pub trait RayTrait{
    fn compute_distance(&)
}*/


//Returns the first object that touches the ray.
fn recc<'x,'a,
    N:NumTrait,
    A: AxisTrait,
    T: SweepTrait<Num=N> + 'x,
    C: CTreeIterator<Item = &'x mut NodeDyn<(),T>>,
    MF:FnMut(ColSingle<T>)->Option<N>, //User returns distance to ray origin if it collides with ray
    MFFast:FnMut(&AABBox<N>)->Option<N>,
    >(axis:A,stuff:C,func:&mut MF,func_fast:&mut MFFast,ray:&Ray<N>,closest:&mut Closest<T>,rectinf:AABBox<T::Num>){


    let (nn,rest)=stuff.next();
 

    match rest {
        Some((left, right)) => {
    
            let div=match nn.div{
                Some(div)=>div,
                None=>{
                    return  //There is nothing to consider in this node or any decendants.
                }
            };

            //We want to recurse the side that is closer to the origin of the ray.
            let ((left,right),(aa,bb))={
                let (aa,bb)=subdivide(&rectinf,axis,div);

                let ray_point=if axis.is_xaxis(){
                    ray.point[0]
                }else{
                    ray.point[1]
                };

                if ray_point<div{
                    ((left,right),(aa,bb))
                }else{
                    ((right,left),(bb,aa))
                }
            };

            if closest.check_and_do(&aa,func_fast){
                recc(axis.next(),left,func,func_fast,ray,closest,aa);
            }else{
                //panic!("impossible");
            }

            if closest.check_and_do(&bb,func_fast){
                recc(axis.next(),right,func,func_fast,ray,closest,bb);
            }

            
            //Check this node only after recursing children.
            match &nn.cont{
                &Some(cont)=>{
                    /*
                    let closer_div=cont.left or right;
                    match ray.compute_intersection_point(axis.next(),closer_div){
                        Some(point)=>{
                            for b in bots{
                                if b.left>point{
                                    break;
                                }
                                if b.right>=point{
                                    closest.consider(b,func);
                                }
                            }
                        },
                        None=>{

                        }
                    }*/

                    
                    let mid=create_middile_box(&rectinf,axis,cont);
                    if closest.check_and_do(&mid,func_fast){
                        closest.consider(&mut nn.range,func);
                    }
                    
                },
                &None=>{
                    //This node doesnt have any bots
                }
            }
        }
        _ => {
            if closest.check_and_do(&rectinf,func_fast){
                closest.consider(&mut nn.range,func);
            }
        }
    }

}
