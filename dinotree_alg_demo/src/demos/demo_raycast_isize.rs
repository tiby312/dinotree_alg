use support::prelude::*;
use dinotree::raycast;
//isize implementation of a ray.
mod ray_isize{
    use super::*;
    use self::raycast::Ray;
    use self::raycast::RayTrait;
    use dinotree_geom;

    pub struct RayT<'a,'c:'a>{
        pub ray:Ray<isize>,
        pub c:&'a Context,
        pub g:&'a mut G2d<'c>,
        pub height:usize
    }

    impl<'a,'c:'a> RayTrait for RayT<'a,'c>{
        type T=BBox<isize,()>;
        type N=isize;

        fn split_ray<A:axgeom::AxisTrait>(&mut self,axis:A,ray:&Ray<Self::N>,fo:Self::N)->Option<(Ray<Self::N>,Ray<Self::N>)>{
            let ray=dinotree_geom::Ray{point:self.ray.point,dir:self.ray.dir,tlen:self.ray.tlen};
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

        fn compute_distance_bot(&mut self,depth:Depth,a:&Self::T)->Option<Self::N>{
            let ((x1,x2),(y1,y2))=a.get().get();
            
            {
                let ((x1,x2),(y1,y2))=((x1 as f64,x2 as f64),(y1 as f64,y2 as f64));
                let square = [x1,y1,x2-x1,y2-y1];
                let rr=depth.0 as f32/self.height as f32;
                rectangle([rr,0.0,0.0,0.8], square, self.c.transform, self.g);
            }

            dinotree_geom::intersects_box(self.ray.point,self.ray.dir,self.ray.tlen,a.get())
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
    fn step(&mut self,cursor:[f64;2],c:&piston_window::Context,g:&mut piston_window::G2d){
        let tree=&self.tree;
        let counter=&mut self.counter;

        let ray={
            let point=[cursor[0] as isize,cursor[1] as isize];
            //let point=[214,388];
            //println!("cursor={:?}",point);
            *counter+=0.005;         
            let dir=[counter.cos()*10.0,counter.sin()*10.0];
            //let dir=[0,-1];
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