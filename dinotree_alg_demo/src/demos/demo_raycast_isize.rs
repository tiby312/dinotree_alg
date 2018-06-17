use support::prelude::*;
use dinotree::raycast;
//isize implementation of a ray.
mod ray_isize{
    use super::*;
    use self::raycast::Ray;
    use self::raycast::RayTrait;

    fn intersects_box(point:[isize;2],dir:[isize;2],matt:isize,rect:&axgeom::Rect<isize>)->Option<isize>{
        let ((x1,x2),(y1,y2))=rect.get();


        let mut tmin=isize::min_value();
        let mut tlen=isize::max_value();

        if dir[0]!=0{
            let tx1=(x1-point[0])/dir[0];
            let tx2=(x2-point[0])/dir[0];

            tmin=tmin.max(tx1.min(tx2));
            tlen=tlen.min(tx1.max(tx2));
            
        }else{
            if point[0] < x1 || point[0] > x2 {
                return None; // parallel AND outside box : no intersection possible
            }
        }
        if dir[1]!=0{
            let ty1=(y1-point[1])/dir[1];
            let ty2=(y2-point[1])/dir[1];

            tmin=tmin.max(ty1.min(ty2));
            tlen=tlen.min(ty1.max(ty2));
        }else{
            if point[1] < y1 || point[1] > y2 {
                return None; // parallel AND outside box : no intersection possible
            }
        }
        if tlen>=tmin && tlen>=0 && tmin<=matt{
            return Some(tmin.max(0));
        }else{
            return None;
        }
                    
    }

