use crate::support::prelude::*;
use dinotree_alg::raycast;
use dinotree_alg;
use std;
use duckduckgeo;
use dinotree_alg::raycast::RayIntersectResult;

use std::cell::RefCell;

//TODO problem with lines are straight up?
//isize implementation of a ray.
mod ray_f32{
    use super::*;
    
    use self::raycast::RayTrait;
    use duckduckgeo;



    pub struct RayT<'a,'c:'a>{
        pub c:&'a Context,
        pub g:RefCell<&'a mut G2d<'c>>,
        pub height:usize,
        pub draw:bool,
    }

    impl<'a,'c:'a> RayTrait for RayT<'a,'c>{
        type T=BBox<F32n,Bot2>;
        type N=F32n;


        fn compute_distance_to_bot(&self,ray:&raycast::Ray<Self::N>,bot:&Self::T)->RayIntersectResult<Self::N>{
            if self.draw{
                draw_rect_f32([1.0,0.0,0.0,0.5],bot.get().as_ref(),self.c,&mut self.g.borrow_mut());
            }
            Self::compute_distance_to_rect(self,ray,bot.get()) 
        }

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

#[derive(Copy,Clone,Debug)]
pub struct Bot2{
    id:usize
}

pub struct RaycastF32DebugDemo{
    tree:DinoTree<axgeom::XAXISS,BBox<F32n,Bot2>>,
    counter:f32,
    dim:Rect<F32n>
}
impl RaycastF32DebugDemo{
    pub fn new(dim:Rect<F32n>)->RaycastF32DebugDemo{


        let bots:Vec<_>=(0..3000).map(|id|Bot2{id}).collect();



        let mut ii=UniformRandGen::new(dim.inner_into()).with_radius(1.0,4.0)
            .take(3000);


        let tree = DinoTreeBuilder::new(axgeom::XAXISS,&bots,|_a|{
            let (pos,radius)=ii.next().unwrap();
            Rect::from_point(pos,radius).inner_try_into().unwrap()
        }).build_par();

        RaycastF32DebugDemo{tree,counter:0.0,dim}
    }
}

impl DemoSys for RaycastF32DebugDemo{
    fn step(&mut self,cursor:Vec2<F32n>,c:&piston_window::Context,g:&mut piston_window::G2d,check_naive:bool){
        let tree=&self.tree;
        let counter=&mut self.counter;

        let ray:raycast::Ray<F32n>={
            *counter+=0.004;         
            let point:Vec2<f32>=cursor.inner_into::<f32>().inner_as();
            //*counter=10.0;
            let dir=vec2(counter.cos()*10.0,counter.sin()*10.0);

            let dir=dir.inner_as();
            raycast::Ray{point,dir}.inner_try_into().unwrap()
        };

        for bot in tree.get_bots().iter(){
            draw_rect_f32([0.0,0.0,0.0,0.3],bot.get().as_ref(),c,g);
        }   

        let k={
            let height=tree.height();
            

            //dbg!("START");
            let test = match raycast::raycast(&tree,self.dim,ray,ray_f32::RayT{draw:true,c:&c,g:RefCell::new(g),height}){
                Some((mut bots,dis))=>{
                    let mut k:Vec<_>=bots.iter().map(|a|a.inner.id).collect();
                    k.sort();
                    Some((k,dis.into_inner()))
                },
                None=>{
                    None
                }
            };

            //dbg!("END");

            if check_naive{

                let tree_ref=&tree;
                
                
                let k = match raycast::naive(tree_ref.get_bots().iter(),ray,ray_f32::RayT{draw:false,c:&c,g:RefCell::new(g),height}){
                    Some((mut bots,dis))=>{
                        let mut k:Vec<_>=bots.iter().map(|a|a.inner.id).collect();
                        k.sort();
                        Some((k,dis.into_inner()))
                    },
                    None=>{
                        None
                    }
                };

                match (&test,&k){
                    (Some(a),Some(b))=>{
                        assert_eq!(a.1,b.1);
                        for (a,b) in a.0.iter().zip(b.0.iter()){
                            assert_eq!(a,b);
                        }
                    },
                    (None,None)=>{
                        //Do nothing
                    },
                    _=>{
                        panic!("fail");
                    }
                }
            }

            let ray:raycast::Ray<f32>=ray.inner_into();
            
            let (ppx,ppy)=match test{
                Some(k)=>{
                    let ppx=ray.point.x+ray.dir.x*k.1;
                    let ppy=ray.point.y+ray.dir.y*k.1;
                    (ppx,ppy)
                },
                None=>{
                    let ppx=ray.point.x+ray.dir.x*800.0;
                    let ppy=ray.point.y+ray.dir.y*800.0;
                    (ppx,ppy)
                }
            };

            let arr=[ray.point.x as f64,ray.point.y as f64,ppx as f64,ppy as f64];
            line([0.0, 0.0, 0.0, 1.0], // black
                 2.0, // radius of line
                 arr, // [x0, y0, x1,y1] coordinates of line
                 c.transform,
                 g);

        };
        
        
        {
            struct Bla<'a,'b:'a>{
                c:&'a Context,
                g:&'a mut G2d<'b>
            }
            impl<'a,'b:'a> dinotree_alg::graphics::DividerDrawer for Bla<'a,'b>{
                type N=F32n;
                fn draw_divider<A:axgeom::AxisTrait>(&mut self,axis:A,div:F32n,cont:[F32n;2],length:[F32n;2],depth:usize){
                    let div=div.into_inner();
                    let length=[length[0].into_inner(),length[1].into_inner()];
                    let cont=[cont[0].into_inner(),cont[1].into_inner()];
                    

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
            dinotree_alg::graphics::draw(&tree,&mut dd,&self.dim);
        }

            

    }
}