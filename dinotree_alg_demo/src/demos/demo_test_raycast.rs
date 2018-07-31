use support::prelude::*;
use dinotree::raycast;
use dinotree;
use std;
use dinotree_geom;
#[derive(Debug,Copy,Clone)]
struct Ray<N>{
    pub point:[N;2],
    pub dir:[N;2],
    pub tlen:N,
}

pub struct TestRaycastDemo{
    counter:f64
}
impl TestRaycastDemo{
    pub fn new(dim2:[f64;2])->TestRaycastDemo{
        
        TestRaycastDemo{counter:0.0}
    }
}

impl DemoSys for TestRaycastDemo{
    fn step(&mut self,cursor:[f64;2],c:&piston_window::Context,g:&mut piston_window::G2d){
        let counter=&mut self.counter;
        let ray={
            let point=[cursor[0] as isize,cursor[1] as isize];
            //let point=[214,388];
            //println!("cursor={:?}",point);
            *counter+=0.005;         
            let dir=[counter.cos()*10.0,counter.sin()*10.0];
            //let dir=[1,1];
            let dir=[dir[0] as isize,dir[1] as isize];
            dinotree_geom::Ray{point,dir,tlen:50}
        };

        let rect=axgeom::Rect::new(100,140,200,300);

        draw_rect_isize([0.0,0.0,0.0,0.3],&rect,c,g);
           

        let res= dinotree_geom::intersects_box(ray.point,ray.dir,ray.tlen,&rect);
        println!("{:?}",res);
    
        let k=match res{
            dinotree_geom::IntersectsBotResult::Hit(val)=>{
                Some(val)
            },
            _=>{
                None
            }
        };

        let (ppx,ppy)=if let Some(k)=k{
            let ppx=ray.point[0]+ray.dir[0]*k;
            let ppy=ray.point[1]+ray.dir[1]*k;
            (ppx,ppy)
        }else{
            let ppx=ray.point[0]+ray.dir[0]*ray.tlen;
            let ppy=ray.point[1]+ray.dir[1]*ray.tlen;
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