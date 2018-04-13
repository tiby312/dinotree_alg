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
    recc(A::new(),dt,&func,&ray,&mut closest);

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
    }

    pub enum Val<X>{
        BothTouch((X,X)),
        OneTouch(X),
    }

    pub trait RayTrait{
        type N:NumTrait;
        fn intersects_divider<A:AxisTrait,X>(&self,axis:A,div:Self::N,left:X,right:X)->Val<X>;
        fn closest_distance_to_cyclinder<A:AxisTrait>(&self,axis:A,cont:Range<Self::N>)->Self::N;
    }
    impl RayTrait for Ray<isize>{
        type N=isize;
        //visit all kd tree nodes intersected by segment S=a+t*d,0<=t
        fn intersects_divider<A:AxisTrait,X>(&self,axis:A,div:isize,left:X,right:X)->Val<X>{
            let point=self.point;
            let dir=self.dir;

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
                return Val::OneTouch(first);
            }

            let t=(div-point.get(axis))/dir.get(axis);

            if t>0{
                Val::BothTouch((first,second))
            }else{
                return Val::OneTouch(first);
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
    A: AxisTrait,
    T: SweepTrait + 'x,
    C: CTreeIterator<Item = &'x mut NodeDyn<T>>,
    MF:Fn(ColSingle<T>)->Option<T::Num>, //User returns distance to ray origin if it collides with ray
    R:RayTrait<N=T::Num>,
    >(axis:A,stuff:C,func:&MF,ray:&R,closest:&mut Closest<T>){


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



#[cfg(test)]
mod test{
    use super::*;
    use test_support::*;
    use support::BBox;
    use test::*;


    #[test]
    fn test_raycast(){
        fn from_point(a:isize,b:isize)->AABBox<isize>{
            AABBox::new((a-10,a+10),(b-10,b+10))
        }

        let mut bots=Vec::new();
        bots.push(BBox::new(Bot::new(0),from_point(-30,0)));
        bots.push(BBox::new(Bot::new(1),from_point(30,0)));
        bots.push(BBox::new(Bot::new(2),from_point(0,-100)));

        let ray=Ray{point:Vec2{x:0,y:0},dir:Vec2{x:0,y:-1}};

        //https://tavianator.com/fast-branchless-raybounding-box-intersections/

        let ray_touch_box=|a:ColSingle<BBox<isize,Bot>>|->Option<isize>{
            let ((x1,x2),(y1,y2))=a.rect.get();
            let point=ray.point;
            let dir=ray.dir;
 
            //top and bottom
            //s(t)=point+t*dir
            let mut tmin=isize::min_value();
            let mut tmax=isize::max_value();

            if dir.x!=0{
                let tx1=(x1-point.x)/dir.x;
                let tx2=(x2-point.x)/dir.x;

                tmin=tmin.max(tx1.min(tx2));
                tmax=tmax.min(tx1.max(tx2));
                
            }
            if dir.y!=0{
                let ty1=(y1-point.y)/dir.y;
                let ty2=(y2-point.y)/dir.y;

                tmin=tmin.max(ty1.min(ty2));
                tmax=tmax.min(ty1.max(ty2));
            }
            println!("max min ={:?}",(tmin,tmax));
            if tmax>=tmin && tmin>=0{
                println!("TOUCH!");
                return Some(tmin);
            }
            
            return None
        };


        {
            let mut dyntree = DinoTree::new(&mut bots,  StartAxis::Yaxis);
            let k=dyntree.raycast(ray,ray_touch_box).expect("nothing hit the ray!");
            println!("{:?}",k.0.inner);
            assert!(false);
        }


    }

}