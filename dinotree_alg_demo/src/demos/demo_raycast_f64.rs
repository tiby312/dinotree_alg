use support::prelude::*;
use dinotree::raycast;
use std;
mod ray_f64{
    use super::*;

    use self::raycast::Ray;
    use self::raycast::RayTrait;
    use dinotree_geom;

    pub struct RayT<'a,'c:'a>{
        pub ray:Ray<F64n>,
        pub c:&'a Context,
        pub g:&'a mut G2d<'c>
    }

    impl<'a,'c:'a> RayTrait for RayT<'a,'c>{
        type T=BBox<F64n,()>;
        type N=F64n;

        fn split_ray<A:axgeom::AxisTrait>(&mut self,axis:A,ray:&Ray<Self::N>,fo:Self::N)->Option<(Ray<Self::N>,Ray<Self::N>)>{
            let ray=dinotree_geom::Ray{point:ray.point,dir:ray.dir,tlen:ray.tlen};
            dinotree_geom::split_ray(axis,&ray,fo).map(|(a,b)|{
                let r1=Ray{point:a.point,dir:a.dir,tlen:a.tlen};
                let r2=Ray{point:b.point,dir:b.dir,tlen:b.tlen};
                (r1,r2)
            })   
        }
        
        //First option is min, second is max
        fn compute_intersection_range<A:axgeom::AxisTrait>(&mut self,axis:A,fat_line:[Self::N;2])->Option<(Self::N,Self::N)>
        {
            let ray=dinotree_geom::Ray{point:self.ray.point,dir:self.ray.dir,tlen:self.ray.tlen};
            dinotree_geom::compute_intersection_range(&ray,axis,fat_line)
        }
        
        fn compute_distance_to_line<A:axgeom::AxisTrait>(&mut self,axis:A,line:Self::N)->Option<Self::N>{
            let ray=dinotree_geom::Ray{point:self.ray.point,dir:self.ray.dir,tlen:self.ray.tlen};
            dinotree_geom::compute_intersection_tvalue(axis,&ray,line)
        }

        fn compute_distance_bot(&mut self,a:&BBox<F64n,()>)->Option<Self::N>{
            dinotree_geom::intersects_box(self.ray.point,self.ray.dir,self.ray.tlen,a.get())
        }
        
    }
}


pub struct RaycastF64Demo{
    tree:DynTree<axgeom::XAXISS,(),BBox<F64n,()>>,
}
impl RaycastF64Demo{

    pub fn new(dim:[f64;2])->RaycastF64Demo{
        let dim=&[0,dim[0] as isize,0,dim[1] as isize];
        let radius=[5,20];
        let velocity=[1,3];
        let mut bot_iter=create_world_generator(500,dim,radius,velocity);

        let bots=vec![();500];

        let tree = DynTree::new(axgeom::XAXISS,(),&bots,|b|{
            let ret=bot_iter.next().unwrap();
            let p=ret.pos;
            let r=ret.radius;
            Conv::from_rect(aabb_from_pointf64(p,r))
        });

        RaycastF64Demo{tree}
    }
}

impl DemoSys for RaycastF64Demo{
    fn step(&mut self,cursor:[f64;2],c:&piston_window::Context,g:&mut piston_window::G2d){
        let tree=&self.tree;
        //Draw bots
        for bot in tree.iter_every_bot(){
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
                    raycast::Ray{point,dir,tlen:f64n!(300.0)}
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
