use support::prelude::*;
use dinotree::raycast;
use dinotree;
use std;
use dinotree_geom;

//isize implementation of a ray.
mod ray_isize{
    use super::*;
    use self::raycast::RayTrait;
    use dinotree_geom;



    pub struct RayT<'a,'c:'a>{
        pub ray:dinotree_geom::Ray<isize>,
        pub c:&'a Context,
        pub g:&'a mut G2d<'c>,
        pub height:usize
    }

    impl<'a,'c:'a> RayTrait for RayT<'a,'c>{
        type T=BBox<isize,Bot>;
        type N=isize;

 

        fn intersects_rect(&self,rect:&axgeom::Rect<Self::N>)->bool{
            use dinotree_geom::IntersectsBotResult;
            match dinotree_geom::intersects_box(self.ray.point,self.ray.dir,self.ray.tlen,rect){
                IntersectsBotResult::Hit(val)=>{
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
            let ray=dinotree_geom::Ray{point:self.ray.point,dir:self.ray.dir,tlen:self.ray.tlen};
            dinotree_geom::compute_intersection_tvalue(axis,&ray,line)
        }


        fn compute_distance_bot(&mut self,a:&Self::T)->Option<Self::N>{
            draw_rect_isize([1.0,0.0,0.0,0.8],a.get(),self.c,self.g);
            use dinotree_geom::IntersectsBotResult;
            match dinotree_geom::intersects_box(self.ray.point,self.ray.dir,self.ray.tlen,a.get()){
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
        pub ray:dinotree_geom::Ray<isize>
    }

    impl RayTrait for RayNoDraw{
        type T=BBox<isize,Bot>;
        type N=isize;

        fn intersects_rect(&self,rect:&axgeom::Rect<Self::N>)->bool{
            unimplemented!();
        }
        fn divider_side(&self,axis:impl axgeom::AxisTrait,div:&Self::N)->std::cmp::Ordering{
            unimplemented!();
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
                IntersectsBotResult::Inside=>{
                    Some(0)
                    //None
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
            dinotree_geom::Ray{point,dir,tlen:500}
        };

        for bot in tree.iter_every_bot(){
            draw_rect_isize([0.0,0.0,0.0,0.3],bot.get(),c,g);
        }   

        let k={
            let height=tree.get_height();
            let mut res1=raycast::raycast(&tree,axgeom::Rect::new(0,self.dim[0],0,self.dim[1]),ray_isize::RayT{ray,c:&c,g,height});
            match res1{
                Some((mut bots,dis))=>{
                    bots.sort_by(|a,b|a.inner.id.cmp(&b.inner.id));

                    if check_naive{
                        let (mut bots2,dis2)  = raycast::naive(tree.iter_every_bot(),ray_isize::RayNoDraw{ray}).unwrap();
                        bots2.sort_by(|a,b|a.inner.id.cmp(&b.inner.id));

                        //println!("len={:?}",bots.len());
                        if bots.len()!=bots2.len(){
                            println!("lengths dont match raycast:{:?} naive:{:?}",bots.len(),bots2.len());
                        }else{
                            for (&v,&p) in bots.iter().zip(bots2.iter()){
                                
                                {
                                    let a=v as *const BBox<isize,Bot>;
                                    let b=p as *const BBox<isize,Bot>;
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