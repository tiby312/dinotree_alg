use support::prelude::*;

pub struct RectDemo{
    bots:Vec<BBoxVisible<isize,Bot>>,
    tree:DynTree<axgeom::XAXISS,(),BBox<isize,Bot>>
}
impl RectDemo{
    pub fn new(dim:[f64;2])->RectDemo{
        let bots=create_bots_isize(|id|Bot{id,col:Vec::new()},&[0,dim[0] as isize,0,dim[1] as isize],500,[2,20]);
        let tree = DynTree::new(axgeom::XAXISS,(),bots.clone().into_iter().map(|b|b.into_bbox()));
        RectDemo{bots,tree}
    }
}

impl DemoSys for RectDemo{
    fn step(&mut self,cursor:[f64N;2],c:&piston_window::Context,g:&mut piston_window::G2d){
        
        let bots=&self.bots;
        let tree=&self.tree;

        for bot in bots.iter(){
            let ((x1,x2),(y1,y2))=bot.rect.get();
            let ((x1,x2),(y1,y2))=((x1 as f64,x2 as f64),(y1 as f64,y2 as f64));

            let square = [x1,y1,x2-x1,y2-y1];
    
            rectangle([0.0,0.0,0.0,0.3], square, c.transform, g);
        }

        let cx=cursor[0].into_inner() as isize;
        let cy=cursor[1].into_inner() as isize;
        let r1=axgeom::Rect::new(cx-50,cx+50,cy-50,cy+50);
        {
            let ((x1,x2),(y1,y2))=r1.get();
            
            {
                let ((x1,x2),(y1,y2))=((x1 as f64,x2 as f64),(y1 as f64,y2 as f64));
                let square = [x1,y1,x2-x1,y2-y1];
                rectangle([0.0,0.0,1.0,0.2], square, c.transform, g);
            }
        }  
        

        let mut rects=dinotree::multirect(tree);


        let mut to_draw=Vec::new();
        let _=rects.for_all_in_rect(&r1, |a| {
            to_draw.push(a)
        });


        let r2=axgeom::Rect::new(20,200,20,200);
        let res= rects.for_all_in_rect(&r2, |a| {
            to_draw.push(a);
        });

        
        match res{
            Ok(())=>{
                for r in to_draw.iter(){
                    let ((x1,x2),(y1,y2))=r.rect.get();
                    
                    {
                        let ((x1,x2),(y1,y2))=((x1 as f64,x2 as f64),(y1 as f64,y2 as f64));
                        let square = [x1,y1,x2-x1,y2-y1];
                        rectangle([1.0,0.0,0.0,1.0], square, c.transform, g);
                    }
                }

            },
            Err(st)=>{
                println!("{:?}",st);
            }
        }

        
   }
}
