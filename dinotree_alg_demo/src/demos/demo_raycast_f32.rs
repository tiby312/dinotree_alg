use crate::support::prelude::*;
use dinotree_alg::raycast;
use std;
use duckduckgeo;
mod ray_f32{
    use super::*;

    use self::raycast::RayTrait;
    use duckduckgeo;

    pub struct RayT<'a,'c:'a>{
        pub ray:duckduckgeo::Ray<F32n>,
        pub c:&'a Context,
        pub g:&'a mut G2d<'c>
    }

    impl<'a,'c:'a> RayTrait for RayT<'a,'c>{
        type T=BBox<F32n,()>;
        type N=F32n;


        fn intersects_rect(&self,rect:&axgeom::Rect<Self::N>)->bool{
            let ray:Ray<f32>=self.ray.inner_into();
            let rect=rect.inner_into();

            match ray_intersects_box(&ray,&rect){
                IntersectsBotResult::Hit(_)=>{
                    true
                },
                IntersectsBotResult::NoHit=>{
                    false
                },
                IntersectsBotResult::Inside=>{
                    true
                }
            }
        }
        fn divider_side(&self,axis:impl axgeom::AxisTrait,div:&Self::N)->std::cmp::Ordering{
            if axis.is_xaxis(){
                self.ray.point.x.cmp(&div)
            }else{
                self.ray.point.y.cmp(&div)
            }
        }
        
        
        fn compute_distance_to_line<A:axgeom::AxisTrait>(&mut self,axis:A,line:Self::N)->Option<Self::N>{
            let ray:Ray<f32>=self.ray.inner_into();
            let line:f32=*line;
            //let ray=duckduckgeo::Ray{point:self.ray.point,dir:self.ray.dir};
            ray_compute_intersection_tvalue(&ray,axis,line).map(|a|NotNan::new(a).unwrap())
        }

        fn compute_distance_bot(&mut self,rect:&BBox<F32n,()>)->Option<Self::N>{
            let ray:Ray<f32>=self.ray.inner_into();
            let rect=rect.get().inner_into();

            match ray_intersects_box(&ray,&rect){
                IntersectsBotResult::Hit(val)=>{
                    Some(val)
                },
                IntersectsBotResult::NoHit=>{
                    None
                },
                IntersectsBotResult::Inside=>{
                    Some(0.0)
                    //None
                }
            }.map(|a|NotNan::new(a).unwrap())
        }
        
    }
}


pub struct RaycastF32Demo{
    tree:DinoTree<axgeom::XAXISS,BBox<F32n,()>>,
    dim:Rect<F32n>
}
impl RaycastF32Demo{

    pub fn new(dim:Rect<F32n>)->Self{
        

        let bots:Vec<()>=(0..500).map(|_|()).collect();

        let mut ii=UniformRandGen::new(dim.inner_into()).with_radius(5.0,20.0).take(500);


        let tree = DinoTreeBuilder::new(axgeom::XAXISS,&bots,|_|{
            let (pos,radius)=ii.next().unwrap();
            Rect::from_point(pos,radius).inner_try_into().unwrap()
        }).build_par();

        Self{tree,dim}
    }
}

impl DemoSys for RaycastF32Demo{
    fn step(&mut self,cursor:Vec2<F32n>,c:&piston_window::Context,g:&mut piston_window::G2d,_check_naive:bool){
        let tree=&self.tree;
        //Draw bots
        for bot in tree.get_bots().iter(){
            draw_rect_f32([0.0,0.0,0.0,0.3],bot.get().as_ref(),c,g);
        }
    
        { 
            for dir in 0..360i32{
                let dir=dir as f32*(std::f32::consts::PI/180.0);
                let x=(dir.cos()*20.0) as f32 ;
                let y=(dir.sin()*20.0) as f32;

                let x=0.0;
                let y=1.0;
                let ray={
                    let k=vec2(x,y).inner_try_into().unwrap();
                    duckduckgeo::Ray::new(cursor,k)
                };

                

                let res=raycast::raycast(&tree,self.dim,ray_f32::RayT{ray,c:&c,g});
                
                let (ppx,ppy)=if let Some(k)=res{
                    let ppx=ray.point.x+ray.dir.x*k.1;
                    let ppy=ray.point.y+ray.dir.y*k.1;
                    (ppx,ppy)
                }else{
                    let ppx=ray.point.x+ray.dir.x*800.0;
                    let ppy=ray.point.y+ray.dir.y*800.0;
                    (ppx,ppy)
                };

                let arr=[ray.point.x.into_inner() as f64 ,ray.point.y.into_inner() as f64,ppx.into_inner() as f64,ppy.into_inner() as f64];
                line([0.0, 0.0, 1.0, 0.2], // black
                     1.0, // radius of line
                     arr, // [x0, y0, x1,y1] coordinates of line
                     c.transform,
                     g);
                
            }
        }
    }
}
