use support::prelude::*;
use dinotree::raycast;
use std;
mod ray_f64{
    use super::*;

    use self::raycast::Ray;
    use self::raycast::RayTrait;

    fn intersects_box(point:[f64N;2],dir:[f64N;2],matt:f64N,rect:&axgeom::Rect<f64N>)->Option<f64N>{
        let ((x1,x2),(y1,y2))=rect.get();

        let x1=x1.into_inner();
        let x2=x2.into_inner();
        let y1=y1.into_inner();
        let y2=y2.into_inner();

        let mut tmin=std::f64::MIN;
        let mut tmax=std::f64::MAX;

        let point=[point[0].into_inner(),point[1].into_inner()];
        let dir=[dir[0].into_inner(),dir[1].into_inner()];

        if dir[0]!=0.0{
            let tx1=(x1-point[0])/dir[0];
            let tx2=(x2-point[0])/dir[0];

            tmin=tmin.max(tx1.min(tx2));
            tmax=tmax.min(tx1.max(tx2));
            
        }else{
            if point[0] < x1 || point[0] > x2 {
                return None; // parallel AND outside box : no intersection possible
            }
        }

        if dir[1]!=0.0{
            let ty1=(y1-point[1])/dir[1];
            let ty2=(y2-point[1])/dir[1];

            tmin=tmin.max(ty1.min(ty2));
            tmax=tmax.min(ty1.max(ty2));
        }else{
            if point[1] < y1 || point[1] > y2 {
                return None; // parallel AND outside box : no intersection possible
            }
        }
        if tmax>=tmin && tmax>=0.0 && tmin<=matt.into_inner(){
            return Some(NotNaN::new(tmin.max(0.0)).unwrap());
        }else{
            return None;
        }                
    }

    pub struct RayT<'a,'c:'a>{
        pub ray:Ray<f64N>,
        pub c:&'a Context,
        pub g:&'a mut G2d<'c>
    }

    fn compute_intersection_point<A:axgeom::AxisTrait>(axis:A,ray:&Ray<f64N>,line:f64N)->Option<(f64N,f64N)>{
        if axis.is_xaxis(){
            //line=ray.point[0]+t*ray.dir[0];
            if ray.dir[0].into_inner()==0.0{
                if ray.point[0]==line{
                    Some((NotNaN::new(0.0).unwrap(),ray.point[1]))
                }else{
                    None
                }
            }else{
                let t=(line-ray.point[0])/ray.dir[0];
                
                if t.into_inner()>=0.0 && t<=ray.tlen{
                    Some((t,ray.point[1]+ray.dir[1]*t))
                }else{
                    None
                }
            }
        }else{
            if ray.dir[1].into_inner()==0.0{
                if ray.point[1]==line{
                    Some((NotNaN::new(0.0).unwrap(),ray.point[0]))
                }else{
                    None
                }
            }else{
                let t=(line-ray.point[1])/ray.dir[1];
                if t.into_inner()>=0.0 && t<=ray.tlen{
                    Some((t,ray.point[0]+ray.dir[0]*t))
                }else{
                    None
                }
            }
            
        }
    }

    impl<'a,'c:'a> RayTrait for RayT<'a,'c>{
        type T=BBox<f64N,Bot>;
        type N=f64N;

        fn split_ray<A:axgeom::AxisTrait>(&mut self,axis:A,ray:&Ray<Self::N>,fo:Self::N)->Option<(Ray<Self::N>,Ray<Self::N>)>{
            let t=if axis.is_xaxis(){
                if ray.dir[0].into_inner()==0.0{
                    if ray.point[0]==fo{
                        let t1=ray.tlen/2.0;
                        let t2=ray.tlen-t1;
                        //Lets just split it into half.
                        let ray_closer=Ray{point:ray.point,dir:ray.dir,tlen:t1};
                        let new_point=[ray.point[0],ray.point[1]+t1];
                        let ray_new=Ray{point:new_point,dir:ray.dir,tlen:t2};

                        return Some((ray_closer,ray_new))
                    }else{
                        return None
                    }
                }else{
                    (fo-ray.point[0])/ray.dir[0]
                }
            }else{
                if ray.dir[1].into_inner()==0.0{
                    if ray.point[1]==fo{
                        let t1=ray.tlen/2.0;
                        let t2=ray.tlen-t1;
                        //Lets just split it into half.
                        let ray_closer=Ray{point:ray.point,dir:ray.dir,tlen:t1};
                        let new_point=[ray.point[0]+t1,ray.point[1]];
                        let ray_new=Ray{point:new_point,dir:ray.dir,tlen:t2};

                        return Some((ray_closer,ray_new))
                    }else{
                        return None
                    }
                }else{
                    (fo-ray.point[1])/ray.dir[1]   
                }
            };

            if t>ray.tlen || t.into_inner()<0.0{
                return None
            }
            //assert!(t<=ray.tlen);

            let new_point=[ray.point[0]+ray.dir[0]*t,ray.point[1]+ray.dir[1]*t];
            

            let ray_closer=Ray{point:ray.point,dir:ray.dir,tlen:t,/*true_len:ray.true_len-ray.tlen+t*/};
            let ray_new=Ray{point:new_point,dir:ray.dir,tlen:ray.tlen-t,/*true_len:ray.true_len*/};
            Some((ray_closer,ray_new))
            
        }

        //First option is min, second is max
        fn compute_intersection_range<A:axgeom::AxisTrait>(&mut self,axis:A,fat_line:[Self::N;2])->Option<(Self::N,Self::N)>
        {
            let o1:Option<(Self::N,Self::N)>=compute_intersection_point(axis,&self.ray,fat_line[0]);
            let o2:Option<(Self::N,Self::N)>=compute_intersection_point(axis,&self.ray,fat_line[1]);

            let o1=o1.map(|a|a.1);
            let o2=o2.map(|a|a.1);

            let [ray_origin_x,ray_origin_y,ray_end_y]=if axis.is_xaxis(){
                [self.ray.point[0],self.ray.point[1],self.ray.point[1]+self.ray.tlen*self.ray.dir[1]]
            }else{
                [self.ray.point[1],self.ray.point[0],self.ray.point[0]+self.ray.tlen*self.ray.dir[0]]
            };

            let origin_inside=ray_origin_x>=fat_line[0] && ray_origin_x<=fat_line[1];

            match (o1,o2){
                (Some(a),None)=>{ 
                    if origin_inside{
                        Some((a.min(ray_origin_y),a.max(ray_origin_y)))
                    }else{
                        Some((a.min(ray_end_y),a.max(ray_end_y)))
                    }
                },
                (None,Some(a))=>{
                    if origin_inside{
                        Some((a.min(ray_origin_y),a.max(ray_origin_y)))
                    }else{
                        Some((a.min(ray_end_y),a.max(ray_end_y)))
                    }
                },
                (Some(a),Some(b))=>{
                    Some((a.min(b),b.max(a)))
                },
                (None,None)=>{
                    //TODO figure out inequalities
                    if origin_inside{
                        Some((ray_origin_y.min(ray_end_y),ray_origin_y.max(ray_end_y)))
                    }else{
                        None
                    }
                }
            }
        }
  
        fn compute_distance_to_line<A:axgeom::AxisTrait>(&mut self,axis:A,line:Self::N)->Option<Self::N>{
            compute_intersection_point(axis,&self.ray,line).map(|a|a.0)
        }

        fn compute_distance_bot(&mut self,depth:Depth,a:&BBox<f64N,Bot>)->Option<Self::N>{
            //let ((x1,x2),(y1,y2))=a.get().get();
            intersects_box(self.ray.point,self.ray.dir,self.ray.tlen,a.get())
        }
        
    }
}


