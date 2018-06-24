use support::prelude::*;
use dinotree::multirect;

pub struct MultiRectDemo{
    tree:DynTree<axgeom::XAXISS,(),BBox<isize,()>>
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
            BBox::new(axgeom::Rect::new(p[0]-r[0],p[0]+r[0],p[1]-r[1],p[1]+r[1]),())
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
            let ((x1,x2),(y1,y2))=bot.get().get();
            let ((x1,x2),(y1,y2))=((x1 as f64,x2 as f64),(y1 as f64,y2 as f64));

            let square = [x1,y1,x2-x1,y2-y1];
    
            rectangle([0.0,0.0,0.0,0.3], square, c.transform, g);
        }

        let cx=cursor[0] as isize;
        let cy=cursor[1] as isize;
        let r1=axgeom::Rect::new(cx-50,cx+50,cy-50,cy+50);
        {
            let ((x1,x2),(y1,y2))=r1.get();
            
            {
                let ((x1,x2),(y1,y2))=((x1 as f64,x2 as f64),(y1 as f64,y2 as f64));
                let square = [x1,y1,x2-x1,y2-y1];
                rectangle([0.0,0.0,1.0,0.2], square, c.transform, g);
            }
        }  
        

        let mut rects=multirect::multi_rect_mut(tree);


        let mut to_draw=Vec::new();
        let _=rects.for_all_in_rect_mut(r1, |a| {
            to_draw.push(a)
        });


        let r2=axgeom::Rect::new(100,400,100,400);
        let res= rects.for_all_in_rect_mut(r2, |a| {
            to_draw.push(a);
        });

        
        match res{
            Ok(())=>{
                for r in to_draw.iter(){
                    let ((x1,x2),(y1,y2))=r.get().get();
                    
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
