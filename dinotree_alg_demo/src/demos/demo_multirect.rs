use crate::support::prelude::*;
use dinotree_alg::multirect;
use dinotree_alg::rect;

#[derive(Copy,Clone)]
struct Bot{
    radius:[isize;2],
    pos:[isize;2]
}

pub struct MultiRectDemo{
    tree:DinoTree<axgeom::XAXISS,(),BBox<isize,Bot>>
}
impl MultiRectDemo{
    pub fn new(dim:[f64;2])->MultiRectDemo{
        let dim=&[0,dim[0] as isize,0,dim[1] as isize];
        let radius=[5,20];
        let velocity=[1,3];
        
        let bots:Vec<Bot>=create_world_generator(500,dim,radius,velocity).map(|ret|{
            let ret=ret.into_isize();
            Bot{radius:ret.radius,pos:ret.pos}
        }).collect();

        let tree = DinoTreeBuilder::new(axgeom::XAXISS,(),&bots,|b|{aabb_from_point_isize(b.pos,b.radius)}).build_par();

        MultiRectDemo{tree}
    }
}

impl DemoSys for MultiRectDemo{
    fn step(&mut self,cursor:[f64;2],c:&piston_window::Context,g:&mut piston_window::G2d,_check_naive:bool){
        
        let tree=&mut self.tree;

        for bot in tree.as_ref().iter(){
            draw_rect_isize([0.0,0.0,0.0,0.3],bot.get(),c,g);
        }

        let cx=cursor[0] as isize;
        let cy=cursor[1] as isize;
        let r1=axgeom::Rect::new(cx-100,cx+100,cy-100,cy+100);
        let r2=axgeom::Rect::new(100,400,100,400);

        {
            let mut rects=multirect::multi_rect_mut(tree.as_ref_mut());


            let mut to_draw=Vec::new();
            let _=rects.for_all_in_rect_mut(r1, |a| {
                to_draw.push(a)
            });


            let res= rects.for_all_in_rect_mut(r2, |a| {
                to_draw.push(a);
            });

            
            
            match res{
                Ok(())=>{
                    draw_rect_isize([0.0,0.0,0.0,0.3],&r1,c,g);
                    draw_rect_isize([0.0,0.0,0.0,0.3],&r2,c,g);
            
                    for r in to_draw.iter(){
                        draw_rect_isize([1.0,0.0,0.0,0.3],r.get(),c,g);
                    }
                },
                Err(_)=>{
                    draw_rect_isize([1.0,0.0,0.0,0.3],&r1,c,g);
                    draw_rect_isize([1.0,0.0,0.0,0.3],&r2,c,g);
                }
            }
        }

        rect::for_all_intersect_rect(tree.as_ref(),&r1,|a|{
            draw_rect_isize([0.0,0.0,1.0,0.3],a.get(),c,g);
            
        });

        let mut rects=multirect::multi_rect_mut(tree.as_ref_mut());
        let _ = multirect::collide_two_rect_parallel(&mut rects,axgeom::YAXISS,&r1,&r2,|a,b|{
            
            let arr=[a.inner.pos[0] as f64,a.inner.pos[1] as f64,b.inner.pos[0] as f64,b.inner.pos[1] as f64];
            line([0.0, 0.0, 0.0, 0.2], // black
                 1.0, // radius of line
                 arr, // [x0, y0, x1,y1] coordinates of line
                 c.transform,
                 g);
        });

        
   }
}
