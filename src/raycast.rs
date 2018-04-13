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
    MF:Fn(ColSingle<T>)->Option<T::Num>, //called to test if this object touches the ray. if it does, return distance to start of ray
    R:RayTrait<N=T::Num>
    >(tree:&'a mut DynTree<A,T>,ray:R,mut func:MF)->Option<(ColSingle<'a,T>,T::Num)>{


    let dt = tree.get_iter_mut();

    //let ray=Ray{point,dir};
    let mut closest=Closest{closest:None};
    recc(A::new(),dt,&func,ray,&mut closest);

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

        fn closest_distance_to_cyclinder<A:AxisTrait>(&self,axis:A,cont:Range<Self::N>)->Self::N;
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

        //Returns the closest possible distance between the ray, 
        //and the two parallel lines.
        fn closest_distance_to_cyclinder<A:AxisTrait>(&self,axis:A,cont:Range<isize>)->isize{
            let point=self.point;
            let dir=self.dir;
            if point.get(axis)<cont.start{
                if dir.get(axis)>0{
                    return 0;
                }else{
                    return point.get(axis);
                }
            }else if point.get(axis)>cont.end{
                if dir.get(axis)<0{
                    return 0;
                }else{
                    return point.get(axis);
                }
            }else{
                return 0
            };
        }
    }

}


//Returns the first object that touches the ray.
fn recc<'x,'a,
    N:NumTrait,
    A: AxisTrait,
    T: SweepTrait<Num=N> + 'x,
    C: CTreeIterator<Item = &'x mut NodeDyn<T>>,
    MF:Fn(ColSingle<T>)->Option<N>, //User returns distance to ray origin if it collides with ray
    R:RayTrait<N=N>,
    >(axis:A,stuff:C,func:&MF,ray:R,closest:&mut Closest<T>){


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
                ray::Val::BothTouch(((ray1,first),(ray2,second)))=>{
                    //Its more likely that we'll find the closest bot in a children, than this node.
                    //This is because only the bots that intersect with this node are kept here.
                    //Many more bots exist in the lower you o in the tree.
                    //So because of this, lets recurse first, before we check this node.
                    recc(axis.next(),first,func,ray1,closest);

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
                        recc(axis.next(),second,func,ray2,closest);
                    }
                },
                ray::Val::OneTouch((ray,first))=>{
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

