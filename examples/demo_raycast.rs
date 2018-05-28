extern crate piston_window;
extern crate axgeom;
extern crate num;
extern crate rand;
extern crate dinotree;

extern crate ordered_float;
use piston_window::*;

mod support;
use dinotree::*;
use dinotree::support::*;
use support::*;

use axgeom::AxisTrait;

fn intersects_box(point:[isize;2],dir:[isize;2],matt:isize,rect:&AABBox<isize>)->Option<isize>{
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

fn main() {

    let mut bots=create_bots_isize(|id|Bot{id,col:Vec::new()},&[0,800,0,800],500,[2,20]);


    let mut window: PistonWindow = WindowSettings::new("dinotree test", [800, 800])
        .exit_on_esc(true)
        .build()
        .unwrap();

    let mut cursor=[0.0,0.0];
    while let Some(e) = window.next() {
        e.mouse_cursor(|x, y| {
            cursor = [x, y];
        });

        window.draw_2d(&e, |c, g| {
            clear([1.0; 4], g);

            let ray={
                let point=[cursor[0] as isize,cursor[1] as isize];
                let dir=[-1,-2];           
                Ray{point,dir,tlen:500,/*true_len:500*/}
            };

            for bot in bots.iter(){
                let ((x1,x2),(y1,y2))=bot.rect.get();
                let ((x1,x2),(y1,y2))=((x1 as f64,x2 as f64),(y1 as f64,y2 as f64));
                    
                let square = [x1,y1,x2-x1,y2-y1];
                rectangle([0.0,0.0,0.0,0.3], square, c.transform, g);
            }
        
        
            {
                let mut tree = DinoTree::new(&mut bots, StartAxis::Xaxis);


                let k={
                    let bb=AABBox::new((0+100,800-100),(0+100,800-100));
                    {
                        let ((x1,x2),(y1,y2))=bb.get();//(bb.xdiv,bb.ydiv);
                        let ((x1,x2),(y1,y2))=((x1 as f64,x2 as f64),(y1 as f64,y2 as f64));
                        let square = [x1,y1,x2-x1,y2-y1];
                        rectangle([0.0,1.0,0.0,0.2], square, c.transform, g);
                    }



                    struct RayT<'a,'c:'a>{
                        ray:Ray<isize>,
                        c:&'a Context,
                        g:&'a mut G2d<'c>,
                    }

                    fn compute_intersection_point<A:AxisTrait>(ray:&Ray<isize>,line:isize)->Option<(isize,isize)>{
                        if A::new().is_xaxis(){
                            let t=(line-ray.point[0])/ray.dir[0];
                            if t>0 && t<=ray.tlen{
                                Some((t,ray.point[1]+ray.dir[1]*t))
                            }else{
                                None
                            }
                        }else{
                            let t=(line-ray.point[1])/ray.dir[1];
                            if t>0 && t<=ray.tlen{
                                Some((t,ray.point[0]+ray.dir[0]*t))
                            }else{
                                None
                            }
                        }
                    }

                    impl<'a,'c:'a> RayTrait for RayT<'a,'c>{
                        type T=BBox<isize,Bot>;
                        type N=isize;


                        /*
                        fn add_ray(&mut self,ray:&Ray<Self::N>,t_to_add:Self::N)->Ray<Self::N>{
                            Ray{point:ray.point,dir:ray.dir,tlen:ray.tlen+t_to_add}
                        }
                        */

                        fn split_ray<A:AxisTrait>(&mut self,ray:&Ray<Self::N>,fo:Self::N)->Option<(Ray<Self::N>,Ray<Self::N>)>{
                            let t=if A::new().is_xaxis(){
                                (fo-ray.point[0])/ray.dir[0]
                                
                            }else{
                                (fo-ray.point[1])/ray.dir[1]
                            };

                            if t>ray.tlen || t<0{
                                return None
                            }
                            //assert!(t<=ray.tlen);

                            let new_point=[ray.point[0]+ray.dir[0]*t,ray.point[1]+ray.dir[1]*t];
                            

                            let ray_closer=Ray{point:ray.point,dir:ray.dir,tlen:t,/*true_len:ray.true_len-ray.tlen+t*/};
                            let ray_new=Ray{point:new_point,dir:ray.dir,tlen:ray.tlen-t,/*true_len:ray.true_len*/};
                            Some((ray_closer,ray_new))
                            
                        }

                        fn compute_intersection_range<A:AxisTrait>(&mut self,fat_line:[Self::N;2])->(Option<Self::N>,Option<Self::N>)
                        {
                            let o1:Option<(Self::N,Self::N)>=compute_intersection_point::<A>(&self.ray,fat_line[0]);
                            let o2:Option<(Self::N,Self::N)>=compute_intersection_point::<A>(&self.ray,fat_line[1]);

                            let o1=o1.map(|a|a.1);
                            let o2=o2.map(|a|a.1);

                            let (o1,o2)={
                                let point=if A::new().is_xaxis(){
                                    self.ray.point[1]
                                }else{
                                    self.ray.point[0]
                                };

                                match (o1,o2){
                                    (Some(a),None)=>{ 
                                        (Some(a),Some(point))
                                    },
                                    (None,Some(b))=>{
                                        (Some(point),Some(b))
                                    },
                                    (a,b)=>{
                                        (a,b)
                                    }
                                }
                            };

                            (o1,o2)
                        }
                        /*
                        fn zero(&mut self)->Self::N{
                            0   
                        }*/
                        fn compute_distance_to_line<A:AxisTrait>(&mut self,line:Self::N)->Option<Self::N>{
                            compute_intersection_point::<A>(&self.ray,line).map(|a|a.0)
                        }


                        fn compute_distance_bot(&mut self,a:ColSingle<Self::T>)->Option<Self::N>{
                            let ((x1,x2),(y1,y2))=a.rect.get();
                            
                            {
                                let ((x1,x2),(y1,y2))=((x1 as f64,x2 as f64),(y1 as f64,y2 as f64));
                                let square = [x1,y1,x2-x1,y2-y1];
                                rectangle([0.0,0.0,1.0,0.8], square, self.c.transform, self.g);
                            }
                            //ray.point
                            intersects_box(self.ray.point,self.ray.dir,self.ray.tlen,a.rect)
                        }
                        
                    }
                    
                    tree.raycast(ray,RayT{ray,c:&c,g})
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
