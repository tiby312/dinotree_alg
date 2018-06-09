extern crate piston_window;
extern crate axgeom;
extern crate num;
extern crate rand;
extern crate dinotree;
extern crate dinotree_inner;
extern crate compt;
extern crate ordered_float;

use piston_window::*;

use compt::*;
mod support;
use dinotree::*;
use dinotree::support::*;
use support::*;
use dinotree_inner::*;
use raycast::*;
use axgeom::AxisTrait;

mod ray{
    use super::*;
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

    fn compute_intersection_point<A:AxisTrait>(ray:&Ray<isize>,line:isize)->Option<(isize,isize)>{
        if A::new().is_xaxis(){
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
        type T=BBox<isize,Bot>;
        type N=isize;

        fn split_ray<A:AxisTrait>(&mut self,ray:&Ray<Self::N>,fo:Self::N)->Option<(Ray<Self::N>,Ray<Self::N>)>{
            let t=if A::new().is_xaxis(){
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
        fn compute_intersection_range<A:AxisTrait>(&mut self,fat_line:[Self::N;2])->Option<(Self::N,Self::N)>
        {
            let o1:Option<(Self::N,Self::N)>=compute_intersection_point::<A>(&self.ray,fat_line[0]);
            let o2:Option<(Self::N,Self::N)>=compute_intersection_point::<A>(&self.ray,fat_line[1]);

            let o1=o1.map(|a|a.1);
            let o2=o2.map(|a|a.1);


            

            let [ray_origin_x,ray_origin_y,ray_end_y]=if A::new().is_xaxis(){
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
  
        fn compute_distance_to_line<A:AxisTrait>(&mut self,line:Self::N)->Option<Self::N>{
            compute_intersection_point::<A>(&self.ray,line).map(|a|a.0)
        }

        fn compute_distance_bot(&mut self,depth:Depth,a:&Self::T)->Option<Self::N>{
            let ((x1,x2),(y1,y2))=a.rect.get();
            
            {
                let ((x1,x2),(y1,y2))=((x1 as f64,x2 as f64),(y1 as f64,y2 as f64));
                let square = [x1,y1,x2-x1,y2-y1];
                let rr=depth.0 as f32/self.height as f32;
                //println!("depth={:?}",depth.0);
                rectangle([rr,0.0,0.0,0.8], square, self.c.transform, self.g);
            }
            //ray.point
            intersects_box(self.ray.point,self.ray.dir,self.ray.tlen,&a.rect)
        }
        
    }
}
fn main() {

    let mut bots=create_bots_isize(|id|Bot{id,col:Vec::new()},&[0,800,0,800],500,[2,20]);
    
    let mut tree = DynTree::new(axgeom::XAXISS,(),bots.into_iter());


    let mut window: PistonWindow = WindowSettings::new("dinotree test", [800, 800])
        .exit_on_esc(true)
        .build()
        .unwrap();

    let mut cursor=[0.0,0.0];

    let mut counter=0.0f32;
    while let Some(e) = window.next() {
        e.mouse_cursor(|x, y| {
            cursor = [x, y];
        });

        window.draw_2d(&e, |c, g| {
            clear([1.0; 4], g);


            let ray={
                let point=[cursor[0] as isize,cursor[1] as isize];
                //let point=[214,388];
                //println!("cursor={:?}",point);
                counter+=0.01;         
                let dir=[counter.cos()*10.0,counter.sin()*10.0];
                //let dir=[1,1];
                let dir=[dir[0] as isize,dir[1] as isize];
                Ray{point,dir,tlen:500}
            };

            for bot in tree.iter(){
                let ((x1,x2),(y1,y2))=bot.rect.get();
                let ((x1,x2),(y1,y2))=((x1 as f64,x2 as f64),(y1 as f64,y2 as f64));
                    
                let square = [x1,y1,x2-x1,y2-y1];
                rectangle([0.0,0.0,0.0,0.3], square, c.transform, g);
            }
        
        
            {
                

                let k={
                    let height=tree.get_height();
                    raycast(&tree,ray,ray::RayT{ray,c:&c,g,height})
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

        });
    }

}
