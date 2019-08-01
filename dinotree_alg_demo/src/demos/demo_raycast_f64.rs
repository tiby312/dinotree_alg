use crate::support::prelude::*;
use dinotree_alg::raycast;
use std;
use duckduckgeo;
mod ray_f64{
    use super::*;

    use self::raycast::RayTrait;
    use duckduckgeo;

    pub struct RayT<'a,'c:'a>{
        pub ray:duckduckgeo::Ray<F64n>,
        pub c:&'a Context,
        pub g:&'a mut G2d<'c>
    }

    impl<'a,'c:'a> RayTrait for RayT<'a,'c>{
        type T=BBox<F64n,()>;
        type N=F64n;


        fn intersects_rect(&self,rect:&axgeom::Rect<Self::N>)->bool{
            let ray:Ray<f64>=self.ray.cast().unwrap();
            //TODO investigate if there is a reference based cast????
            let rect=rect.cast().unwrap();

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
            let ray:Ray<f64>=self.ray.cast().unwrap();
            let line:f64=*line;
            //let ray=duckduckgeo::Ray{point:self.ray.point,dir:self.ray.dir};
            ray_compute_intersection_tvalue(&ray,axis,line).map(|a|NotNan::new(a).unwrap())
        }

        fn compute_distance_bot(&mut self,rect:&BBox<F64n,()>)->Option<Self::N>{
            let ray:Ray<f64>=self.ray.cast().unwrap();
            //TODO investigate if there is a reference based cast????
            let rect=rect.get().cast().unwrap();

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


pub struct RaycastF64Demo{
    tree:DinoTree<axgeom::XAXISS,BBox<F64n,()>>,
    dim:Vector2<F64n>
}
impl RaycastF64Demo{

    pub fn new(dim:Vector2<F64n>)->RaycastF64Demo{
        let dim2:Vector2<f64>=dim.cast().unwrap();
        let border=axgeom::Rect::new(0.0,dim2.x,0.0,dim2.y);
        
        let rand_radius=dists::RandomRectBuilder::new(vec2(5.0,5.0),vec2(20.0,20.0));
        
        let mut ii=dists::uniform_rand::UniformRangeBuilder::new(border).build().take(500).zip(rand_radius);


        let bots:Vec<()>=(0..500).map(|a|()).collect();



        let tree = DinoTreeBuilder::new(axgeom::XAXISS,&bots,|bot|{
            let (pos,radius)=ii.next().unwrap();
            rect_from_point(pos,radius).cast().unwrap()
        }).build_par();

        RaycastF64Demo{tree,dim}
    }
}

impl DemoSys for RaycastF64Demo{
    fn step(&mut self,cursor:Vector2<F64n>,c:&piston_window::Context,g:&mut piston_window::G2d,_check_naive:bool){
        let tree=&self.tree;
        //Draw bots
        for bot in tree.get_bots().iter(){
            draw_rect_f64n([0.0,0.0,0.0,0.3],bot.get(),c,g);
        }
    
        { 
            for dir in 0..360{
                let dir=f64::from(dir)*(std::f64::consts::PI/180.0);
                let x=(dir.cos()*20.0) as f64 ;
                let y=(dir.sin()*20.0) as f64;

                let ray={
                    let k=vec2(x,y).cast().unwrap();
                    duckduckgeo::Ray::new(cursor,k)
                };

                
                let res=raycast::raycast(&tree,axgeom::Rect::new(NotNan::<_>::zero(),self.dim[0],NotNan::<_>::zero(),self.dim[1]),ray_f64::RayT{ray,c:&c,g});
                
                let (ppx,ppy)=if let Some(k)=res{
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
