use crate::support::prelude::*;
use dinotree_alg::multirect;
use dinotree_alg::rect;

#[derive(Copy,Clone,Debug)]
struct Bot{
    radius:Vector2<isize>,
    pos:Vector2<isize>
}

pub struct MultiRectDemo{
    tree:DinoTree<axgeom::XAXISS,BBox<isize,Bot>>
}
impl MultiRectDemo{
    pub fn new(dim:Vector2<F64n>)->MultiRectDemo{


        let dim2:Vector2<f64>=vec2_inner_into(dim);        
        let border=axgeom::Rect::new(0.0,dim2.x,0.0,dim2.y);


        let rand_radius=dists::RandomRectBuilder::new(vec2(5.0,5.0),vec2(20.0,20.0));
        let bots:Vec<_>=dists::uniform_rand::UniformRangeBuilder::new(border).build().
            take(500).zip(rand_radius).enumerate().map(|(id,(pos,radius))|{
            let pos=pos.cast().unwrap();
            let radius=radius.cast().unwrap();
            Bot{pos,radius}
        }).collect();


        let tree = DinoTreeBuilder::new(axgeom::XAXISS,&bots,|b|{ rect_from_point(b.pos,b.radius)}).build_par();

        MultiRectDemo{tree}
    }
}

impl DemoSys for MultiRectDemo{
    fn step(&mut self,cursor:Vector2<F64n>,c:&piston_window::Context,g:&mut piston_window::G2d,_check_naive:bool){
        
        let tree=&mut self.tree;

        for bot in tree.get_bots().iter(){
            //println!("bot={:?}",bot);
            draw_rect_isize([0.0,0.0,0.0,0.3],bot.get(),c,g);
        }

        let cc:Vector2<isize>=cursor.cast().unwrap();
        let r1=axgeom::Rect::new(cc.x-100,cc.x+100,cc.y-100,cc.y+100);
        let r2=axgeom::Rect::new(100,400,100,400);

        
        {
            let mut rects=multirect::multi_rect_mut(tree);


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
        

        
        rect::for_all_intersect_rect(&tree,&r1,|a|{
            draw_rect_isize([0.0,0.0,1.0,0.3],a.get(),c,g);
        });
        
        let mut rects=multirect::multi_rect_mut(tree);
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
