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
    R:RayTrait<N=T::Num>
    >(tree:&'a mut DynTree<A,T>,ray:R,mut func:MF)->Option<(ColSingle<'a,T>,T::Num)>{


    let dt = tree.get_iter_mut();

    //let ray=Ray{point,dir};
    let mut closest=Closest{closest:None};
    recc(A::new(),dt,&mut func,ray,&mut closest);

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
use self::ray::RayTrait;
pub mod ray{
    use super::*;


    //A finite ray
    #[derive(Copy,Clone)]
    pub struct Ray<N:NumTrait>{
        pub point:Vec2<N>,
        pub dir:Vec2<N>,
        pub tmax:Option<N>
    }

    pub enum Val<X>{
        BothTouch((X,X)),
        OneTouch(X),
    }

    pub trait RayTrait:Sized{
        type N:NumTrait;
        fn intersects_divider<A:AxisTrait,X>(&self,axis:A,div:Self::N,left:X,right:X)->Val<(Self,X)>;

        fn closest_distance_to_cyclinder<A:AxisTrait>(&self,axis:A,cont:Range<Self::N>)->Option<Self::N>;
    }
    impl RayTrait for Ray<isize>{
        type N=isize;
        //visit all kd tree nodes intersected by segment S=a+t*d,0<=t
        fn intersects_divider<A:AxisTrait,X>(&self,axis:A,div:isize,left:X,right:X)->Val<(Ray<isize>,X)>{
            let point=self.point;
            let dir=self.dir;
            let tmax=self.tmax;
            //s=a+t*d
            //s-a=t*d
            //(s-a)/d=t


            //s(t)=point+t*dir
            //we want to figure out what t is when the ray has the same value as the divider
            //for the irght axis.
            //so if this were a divider that was splitting alone the xaxis, we'd have:
            // div=point.x+t*dir.x
            // t=(div-point.x)/dir.x
            //
            //So clearly dir.x cant be zero.
            //What would it mean if dir.x is zero?
            //it means that in the equation div=point.x+t*dir.x, the t*dir.x term disappears.
            //so the only way div=point.x is if the point is directly on the point. 


            let (first,second)=if point.get(axis)<div{
                (left,right)
            }else{
                (right,left)
            };

            if dir.get(axis)==0{
                return Val::OneTouch((*self,first));
            }

            let t=(div-point.get(axis))/dir.get(axis);
            /*
            if t>0{
                let r1=Ray{point,dir,tmax:None};

                let r2=Ray{point,dir,tmax:None};
                let r1=(r1,first);
                let r2=(r2,second);
                Val::BothTouch((r1,r2))
            }else{
                Val::OneTouch((*self,first))
            }*/
            
            match tmax{
                Some(tmax)=>{
                    if t>0 && t<tmax{
                        let r1=Ray{point,dir,tmax:Some(t)};
                        let newpx=point.x+t*dir.x;
                        let newpy=point.y+t*dir.y;
                        let newp=Vec2{x:newpx,y:newpy};
                        let r2=Ray{point:newp,dir,tmax:Some(tmax-t)};
                        let r1=(r1,first);
                        let r2=(r2,second);
                        Val::BothTouch((r1,r2))
                    }else{
                        Val::OneTouch((*self,first))
                    }
                },
                None=>{
                    if t>0{
                        let r1=Ray{point,dir,tmax:Some(t)};
                        let newpx=point.x+t*dir.x;
                        let newpy=point.y+t*dir.y;
                        let newp=Vec2{x:newpx,y:newpy};
                        let r2=Ray{point:newp,dir,tmax:None};
                        let r1=(r1,first);
                        let r2=(r2,second);
                        Val::BothTouch((r1,r2))
                    }else{
                        Val::OneTouch((*self,first))
                    }
                }
            }
        }

        //Returns the closest possible tvalue to something that intersects this divider.

        fn closest_distance_to_cyclinder<A:AxisTrait>(&self,axis:A,cont:Range<isize>)->Option<isize>{
            let point=self.point;
            let dir=self.dir;



            let div=if point.get(axis)<cont.start{  //TODO less and equal?
                cont.start
            }else if point.get(axis)>cont.end{ //TODO less and equal?
                cont.end
            }else{
                return Some(0); //point is inside the range, possible that something touches
            };


            if dir.get(axis)==0{
                //We already know from the above, that div is not inside of the range.
                //So this case means that the ray is running parallel to the range, but not inside it,
                //so not possible that it touches.
                return None;
            }

            let t=(div-point.get(axis))/dir.get(axis);
            if t<0{
                return None;
            }
            return Some(t);
        }
    }

}


//Returns the first object that touches the ray.
fn recc<'x,'a,
    N:NumTrait,
    A: AxisTrait,
    T: SweepTrait<Num=N> + 'x,
    C: CTreeIterator<Item = &'x mut NodeDyn<T>>,
    MF:FnMut(ColSingle<T>)->Option<N>, //User returns distance to ray origin if it collides with ray
    R:RayTrait<N=N>,
    >(axis:A,stuff:C,func:&mut MF,ray:R,closest:&mut Closest<T>){


    let (nn,rest)=stuff.next();


    match rest {
        Some((left, right)) => {
    
            let div=match nn.div{
                Some(div)=>div,
                None=>{
                    return  //There is nothing to consider in this node or any decendants.
                }
            };

            let (ray,second)=match ray.intersects_divider(axis,div,left,right){
                ray::Val::BothTouch(((ray1,first),(ray2,second)))=>{
                    //Its more likely that we'll find the closest bot in a children, than this node.
                    //This is because only the bots that intersect with this node are kept here.
                    //Many more bots exist in the lower you o in the tree.
                    //So because of this, lets recurse first, before we check this node.
                    recc(axis.next(),first,func,ray1,closest);

                    (ray2,second)
                },
                ray::Val::OneTouch((ray,first))=>{
                    //TODO is this needed?
                    //closest.consider(&mut nn.range,func);  
                    
                    //recc(axis.next(),first,func,ray,closest);
                    (ray,first)
                },
            };

            //Only bother considering the bots in this node,
            
            match nn.cont{
                Some(cont)=>{
                    match ray.closest_distance_to_cyclinder(axis,cont){
                        Some(closest_possible)=>{
                            match closest.get_dis(){
                                Some(dis)=>{
                                    if closest_possible<=dis{
                                        closest.consider(&mut nn.range,func);
                                    }
                                },
                                None=>{
                                    //We have to check them all since there isnt a closest 
                                    closest.consider(&mut nn.range,func);
                                }
                            }
                        },
                        None=>{
                            //Impossible for anything in this node to touch the ray
                        }
                    }
                },
                None=>{
                    //This node doesnt have anything.
                }
            }
            
            /*
            //Only bother considering the bots in this node,
            match closest.get_dis(){
                Some(dis)=>{
                    match nn.cont{
                        Some(cont)=>{
                            match ray.closest_distance_to_cyclinder(axis,cont){
                                Some(closest_possible)=>{
                                    if closest_possible<=dis{
                                        closest.consider(&mut nn.range,func);
                                    }
                                },
                                None=>{}
                            } 
                                    
                        },
                        None=>{
                            //this node does not have any nodes, so dont need to consider
                            //anything in this node.
                        }
                    }
                },
                None=>{
                    match nn.cont{
                        Some(cont)=>{
                            match ray.closest_distance_to_cyclinder(axis,cont){
                                Some(_)=>{
                                    
                                    closest.consider(&mut nn.range,func);  
                                },
                                None=>{}
                            } 
                        },
                        None=>{

                        }
                    }
                }
            }
            */

            //So only in the case where we literally could not find a single bot 
            //that intersected the ray, do we recurse the side of the node that is
            //further away from the ray's origin.
            if closest.is_empty(){ //TODO 
                recc(axis.next(),second,func,ray,closest);
            }

        }
        _ => {
            //If we are a leaf node, there are so little bots in here,
            //that lets just consider them all no matter what.
            closest.consider(&mut nn.range,func);
        }
    }

}

