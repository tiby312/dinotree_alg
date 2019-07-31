use crate::support::prelude::*;
use dinotree_alg::raycast;
use dinotree_alg;
use std;
use duckduckgeo;


//TODO problem with lines are straight up?
//isize implementation of a ray.
mod ray_isize{
    use super::*;
    
    use self::raycast::RayTrait;
    use duckduckgeo;



    pub struct RayT<'a,'c:'a>{
        pub ray:duckduckgeo::Ray<isize>,
        pub c:&'a Context,
        pub g:&'a mut G2d<'c>,
        pub height:usize
    }

    impl<'a,'c:'a> RayTrait for RayT<'a,'c>{
        type T=BBox<isize,Bot2>;
        type N=isize;

 

        fn intersects_rect(&self,rect:&axgeom::Rect<Self::N>)->bool{
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
            self.ray.compute_intersection_tvalue(axis,line)
        }


        fn compute_distance_bot(&mut self,a:&Self::T)->Option<Self::N>{
            draw_rect_isize([1.0,0.0,0.0,0.8],a.get(),self.c,self.g);
            match self.ray.intersects_box(a.get()){
                IntersectsBotResult::Hit(val)=>{
                    Some(val)
                },
                IntersectsBotResult::NoHit=>{
                    None
                },
                IntersectsBotResult::Inside=>{
                    Some(0)
                    //Return none if you do not want results that intersect the ray origin.
                    //None
                }
            }
        }
        
    }

    #[derive(Copy,Clone,Debug)]
    pub struct RayNoDraw{
        pub ray:duckduckgeo::Ray<isize>
    }

    impl RayTrait for RayNoDraw{
        type T=BBox<isize,Bot2>;
        type N=isize;

        fn intersects_rect(&self,_rect:&axgeom::Rect<Self::N>)->bool{
            unimplemented!();
        }
        fn divider_side(&self,_axis:impl axgeom::AxisTrait,_div:&Self::N)->std::cmp::Ordering{
            unimplemented!();
        }
  
        fn compute_distance_to_line<A:axgeom::AxisTrait>(&mut self,axis:A,line:Self::N)->Option<Self::N>{
            //let ray=duckduckgeo::Ray{point:self.ray.point,dir:self.ray.dir};
            self.ray.compute_intersection_tvalue(axis,line)
        }

        fn compute_distance_bot(&mut self,a:&Self::T)->Option<Self::N>{
            match self.ray.intersects_box(a.get()){
                IntersectsBotResult::Hit(val)=>{
                    Some(val)
                },
                IntersectsBotResult::NoHit=>{
                    None
                },
                IntersectsBotResult::Inside=>{
                    Some(0)
                    //None
                }
            }
        }
        
    }
}

#[derive(Copy,Clone,Debug)]
pub struct Bot2{
    id:usize
}

pub struct RaycastDemo{
    tree:DinoTree<axgeom::XAXISS,BBox<isize,Bot2>>,
    counter:f64,
    dim:[isize;2]
}
impl RaycastDemo{
    pub fn new(dim2:[f64;2])->RaycastDemo{
        let dim=&[0,dim2[0] as isize,0,dim2[1] as isize];
        let radius=[2,6];
        let velocity=[1,3];
        let mut bots_fake=create_world_generator(4000,dim,radius,velocity);

        let bots:Vec<Bot2>=(0..4000).map(|id|Bot2{id}).collect();

        let tree = DinoTreeBuilder::new(axgeom::XAXISS,&bots,|_a|{
            let ret=bots_fake.next().unwrap();
            let ret=ret.into_isize();
            let p=ret.pos;
            let r=ret.radius;
            axgeom::Rect::new(p[0]-r[0],p[0]+r[0],p[1]-r[1],p[1]+r[1])
        }).build_par();
        RaycastDemo{tree,counter:0.0,dim:[dim2[0] as isize,dim2[1] as isize]}
    }
}

impl DemoSys for RaycastDemo{
    fn step(&mut self,cursor:[f64;2],c:&piston_window::Context,g:&mut piston_window::G2d,check_naive:bool){
        let tree=&self.tree;
        let counter=&mut self.counter;

        let ray={
            let point=[cursor[0] as isize,cursor[1] as isize];
            //let point=[573,161];
            //let point=[214,388];
            //println!("cursor={:?}",point);
            *counter+=0.005;         
            let dir=[counter.cos()*10.0,counter.sin()*10.0];
            //let dir=[1,0];
            let dir=[dir[0] as isize,dir[1] as isize];
            duckduckgeo::Ray{point,dir}
        };

        for bot in tree.get_bots().iter(){
            draw_rect_isize([0.0,0.0,0.0,0.3],bot.get(),c,g);
        }   

        let k={
            let height=tree.height();
            let res1=raycast::raycast(&tree,axgeom::Rect::new(0,self.dim[0],0,self.dim[1]),ray_isize::RayT{ray,c:&c,g,height});
            match res1{
                Some((mut bots,dis))=>{
                    bots.sort_by(|a,b|a.inner.id.cmp(&b.inner.id));
 
                    if check_naive{
                        let tree_ref=&tree;
                        let (mut bots2,dis2)  = raycast::naive(tree_ref.get_bots().iter(),ray_isize::RayNoDraw{ray}).unwrap();
                        assert_eq!(dis,dis2);
                        bots2.sort_by(|a,b|a.inner.id.cmp(&b.inner.id));

                        if bots.len()!=bots2.len(){
                            println!("lengths dont match raycast:{:?} naive:{:?}",bots.len(),bots2.len());
                        }else{
                            for (&v,&p) in bots.iter().zip(bots2.iter()){
                                
                                {
                                    let a=v as *const BBox<isize,Bot2>;
                                    let b=p as *const BBox<isize,Bot2>;
                                    //println!("{:?}",(a,b));
                                    if  a!=b {
                                        println!("{:?}\n{:?}\n{:?}",v,p,ray.point);
                                        panic!();
                                    }
                                }   
                             
                            }
                        }
                        
                        
                    }
                    
                    bots.into_iter().next().map(|a|(a,dis))

                },
                None=>{
                    None
                }
            } 
        };
        
        
        {
            struct Bla<'a,'b:'a>{
                c:&'a Context,
                g:&'a mut G2d<'b>
            }
            impl<'a,'b:'a> dinotree_alg::graphics::DividerDrawer for Bla<'a,'b>{
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
            dinotree_alg::graphics::draw(&tree,&mut dd,&axgeom::Rect::new(0,self.dim[0],0,self.dim[1]));
        }


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