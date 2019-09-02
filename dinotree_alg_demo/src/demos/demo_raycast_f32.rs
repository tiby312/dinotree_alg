use crate::support::prelude::*;
use dinotree_alg::raycast;
use std;
use duckduckgeo;
use dinotree_alg::raycast::RayIntersectResult;
use core::marker::PhantomData;
mod ray_f32{
    use super::*;

    use self::raycast::RayTrait;
    use duckduckgeo;

    pub struct RayT<'a,'c:'a>{
        pub c:&'a Context,
        pub g:&'a mut G2d<'c>
    }

    impl<'a,'c:'a> RayTrait for RayT<'a,'c>{
        //type T=BBoxPtr<F32n,()>;
        type N=F32n;
        type Inner=();


        fn compute_distance_to_rect(&self,ray:&raycast::Ray<Self::N>,rect:&Rect<Self::N>)->RayIntersectResult<Self::N>{
            let ray:duckduckgeo::Ray<f32>=Ray{point:ray.point.inner_into(),dir:ray.dir.inner_into()};
            let rect:&Rect<f32>=rect.as_ref();

            
            let k=ray_intersects_box(&ray,&rect);
            match k{
                IntersectsBotResult::Hit(val)=>{
                    RayIntersectResult::Hit(NotNan::new(val).unwrap())
                },
                IntersectsBotResult::NoHit=>{
                    RayIntersectResult::NoHit
                },
                IntersectsBotResult::Inside=>{
                    RayIntersectResult::Hit(NotNan::new(0.0).unwrap())
                    
                    //Return none if you do not want results that intersect the ray origin.
                    //None
                }
            }
        }
        
    }
}


pub struct RaycastF32Demo{
    tree:DinoTreeOwned<axgeom::XAXISS,F32n,()>,
    dim:Rect<F32n>
}
impl RaycastF32Demo{

    pub fn new(dim:Rect<F32n>)->Self{
        
        let mut vv:Vec<_> = (0..500).map(|_|()).collect();
        
        let mut ii=UniformRandGen::new(dim.inner_into()).with_radius(5.0,10.0).map(|(pos,radius)|{
            Rect::from_point(pos,radius).inner_try_into().unwrap()
        });


        let tree = DinoTreeOwnedBuilder::new(axgeom::XAXISS,vv,|a|{
            ii.next().unwrap()
        }).build_seq();


        Self{tree,dim}
    }
}

impl DemoSys for RaycastF32Demo{
    fn step(&mut self,cursor:Vec2<F32n>,c:&piston_window::Context,g:&mut piston_window::G2d,_check_naive:bool){
        
        //Draw bots
        for bot in self.tree.as_ref().get_bots().iter(){
            draw_rect_f32([0.0,0.0,0.0,0.3],bot.rect.as_ref(),c,g);
        }
    
        let mut tree=self.tree.as_mut();
        
        { 
            for dir in 0..360i32{
                let dir=dir as f32*(std::f32::consts::PI/180.0);
                let x=(dir.cos()*20.0) as f32 ;
                let y=(dir.sin()*20.0) as f32;

                let ray={
                    let k=vec2(x,y).inner_try_into().unwrap();
                    raycast::Ray{point:cursor,dir:k}
                };

                

                let res=raycast::raycast_mut(&mut tree,self.dim,ray,ray_f32::RayT{c:&c,g});
                
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
