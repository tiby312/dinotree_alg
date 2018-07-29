use support::prelude::*;
use dinotree::raycast;
use dinotree;
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
        type T=BBox<isize,Bot>;
        type N=isize;

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

        fn compute_distance_bot(&mut self,a:&Self::T)->Option<Self::N>{
            draw_rect_isize([0.5,0.0,0.0,0.3],a.get(),self.c,self.g);
            use dinotree_geom::IntersectsBotResult;
            match dinotree_geom::intersects_box(self.ray.point,self.ray.dir,self.ray.tlen,a.get()){
                IntersectsBotResult::Hit(val)=>{
                    Some(val)
                },
                IntersectsBotResult::NoHit=>{
                    None
                },
                IntersectsBotResult::Inside|IntersectsBotResult::TooFar=>{
                    /*
                    let ((a,b),(c,d))=a.get().get();
                    let point=[a,c];
                    let d=dinotree_geom::distance_squred_point(point,self.ray.point);
                    Some(self.ray.tlen*2+d)
                    */
                    None
                }
            }
        }
        
    }

    pub struct RayNoDraw{
        pub ray:Ray<isize>
    }

    impl RayTrait for RayNoDraw{
        type T=BBox<isize,Bot>;
        type N=isize;

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

        fn compute_distance_bot(&mut self,a:&Self::T)->Option<Self::N>{
            use dinotree_geom::IntersectsBotResult;
            match dinotree_geom::intersects_box(self.ray.point,self.ray.dir,self.ray.tlen,a.get()){
                IntersectsBotResult::Hit(val)=>{
                    Some(val)
                },
                IntersectsBotResult::NoHit=>{
                    None
                },
                IntersectsBotResult::Inside|IntersectsBotResult::TooFar=>{
                    /*
                    let ((a,b),(c,d))=a.get().get();
                    let point=[a,c];
                    let d=dinotree_geom::distance_squred_point(point,self.ray.point);
                    Some(self.ray.tlen*2+d)
                    */
                    None
                }
            }
        }
        
    }
}

#[derive(Copy,Clone,Debug)]
pub struct Bot{
    id:usize
}

pub struct RaycastDemo{
    tree:DynTree<axgeom::XAXISS,(),BBox<isize,Bot>>,
    counter:f64,
    dim:[isize;2]
}
impl RaycastDemo{
    pub fn new(dim2:[f64;2])->RaycastDemo{
        let dim=&[0,dim2[0] as isize,0,dim2[1] as isize];
        let radius=[5,20];
        let velocity=[1,3];
        let mut bots_fake=create_world_generator(500,dim,radius,velocity);

        let bots:Vec<Bot>=(0..500).map(|id|Bot{id}).collect();

        //let bots=create_bots_isize(|id|Bot{id,col:Vec::new()},&[0,dim[0] as isize,0,dim[1] as isize],500,[2,20]);
        let tree = DynTree::new(axgeom::XAXISS,(),&bots,|a|{
            let ret=bots_fake.next().unwrap();
            let ret=ret.into_isize();
            let p=ret.pos;
            let r=ret.radius;
            axgeom::Rect::new(p[0]-r[0],p[0]+r[0],p[1]-r[1],p[1]+r[1])
        });
        RaycastDemo{tree,counter:0.0,dim:[dim2[0] as isize,dim2[1] as isize]}
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
            //*counter+=0.005;         
            //let dir=[counter.cos()*10.0,counter.sin()*10.0];
            let dir=[1,1];
            let dir=[dir[0] as isize,dir[1] as isize];
            raycast::Ray{point,dir,tlen:500}
        };

        for bot in tree.iter_every_bot(){
            draw_rect_isize([0.0,0.0,0.0,0.3],bot.get(),c,g);
        }   

        let k={
            let height=tree.get_height();
            let mut res1:Vec<(&BBox<isize,Bot>,isize)>=raycast::raycast(&tree,ray,ray_isize::RayT{ray,c:&c,g,height}).collect();
            res1.sort_by(|a,b|a.0.inner.id.cmp(&b.0.inner.id));

            
            let check_naive=true;
            if check_naive{
                let mut res2:Vec<(&BBox<isize,Bot>,isize)> = raycast::naive(tree.iter_every_bot(),ray_isize::RayNoDraw{ray}).collect();
                res2.sort_by(|a,b|a.0.inner.id.cmp(&b.0.inner.id));

                assert_eq!(res1.len(),res2.len());
                for (v,p) in res1.iter().zip(res2.iter()){
                   
                    if v.0 as *const BBox<isize,Bot> != p.0 as *const BBox<isize,Bot>{
                        println!("fail");
                    }   
                 
                }
                
            }
            
            res1.into_iter().next()
        };
        
        {
            struct Bla<'a,'b:'a>{
                c:&'a Context,
                g:&'a mut G2d<'b>
            }
            impl<'a,'b:'a> dinotree::graphics::DividerDrawer for Bla<'a,'b>{
                type N=isize;
                fn draw_divider<A:axgeom::AxisTrait>(&mut self,axis:A,div:isize,cont:[isize;2],length:[isize;2],depth:usize){
                    
                    

                    let arr=if axis.is_xaxis(){
                        [div as f64,length[0] as f64,div as f64,length[1] as f64]
                    }else{
                        [length[0] as f64,div as f64,length[1] as f64,div as f64]
                    };


                    let radius=(1isize.max(5-depth as isize)) as f64;

                    line([0.0, 0.0, 0.0, 0.5], // black
                         radius, // radius of line
                         arr, // [x0, y0, x1,y1] coordinates of line
                         self.c.transform,
                         self.g);

                    let [x1,y1,w1,w2]=if axis.is_xaxis(){
                        [cont[0],length[0],cont[1]-cont[0],length[1]-length[0]]
                    }else{
                        [length[0],cont[0],length[1]-length[0],cont[1]-cont[0]]
                    };

                    let [x1,y1,w1,w2]=[x1 as f64,y1 as f64,w1 as f64,w2 as f64];

                    let square = [x1,y1,w1,w2];
                    rectangle([0.0,1.0,1.0,0.2], square, self.c.transform, self.g);
                
                    
                    
                }
            }

            let mut dd=Bla{c:&c,g};
            dinotree::graphics::draw(&tree,&mut dd,&axgeom::Rect::new(0,self.dim[0],0,self.dim[1]));
        }


        let (ppx,ppy)=if let Some(k)=k{
            println!("k={:?}",k.1);
            let ppx=ray.point[0]+ray.dir[0]*k.1;
            let ppy=ray.point[1]+ray.dir[1]*k.1;
            (ppx,ppy)
        }else{
            let ppx=ray.point[0]+ray.dir[0]*800;
            let ppy=ray.point[1]+ray.dir[1]*800;
            (ppx,ppy)
        };
        println!("dir={:?}",ray.dir);

        let arr=[ray.point[0] as f64,ray.point[1] as f64,ppx as f64,ppy as f64];
        line([0.0, 0.0, 0.0, 1.0], // black
             2.0, // radius of line
             arr, // [x0, y0, x1,y1] coordinates of line
             c.transform,
             g);
            

    }
}