use inner_prelude::*;
use super::*;


//TODO use the property that the trees are sorted somehow.


pub fn raycast<
    'a,A:AxisTrait,
    T:SweepTrait,
    R:RayTrait<T=T,N=T::Num>
    >(tree:&'a mut DynTree<A,(),T>,mut ray:Ray<T::Num>,mut rtrait:R)->Option<(ColSingle<'a,T>,T::Num)>{
    
    let axis=A::new();
    //let ray=&Ray{point,dir};
    let dt = tree.get_iter_mut().with_depth(Depth(0));

    let mut closest=Closest{closest:None};
    recc(axis,dt,&mut rtrait,ray,&mut closest);

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
    
    //unimplemented!()
}



struct Closest<T:SweepTrait>{
    closest:Option<(*mut T,T::Num)>
}
impl<T:SweepTrait> Closest<T>{
    fn consider<R:RayTrait<T=T,N=T::Num>>(&mut self,depth:Depth,b:&mut T,raytrait:&mut R,ray:&Ray<T::Num>){

        let val={
            let (a,bb)=b.get_mut();
            let cc=ColSingle{inner:bb,rect:a};
            //func(cc)
            raytrait.compute_distance_bot(depth,cc)
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

/*
fn subdivide<A:AxisTrait,N:NumTrait>(r:&AABBox<N>,_axis:A,div:N)->(AABBox<N>,AABBox<N>){
    let (a,b)=r.0.subdivide(div,A::get());
    (AABBox(a),AABBox(b))
}


fn create_middile_box<A:AxisTrait,N:NumTrait>(r:&AABBox<N>,_axis:A,cont:Range<N>)->AABBox<N>{
    let mut r=r.clone();
    *r.0.get_range_mut(A::get())=cont;
    r
}
*/

use self::ray::Ray;
pub mod ray{
    use super::*;

    #[derive(Debug,Copy,Clone)]
    pub struct Ray<N:NumTrait>{
        pub point:[N;2],
        pub dir:[N;2],
        pub tlen:N,
    }


}




//TODO use this
pub trait RayTrait{
    type T:SweepTrait<Num=Self::N>;
    type N:NumTrait;

    fn compute_intersection_range<A:AxisTrait>(&mut self,fat_line:[Self::N;2])->Option<(Self::N,Self::N)>;

    fn compute_distance_to_line<A:AxisTrait>(&mut self,fat_line:Self::N)->Option<Self::N>;

    fn compute_distance_bot(&mut self,depth:Depth,ColSingle<Self::T>)->Option<Self::N>;


    fn split_ray<A:AxisTrait>(&mut self,ray1:&Ray<Self::N>,fo:Self::N)->Option<(Ray<Self::N>,Ray<Self::N>)>;

    //ray1 and ray2 are passed with the guarentee that they have the same direction.
    //fn add_ray(&mut self,ray1:&Ray<Self::N>,t_to_add:Self::N)->Ray<Self::N>;
    fn zero(&mut self)->Self::N;
}



//Returns the first object that touches the ray.
fn recc<'x,'a,
    N:NumTrait,
    A: AxisTrait,
    T: SweepTrait<Num=N>,
    R: RayTrait<T=T,N=N>
    >(axis:A,stuff:LevelIter<NdIterMut<(),T>>,rtrait:&mut R,ray:Ray<N>,closest:&mut Closest<T>){


    let ((depth,nn),rest)=stuff.next();
 

    match rest {
        Some((left, right)) => {
    
            let div=match nn.div{
                Some(div)=>div,
                None=>{
                    return  //There is nothing to consider in this node or any decendants.
                }
            };





            {
                let ray_point=if axis.is_xaxis(){
                    ray.point[0]
                }else{
                    ray.point[1]
                };


                let axis_next=axis.next();

                //We want to recurse the side that is closer to the origin.
                if ray_point<div{
                    
                    match rtrait.split_ray::<A>(&ray,div){
                        Some((ray_closer,ray_further))=>{
                            recc(axis_next,left,rtrait,ray_closer,closest);
                            //recc(axis_next,right,rtrait,ray_further,closest);
                            dop(axis,right,rtrait,ray_further,closest,div);
                        },
                        None=>{
                            //The ray isnt long enough to touch the divider.
                            //So just recurse the one side.
                            recc(axis_next,left,rtrait,ray,closest);
                        }
                    }
                
                }else{
                    match rtrait.split_ray::<A>(&ray,div){
                        Some((ray_closer,ray_further))=>{
                            recc(axis_next,right,rtrait,ray_closer,closest);
                            //recc(axis_next,left,rtrait,ray_further,closest);
                            dop(axis,left,rtrait,ray_further,closest,div);
                        },
                        None=>{
                            recc(axis_next,right,rtrait,ray,closest);
                        }
                    }
                
                }  
            };
            
            fn dop<
                N:NumTrait,
                A: AxisTrait,
                T: SweepTrait<Num=N>,
                R: RayTrait<T=T,N=N>
                >(axis:A,nd:LevelIter<NdIterMut<(),T>>,rtrait:&mut R,ray:Ray<N>,closest:&mut Closest<T>,div:N){
                match closest.get_dis(){
                    Some(dis)=>{
                        match rtrait.compute_distance_to_line::<A>(div){
                            Some(dis2)=>{
                                if dis2<dis{
                                    recc(axis.next(),nd,rtrait,ray,closest);
                                }else{
                                    //We get to skip here

                                    //recc(axis.next(),second,rtrait,ray,closest);
                                }
                            },
                            None=>{
                                //panic!("IMPOSSIBLE!!!!");
                                //Ray doesnt intersect other side
                            }
                        }
                    },
                    None=>{
                        recc(axis.next(),nd,rtrait,ray,closest);
                    }
                }
            }


                        
            //Check this node only after recursing children.
            //We recurse first since this way we might find out we dont need to recurse the children in this node.
            //Its more likely that we will find the closest bot in a child node
            match &nn.cont{
                &Some(cont)=>{
                    let ff=[cont.left(),cont.right()];


                    let ray_point=if axis.is_xaxis(){
                        ray.point[0]
                    }else{
                        ray.point[1]
                    };

                    let handle_middle=if ray_point>=ff[0] && ray_point<=ff[1]{
                        (true,42)
                    }else{

                        let (closer_line,further_line)=if axis.is_xaxis(){
                            if ray.point[0]<div{
                                (ff[0],ff[1])
                            }else{
                                (ff[1],ff[0])
                            }
                        }else{
                            if ray.point[1]<div{
                                (ff[0],ff[1])
                            }else{
                                (ff[1],ff[0])
                            }
                        };

                        match closest.get_dis(){
                            Some(dis)=>{

                                let closest_possible=match rtrait.compute_distance_to_line::<A>(closer_line){
                                    Some(dis2)=>{
                                        dis2
                                    },
                                    None=>{
                                        rtrait.zero()
                                    }
                                };

                                if closest_possible<dis{
                                    (true,0)
                                }else{
                                    if depth.0==0{
                                        println!("{:?}",(closest_possible,dis));
                                    }
                                    (false,1)
                                }
                            },
                            None=>{
                                (true,2)
                            }
                        }
                    };

                    if depth.0==0{
                        println!("{:?}",handle_middle);
                    }

                    if handle_middle.0{
                        
                        match rtrait.compute_intersection_range::<A>(ff){
                        //match compute_intersection_range_converted::<A,_>(rtrait,ff,&ray){
                            //(InSome(a),Some(b))=>{
                            Some((a,b))=>{
                                if depth.0==0{
                                    println!("ab={:?}",(a,b));
                                }
                                for (i,bot) in nn.range.iter_mut().enumerate(){
                                    
                                    let rang=*((bot.get().0).0).get_range2::<A::Next>();
                                    if rang.left()>b{
                                        break;
                                    }
                                    
                                    if rang.right()>=a{
                                        closest.consider(depth,bot,rtrait,&ray);
                                    }
                                    
                                }
                            },
                            None=>{
                                //Do nothing
                            }
                        }
                    } 
                },
                &None=>{
                    //This node doesnt have any bots
                }
            }
        }
        _ => {
            //TODO do better here
            for b in nn.range.iter_mut(){
                closest.consider(depth,b,rtrait,&ray);
            }
        }
    }

}

