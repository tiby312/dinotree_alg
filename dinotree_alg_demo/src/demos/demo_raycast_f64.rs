use support::prelude::*;
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
            use duckduckgeo::IntersectsBotResult;
            match self.ray.intersects_box(rect){
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
                self.ray.point[0].cmp(&div)
            }else{
                self.ray.point[1].cmp(&div)
            }
        }
        
        
        fn compute_distance_to_line<A:axgeom::AxisTrait>(&mut self,axis:A,line:Self::N)->Option<Self::N>{
            //let ray=duckduckgeo::Ray{point:self.ray.point,dir:self.ray.dir};
            self.ray.compute_intersection_tvalue(axis,line)
        }

        fn compute_distance_bot(&mut self,a:&BBox<F64n,()>)->Option<Self::N>{
            use duckduckgeo::IntersectsBotResult;
            match self.ray.intersects_box(a.get()){
                IntersectsBotResult::Hit(val)=>{
                    Some(val)
                },
                IntersectsBotResult::NoHit=>{
                    None
                },
                IntersectsBotResult::Inside=>{
                    Some(f64n!(0.0))
                    //None
                }
            }
        }
        
    }
}


pub struct RaycastF64Demo{
    tree:DinoTree<axgeom::XAXISS,(),BBox<F64n,()>>,
    dim:[f64;2]
}
impl RaycastF64Demo{

    pub fn new(dim:[f64;2])->RaycastF64Demo{
        let dim2=&[0,dim[0] as isize,0,dim[1] as isize];
        let radius=[5,20];
        let velocity=[1,3];
        let mut bot_iter=create_world_generator(500,dim2,radius,velocity);

        let bots=vec![();500];

        let tree = DinoTree::new(axgeom::XAXISS,(),&bots,|_|{
            let ret=bot_iter.next().unwrap();
            let p=ret.pos;
            let r=ret.radius;
            Conv::from_rect(aabb_from_pointf64(p,r))
        });

        RaycastF64Demo{tree,dim}
    }
}

impl DemoSys for RaycastF64Demo{
    fn step(&mut self,cursor:[f64;2],c:&piston_window::Context,g:&mut piston_window::G2d,_check_naive:bool){
        let tree=&self.tree;
        //Draw bots
        for bot in tree.iter(){
            draw_rect_f64n([0.0,0.0,0.0,0.3],bot.get(),c,g);
        }
    
        { 
            for i in 0..360{
                let i=i as f64*(std::f64::consts::PI/180.0);
                let x=(i.cos()*20.0) as f64 ;
                let y=(i.sin()*20.0) as f64;

                let ray={
                    let dir=[f64n!(x),f64n!(y)];
                    let point=[f64n!(cursor[0]),f64n!(cursor[1])];
                    duckduckgeo::Ray{point,dir}
                };

                
                let k=raycast::raycast(&tree,axgeom::Rect::new(f64n!(0.0),f64n!(self.dim[0]),f64n!(0.0),f64n!(self.dim[1])),ray_f64::RayT{ray,c:&c,g});
                
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
