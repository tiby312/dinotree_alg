use support::prelude::*;
use dinotree::multirect;

struct Bot{
    pos:[isize;2]
}

pub struct MultiRectDemo{
    tree:DynTree<axgeom::XAXISS,(),BBox<isize,Bot>>
}
impl MultiRectDemo{
    pub fn new(dim:[f64;2])->MultiRectDemo{
        let dim=&[0,dim[0] as isize,0,dim[1] as isize];
        let radius=[5,20];
        let velocity=[1,3];
        let bots=create_world_generator(500,dim,radius,velocity).map(|ret|{
            let ret=ret.into_isize();
            let p=ret.pos;
            let r=ret.radius;
            BBox::new(axgeom::Rect::new(p[0]-r[0],p[0]+r[0],p[1]-r[1],p[1]+r[1]),Bot{pos:p})
        });

        let tree = DynTree::new(axgeom::XAXISS,(),bots);

        //let bots=create_bots_isize(|id|Bot{id,col:Vec::new()},&[0,dim[0] as isize,0,dim[1] as isize],500,[2,20]);
        //let tree = DynTree::new(axgeom::XAXISS,(),bots.into_iter().map(|b|b.into_bbox()));
        MultiRectDemo{tree}
    }
}

impl DemoSys for MultiRectDemo{
    fn step(&mut self,cursor:[f64;2],c:&piston_window::Context,g:&mut piston_window::G2d){
        
        let tree=&mut self.tree;

        for bot in tree.iter(){
            draw_rect_isize([0.0,0.0,0.0,0.3],bot.get(),c,g);
        }

        let cx=cursor[0] as isize;
        let cy=cursor[1] as isize;
        let r1=axgeom::Rect::new(cx-100,cx+100,cy-100,cy+100);
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

            
            draw_rect_isize([0.0,0.0,0.0,0.3],&r1,c,g);
            draw_rect_isize([0.0,0.0,0.0,0.3],&r2,c,g);
            
            match res{
                Ok(())=>{
                    for r in to_draw.iter(){
                        draw_rect_isize([1.0,0.0,0.0,0.3],r.get(),c,g);
                    }
                },
                Err(st)=>{
                    println!("{:?}",st);
                }

            }
        }

        let mut rects=multirect::multi_rect_mut(tree);
        let _ = multirect::collide_two_rect_parallel(&mut rects,axgeom::YAXISS,&r1,&r2,|a,b|{
            
            let arr=[a.inner.pos[0] as f64,a.inner.pos[1] as f64,b.inner.pos[0] as f64,b.inner.pos[1] as f64];
            line([0.0, 0.0, 0.0, 0.2], // black
                 1.0, // radius of line
                 arr, // [x0, y0, x1,y1] coordinates of line
                 c.transform,
                 g);

            //let mut r=*a.get();
            //draw_rect_isize([0.0,1.0,0.0,0.3],r.grow_to_fit(b.get()),c,g);
        });

        
   }
}