    pub struct RayT<'a,'c:'a>{
        pub ray:Ray<isize>,
        pub c:&'a Context,
        pub g:&'a mut G2d<'c>,
        pub height:usize
    }

    fn compute_intersection_point<A:axgeom::AxisTrait>(axis:A,ray:&Ray<isize>,line:isize)->Option<(isize,isize)>{
        if axis.is_xaxis(){
            if ray.dir[0]==0{
                if ray.point[0]==line{
                    Some((0,ray.point[1]))
                }else{
                    None
                }
            }else{
                let t=(line-ray.point[0])/ray.dir[0];
                
                if t>=0 && t<=ray.tlen{
                    Some((t,ray.point[1]+ray.dir[1]*t))
                }else{
                    None
                }
            }
        }else{
            if ray.dir[1]==0{
                if ray.point[1]==line{
                    Some((0,ray.point[0]))
                }else{
                    None
                }
            }else{
                let t=(line-ray.point[1])/ray.dir[1];
                if t>=0 && t<=ray.tlen{
                    Some((t,ray.point[0]+ray.dir[0]*t))
                }else{
                    None
                }
            }
            
        }
    }

    impl<'a,'c:'a> RayTrait for RayT<'a,'c>{
        type T=BBox<isize,()>;
        type N=isize;

        fn split_ray<A:axgeom::AxisTrait>(&mut self,axis:A,ray:&Ray<Self::N>,fo:Self::N)->Option<(Ray<Self::N>,Ray<Self::N>)>{
            let t=if axis.is_xaxis(){
                if ray.dir[0]==0{
                    if ray.point[0]==fo{
                        let t1=ray.tlen/2;
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
                if ray.dir[1]==0{
                    if ray.point[1]==fo{
                        let t1=ray.tlen/2;
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

            if t>ray.tlen || t<0{
                return None
            }

            let new_point=[ray.point[0]+ray.dir[0]*t,ray.point[1]+ray.dir[1]*t];
            
            let ray_closer=Ray{point:ray.point,dir:ray.dir,tlen:t};
            let ray_new=Ray{point:new_point,dir:ray.dir,tlen:ray.tlen-t};
            Some((ray_closer,ray_new))
        }

        //First option is min, second is max
        fn compute_intersection_range<A:axgeom::AxisTrait>(&mut self,axis:A,fat_line:[Self::N;2])->Option<(Self::N,Self::N)>
        {
            let o1:Option<(Self::N,Self::N)>=compute_intersection_point::<A>(axis,&self.ray,fat_line[0]);
            let o2:Option<(Self::N,Self::N)>=compute_intersection_point::<A>(axis,&self.ray,fat_line[1]);

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

        fn compute_distance_bot(&mut self,depth:Depth,a:&Self::T)->Option<Self::N>{
            let ((x1,x2),(y1,y2))=a.get().get();
            
            {
                let ((x1,x2),(y1,y2))=((x1 as f64,x2 as f64),(y1 as f64,y2 as f64));
                let square = [x1,y1,x2-x1,y2-y1];
                let rr=depth.0 as f32/self.height as f32;
                //println!("depth={:?}",depth.0);
                rectangle([rr,0.0,0.0,0.8], square, self.c.transform, self.g);
            }
            //ray.point
            intersects_box(self.ray.point,self.ray.dir,self.ray.tlen,a.get())
        }
        
    }
}


pub struct RaycastDemo{
    tree:DynTree<axgeom::XAXISS,(),BBox<isize,()>>,
    counter:f64
}
impl RaycastDemo{
    pub fn new(dim:[f64;2])->RaycastDemo{
        let dim2=[f64n!(dim[0]),f64n!(dim[1])];
        let dim=&[0,dim[0] as isize,0,dim[1] as isize];
        let radius=[5,20];
        let velocity=[1,3];
        let bots=create_world_generator(500,dim,radius,velocity).map(|ret|{
            let ret=ret.into_isize();
            let p=ret.pos;
            let r=ret.radius;
            BBox::new(axgeom::Rect::new(p[0]-r[0],p[0]+r[0],p[1]-r[1],p[1]+r[1]),())
        });

        //let bots=create_bots_isize(|id|Bot{id,col:Vec::new()},&[0,dim[0] as isize,0,dim[1] as isize],500,[2,20]);
        let tree = DynTree::new(axgeom::XAXISS,(),bots);
        RaycastDemo{tree,counter:0.0}
    }
}

impl DemoSys for RaycastDemo{
    fn step(&mut self,cursor:[NotNaN<f64>;2],c:&piston_window::Context,g:&mut piston_window::G2d){
        let tree=&self.tree;
        let counter=&mut self.counter;

        let ray={
            let point=[cursor[0].into_inner() as isize,cursor[1].into_inner() as isize];
            //let point=[214,388];
            //println!("cursor={:?}",point);
            *counter+=0.01;         
            let dir=[counter.cos()*10.0,counter.sin()*10.0];
            //let dir=[1,1];
            let dir=[dir[0] as isize,dir[1] as isize];
            raycast::Ray{point,dir,tlen:500}
        };

        for bot in tree.iter(){
            let ((x1,x2),(y1,y2))=bot.get().get();
            let ((x1,x2),(y1,y2))=((x1 as f64,x2 as f64),(y1 as f64,y2 as f64));
                
            let square = [x1,y1,x2-x1,y2-y1];
            rectangle([0.0,0.0,0.0,0.3], square, c.transform, g);
        }   

        let k={
            let height=tree.get_height();
            raycast::raycast(&tree,ray,ray_isize::RayT{ray,c:&c,g,height})
        };

        let (ppx,ppy)=if let Some(k)=k{
            let ppx=ray.point[0]+ray.dir[0]*k.1;
            let ppy=ray.point[1]+ray.dir[1]*k.1;
            (ppx,ppy)
        }else{
            let ppx=ray.point[0]+ray.dir[0]*800;
            let ppy=ray.point[1]+ray.dir[1]*800;
            (ppx,ppy)
        };

        let arr=[ray.point[0] as f64,ray.point[1] as f64,ppx as f64,ppy as f64];
        line([0.0, 0.0, 0.0, 1.0], // black
             2.0, // radius of line
             arr, // [x0, y0, x1,y1] coordinates of line
             c.transform,
             g);
            

    }
}