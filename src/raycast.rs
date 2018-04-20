use inner_prelude::*;
use super::*;

#[derive(Copy,Clone)]
pub struct Vec2<N:Copy>{
    pub x:N,
    pub y:N
}
impl<N:Copy> Vec2<N>{
    pub fn get<A:AxisTrait>(&self,axis:A)->N{
        if axis.is_xaxis(){
            self.x
        }else{
            self.y
        }
    }
}

pub fn raycast<
    'a,A:AxisTrait,
    T:SweepTrait,
    MF:FnMut(ColSingle<T>)->Option<T::Num>, //called to test if this object touches the ray. if it does, return distance to start of ray
    MFFast:FnMut(&RectInf<T::Num>)->Option<T::Num>,
    >(tree:&'a mut DynTree<A,T>,ray:&Ray<T::Num>,mut func:MF,mut func_fast:MFFast,rect:RectInf<T::Num>)->Option<(ColSingle<'a,T>,T::Num)>{


    let dt = tree.get_iter_mut();

    let mut closest=Closest{closest:None};
    recc(A::new(),dt,&mut func,&mut func_fast,ray,&mut closest,rect);

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

    fn check_and_do<MFFast:FnMut(&RectInf<T::Num>)->Option<T::Num>>(&self,rect:&RectInf<T::Num>,func:&mut MFFast)->bool{
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

pub struct RectInf<N:NumTrait>{
    pub xdiv:(N,N),
    pub ydiv:(N,N)
}
impl<N:NumTrait> RectInf<N>{

    fn subdivide<A:AxisTrait>(&self,axis:A,div:N)->(RectInf<N>,RectInf<N>){
        if axis.is_xaxis(){
            let r1=RectInf{xdiv:(self.xdiv.0,div),ydiv:self.ydiv};
            let r2=RectInf{xdiv:(div,self.xdiv.1),ydiv:self.ydiv};
            (r1,r2)
        }else{
            let r1=RectInf{xdiv:self.xdiv,ydiv:(self.ydiv.0,div)};
            let r2=RectInf{xdiv:self.xdiv,ydiv:(div,self.ydiv.1)};
            (r1,r2)
        }
    }


    fn create_middile_box<A:AxisTrait>(&self,axis:A,cont:Range<N>)->RectInf<N>{
        if axis.is_xaxis(){
            RectInf{xdiv:(cont.start,cont.end),ydiv:self.ydiv}
        }else{
            RectInf{xdiv:self.xdiv,ydiv:(cont.start,cont.end)}
        }
    }
}


use self::ray::Ray;
//use self::ray::RayTrait;
pub mod ray{
    use super::*;


    //A finite ray
    #[derive(Copy,Clone)]
    pub struct Ray<N:NumTrait>{
        pub point:Vec2<N>,
        pub dir:Vec2<N>,
    }

    pub enum Val<X>{
        BothTouch((X,X)),
        OneTouch(X),
    }
}


//Returns the first object that touches the ray.
fn recc<'x,'a,
    N:NumTrait,
    A: AxisTrait,
    T: SweepTrait<Num=N> + 'x,
    C: CTreeIterator<Item = &'x mut NodeDyn<T>>,
    MF:FnMut(ColSingle<T>)->Option<N>, //User returns distance to ray origin if it collides with ray
    MFFast:FnMut(&RectInf<N>)->Option<N>,
    >(axis:A,stuff:C,func:&mut MF,func_fast:&mut MFFast,ray:&Ray<N>,closest:&mut Closest<T>,rectinf:RectInf<T::Num>){


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
                let (aa,bb)=rectinf.subdivide(axis,div);

                let ray_point=if axis.is_xaxis(){
                    ray.point.x
                }else{
                    ray.point.y
                };

                if ray_point<div{
                    ((left,right),(aa,bb))
                }else{
                    ((right,left),(bb,aa))
                }
            };

            if closest.check_and_do(&aa,func_fast){
                recc(axis.next(),left,func,func_fast,ray,closest,aa);
            }

            if closest.check_and_do(&bb,func_fast){
                recc(axis.next(),right,func,func_fast,ray,closest,bb);
            }

            
            //Check this node only after recursing children.
            match &nn.cont{
                &Some(cont)=>{
                    let mid=rectinf.create_middile_box(axis,cont);
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

