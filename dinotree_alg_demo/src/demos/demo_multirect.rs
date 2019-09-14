use crate::support::prelude::*;
use dinotree_alg::rect::*;

#[derive(Copy,Clone,Debug)]
struct Bot{
    radius:Vec2<i32>,
    pos:Vec2<i32>
}

pub struct MultiRectDemo{
    tree:DinoTreeOwned<axgeom::XAXISS,i32,Bot>
}
impl MultiRectDemo{
    pub fn new(dim:Rect<F32n>)->MultiRectDemo{



        let mut bots:Vec<_>=UniformRandGen::new(dim.inner_into()).with_radius(5.0,20.0).
            take(200).map(|(pos,radius)|{
            let pos=pos.inner_as();
            let radius=radius.inner_as();
            Bot{pos,radius}
        }).collect();


        let tree = create_owned(axgeom::XAXISS,bots,|b|{ Rect::from_point(b.pos,b.radius)},|axis,bots|DinoTreeBuilder::new(axis,bots).build_seq());


        MultiRectDemo{tree}
    }
}

impl DemoSys for MultiRectDemo{
    fn step(&mut self,cursor:Vec2<F32n>,c:&piston_window::Context,g:&mut piston_window::G2d,_check_naive:bool){
        
        
        
        for bot in self.tree.get_aabb_bots().iter(){
            draw_rect_i32([0.0,0.0,0.0,0.3],&bot.rect,c,g);
        }


        let cc:Vec2<i32>=cursor.inner_into::<f32>().inner_as();
        let r1=axgeom::Rect::new(cc.x-100,cc.x+100,cc.y-100,cc.y+100);
        let r2=axgeom::Rect::new(100,400,100,400);

        
        {
            let mut rects=MultiRectMut::new(self.tree.get_mut());


            let mut to_draw=Vec::new();
            let _=rects.for_all_in_rect_mut(r1, |a| {
                to_draw.push(a)
            });


            let res= rects.for_all_in_rect_mut(r2, |a| {
                to_draw.push(a);
            });

            
            
            match res{
                Ok(())=>{
                    draw_rect_i32([0.0,0.0,0.0,0.3],&r1,c,g);
                    draw_rect_i32([0.0,0.0,0.0,0.3],&r2,c,g);
            
                    for r in to_draw.iter(){
                        draw_rect_i32([1.0,0.0,0.0,0.3],r.get(),c,g);
                    }
                },
                Err(_)=>{
                    draw_rect_i32([1.0,0.0,0.0,0.3],&r1,c,g);
                    draw_rect_i32([1.0,0.0,0.0,0.3],&r2,c,g);
                }
            }
        }
        

        
        for_all_intersect_rect(self.tree.get(),&r1,|a|{
            draw_rect_i32([0.0,0.0,1.0,0.3],a.get(),c,g);
        });
        
        /* TODO do something else here
        let mut rects=multirect::multi_rect_mut(&mut self.tree);
        let _ = multirect::collide_two_rect_parallel(&mut rects,axgeom::YAXISS,&r1,&r2,|a,b|{
            
            let arr=[a.inner.pos.x as f64,a.inner.pos.y as f64,b.inner.pos.x as f64,b.inner.pos.y as f64];
            line([0.0, 0.0, 0.0, 0.2], // black
                 1.0, // radius of line
                 arr, // [x0, y0, x1,y1] coordinates of line
                 c.transform,
                 g);
        });
        */


        
   }
}
