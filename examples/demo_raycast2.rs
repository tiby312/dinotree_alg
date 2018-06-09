extern crate piston_window;
extern crate axgeom;
extern crate num;
extern crate rand;
extern crate dinotree;
//extern crate axgeom;
extern crate dinotree_inner;
extern crate compt;
extern crate ordered_float;
use piston_window::*;

mod support;
use compt::*;
use axgeom::AxisTrait;
use dinotree::*;
use dinotree::support::*;
use dinotree_inner::*;
use support::*;
use ordered_float::*;

use dinotree::raycast::*;

type FN64=NotNaN<f64>;

macro_rules! fn64 {
    ( $x:expr  ) => {
        {
            NotNaN::new($x).unwrap()
        }
    };
}


mod ray{
    use super::*;
    fn intersects_box(point:[FN64;2],dir:[FN64;2],matt:FN64,rect:&axgeom::Rect<FN64>)->Option<FN64>{
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
        pub ray:Ray<FN64>,
        pub c:&'a Context,
        pub g:&'a mut G2d<'c>
    }

    fn compute_intersection_point<A:AxisTrait>(ray:&Ray<FN64>,line:FN64)->Option<(FN64,FN64)>{
        if A::new().is_xaxis(){
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
        type T=BBox<FN64,Bot>;
        type N=FN64;

        fn split_ray<A:AxisTrait>(&mut self,ray:&Ray<Self::N>,fo:Self::N)->Option<(Ray<Self::N>,Ray<Self::N>)>{
            let t=if A::new().is_xaxis(){
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
            intersects_box(self.ray.point,self.ray.dir,self.ray.tlen,&a.rect)
        }
        
    }
}




fn main() {

    let mut bots=create_bots_f64(|id,_pos|Bot{id,col:Vec::new()},&[0,800,0,800],500,[2,20]);

    let mut tree = DynTree::new(axgeom::XAXISS,(),bots.into_iter());


    let mut window: PistonWindow = WindowSettings::new("dinotree test", [800, 800])
        .exit_on_esc(true)
        .samples(4)
        .build()
        .unwrap();

    let mut cursor=[0.0,0.0];
    while let Some(e) = window.next() {
        e.mouse_cursor(|x, y| {
            cursor = [x, y];
        });

        window.draw_2d(&e, |c, g| {
            clear([0.0; 4], g);

            
            //Draw bots
            for bot in tree.iter(){
                let ((x1,x2),(y1,y2))=bot.rect.get();
                let ((x1,x2),(y1,y2))=((x1.into_inner(),x2.into_inner()),(y1.into_inner(),y2.into_inner()));
                    
                let square = [x1,y1,x2-x1,y2-y1];
                rectangle([1.0,1.0,1.0,0.3], square, c.transform, g);
            }
        
            {

                let bb=axgeom::Rect::new(fn64!(0.0),fn64!(800.0),fn64!(0.0),fn64!(800.0));
                
                for i in 0..360{
                    let i=i as f64*(std::f64::consts::PI/180.0);
                    let x=(i.cos()*20.0) as f64 ;
                    let y=(i.sin()*20.0) as f64;

                    let ray={
                        let point=[NotNaN::new(cursor[0]).unwrap(),NotNaN::new(cursor[1]).unwrap()];
                        
                        let dir=[NotNaN::new(x).unwrap(),NotNaN::new(y).unwrap()];
                        Ray{point,dir,tlen:NotNaN::new(300.0).unwrap(),}
                    };

                    
                    let k=raycast(&tree,ray,ray::RayT{ray,c:&c,g});

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
                    line([1.0, 1.0, 1.0, 0.2], // black
                         1.0, // radius of line
                         arr, // [x0, y0, x1,y1] coordinates of line
                         c.transform,
                         g);
                }
            }
        });
    }
}
