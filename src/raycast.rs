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
    >(tree:&'a mut DynTree<A,T>,ray:R,mut func:MF,rect:RectInf<T::Num>)->Option<(ColSingle<'a,T>,T::Num)>{


    let dt = tree.get_iter_mut();

    //let ray=Ray{point,dir};
    let mut closest=Closest{closest:None};
    //let rect=RectInf{xdiv:(None,None),ydiv:(None,None)};
    recc(A::new(),dt,&mut func,ray,&mut closest,rect);

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

    fn check_and_do<R:RayTrait<N=T::Num>>(&self,ray:&R,rect:&RectInf<R::N>)->bool{
        match ray.intersects_box(rect){
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

    pub trait RayTrait:Sized+Copy{
        type N:NumTrait;
        fn get_point(&self)->(Self::N,Self::N);
        //fn intersects_divider<A:AxisTrait,X>(&self,axis:A,div:Self::N,left:X,right:X)->Val<(Self,X)>;
        //fn closest_distance_to_gradient<A:AxisTrait>(&self,axis:A,div:(Self::N,bool))->Option<Self::N>;
        //fn closest_distance_to_cont<A:AxisTrait>(&self,axis:A,cont:Range<Self::N>)->Option<Self::N>;
        fn intersects_box(&self,a:&RectInf<Self::N>)->Option<Self::N>;
    }
    impl RayTrait for Ray<isize>{
        type N=isize;

        fn get_point(&self)->(Self::N,Self::N){
            (self.point.x,self.point.y)
        }
        fn intersects_box(&self,rect:&RectInf<Self::N>)->Option<Self::N>{
            let ((x1,x2),(y1,y2))=(rect.xdiv,rect.ydiv);


            let point=self.point;
            let dir=self.dir;


            //top and bottom
            //s(t)=point+t*dir
            let mut tmin=isize::min_value();
            let mut tmax=isize::max_value();

            if dir.x!=0{
                let tx1=(x1-point.x)/dir.x;
                let tx2=(x2-point.x)/dir.x;

                tmin=tmin.max(tx1.min(tx2));
                tmax=tmax.min(tx1.max(tx2));
                
            }else{
                if point.x < x1 || point.x > x2 {
                    return None; // parallel AND outside box : no intersection possible
                }
            }
            if dir.y!=0{
                let ty1=(y1-point.y)/dir.y;
                let ty2=(y2-point.y)/dir.y;

                tmin=tmin.max(ty1.min(ty2));
                tmax=tmax.min(ty1.max(ty2));
            }else{
                if point.y < y1 || point.y > y2 {
                    return None; // parallel AND outside box : no intersection possible
                }
            }
            if tmax>=tmin && tmax>=0{
                //println!("bla=max:{:?} min:{:?}",tmax,tmin);
                return Some(tmin.max(0));
            }else{
                return None;
            }
                        
        }
        /*
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
            
            if t>0{
                let r1=Ray{point,dir,tmax:None};

                let r2=Ray{point,dir,tmax:None};
                let r1=(r1,first);
                let r2=(r2,second);
                Val::BothTouch((r1,r2))
            }else{
                Val::OneTouch((*self,first))
            }
            /*
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
            */
        }*/
        /*
        fn get_origin(&self)->&Vec2<isize>{
            &self.point
        }
        //Returns the closest possible tvalue to something that intersects this divider.


        fn closest_distance_to_cont<A:AxisTrait>(&self,axis:A,cont:Range<isize>)->Option<isize>{
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

        //bool is true if the gradient is left of the div
        fn closest_distance_to_gradient<A:AxisTrait>(&self,axis:A,div:(isize,bool))->Option<isize>{
            let point=self.point;
            let dir=self.dir;

            if div.1{
                //TODO this is the left.

                if point.get(axis)<div.0{
                    //The ray is starting inside this gradient.
                    Some(0)
                }else{
                    if dir.get(axis)==0{
                        None
                    }else{

                        let t=(div.0-point.get(axis))/dir.get(axis);
                        if t<0{
                            None
                        }else{
                            Some(t)
                        }
                    }
                }
            }else{
                if point.get(axis)>div.0{
                    //The ray is starting inside this gradient.
                    Some(0)
                }else{
                    if dir.get(axis)==0{
                        None
                    }else{

                        let t=(div.0-point.get(axis))/dir.get(axis);
                        if t<0{
                            None
                        }else{
                            Some(t)
                        }
                    }
                } 
            }
        }
        */
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
    >(axis:A,stuff:C,func:&mut MF,ray:R,closest:&mut Closest<T>,rectinf:RectInf<T::Num>){


    let (nn,rest)=stuff.next();
 

    match rest {
        Some((left, right)) => {
    
            let div=match nn.div{
                Some(div)=>div,
                None=>{
                    return  //There is nothing to consider in this node or any decendants.
                }
            };

            let ((left,right),(aa,bb))={
                let (aa,bb)=rectinf.subdivide(axis,div);

                let ray_point=if axis.is_xaxis(){
                    ray.get_point().0
                }else{
                    ray.get_point().1
                };

                if ray_point<div{
                    ((left,right),(aa,bb))
                }else{
                    ((right,left),(bb,aa))
                }
            };

            if closest.check_and_do(&ray,&aa){
                recc(axis.next(),left,func,ray,closest,aa);
            }

            if closest.check_and_do(&ray,&bb){
                recc(axis.next(),right,func,ray,closest,bb);
            }

            
            //Check this node only after recursing children.
            match &nn.cont{
                &Some(cont)=>{
                    let mid=rectinf.create_middile_box(axis,cont);
                    if closest.check_and_do(&ray,&mid){
                        closest.consider(&mut nn.range,func);
                    }
                },
                &None=>{
                    //This node doesnt have any bots
                }
            }
        }
        _ => {
            if closest.check_and_do(&ray,&rectinf){
                closest.consider(&mut nn.range,func);
            }
        }
    }

}