pub struct RaycastF64Demo{
    tree:DynTree<axgeom::XAXISS,(),BBox<f64N,Bot>>,
}
impl RaycastF64Demo{
    pub fn new(dim:[f64;2])->RaycastF64Demo{
        let bots=create_bots_f64(|id,pos|Bot{id,col:Vec::new()},&[0,dim[0] as isize,0,dim[1] as isize],500,[2,20]);
        let tree = DynTree::new(axgeom::XAXISS,(),bots.into_iter().map(|b|b.into_bbox()));
        RaycastF64Demo{tree}
    }
}

impl DemoSys for RaycastF64Demo{
    fn step(&mut self,cursor:[f64N;2],c:&piston_window::Context,g:&mut piston_window::G2d){
        let tree=&self.tree;
        //Draw bots
        for bot in tree.iter(){
            let ((x1,x2),(y1,y2))=bot.get().get();
            let ((x1,x2),(y1,y2))=((x1.into_inner(),x2.into_inner()),(y1.into_inner(),y2.into_inner()));
                
            let square = [x1,y1,x2-x1,y2-y1];
            rectangle([0.0,0.0,0.0,0.3], square, c.transform, g);
        }
    
        {

            let bb=axgeom::Rect::new(f64n!(0.0),f64n!(800.0),f64n!(0.0),f64n!(800.0));
            
            for i in 0..360{
                let i=i as f64*(std::f64::consts::PI/180.0);
                let x=(i.cos()*20.0) as f64 ;
                let y=(i.sin()*20.0) as f64;

                let ray={
                    let point=cursor;

                    let dir=[NotNaN::new(x).unwrap(),NotNaN::new(y).unwrap()];
                    raycast::Ray{point,dir,tlen:NotNaN::new(300.0).unwrap(),}
                };

                
                let k=raycast::raycast(&tree,ray,ray_f64::RayT{ray,c:&c,g});

                let (ppx,ppy)=if let Some(k)=k{
                    let ppx=ray.point[0]+ray.dir[0]*k.1;
                    let ppy=ray.point[1]+ray.dir[1]*k.1;
                    (ppx,ppy)
                }else{
                    let ppx=ray.point[0]+ray.dir[0]*800.0;
                    let ppy=ray.point[1]+ray.dir[1]*800.0;
                    (ppx,ppy)
                };

                let arr=[ray.point[0].into_inner() ,ray.point[1].into_inner() ,ppx.into_inner() ,ppy.into_inner() ];
                line([0.0, 0.0, 1.0, 0.2], // black
                     1.0, // radius of line
                     arr, // [x0, y0, x1,y1] coordinates of line
                     c.transform,
                     g);
            }
        }
    }
}
