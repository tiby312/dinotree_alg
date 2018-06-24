use support::prelude::*;
use dinotree::colfind;
use dinotree::rect;
use dinotree_geom;

pub struct Bot{
    pos:[f64;2],
    vel:[f64;2],
    force:[f64;2],
}
impl Bot{
    fn update(&mut self){
        self.vel[0]+=self.force[0];
        self.vel[1]+=self.force[1];

        //non linear drag
        self.vel[0]*=0.9;
        self.vel[1]*=0.9;

        self.pos[0]+=self.vel[0];
        self.pos[1]+=self.vel[1];
        self.force[0]=0.0;
        self.force[1]=0.0;

    }

    fn repel_mouse(&mut self,mouse:[f64;2]){
        let a=self;
        let bpos=mouse;
        let diff=[bpos[0]-a.pos[0],bpos[1]-a.pos[1]];
        a.force[0]-=diff[0]*0.1;
        a.force[1]-=diff[1]*0.1;
    }
    fn repel(&mut self,other:&mut Bot){
        let a=self;
        let b=other;
        let diff=[b.pos[0]-a.pos[0],b.pos[1]-a.pos[1]];

        a.force[0]-=diff[0]*0.01;
        a.force[1]-=diff[1]*0.01;
        b.force[0]+=diff[0]*0.01;
        b.force[1]+=diff[1]*0.01;
    }
}
pub struct IntersectEveryDemo{
    radius:f64,
    bots:Vec<Bot>,
    dim:[f64;2]
}
impl IntersectEveryDemo{
    pub fn new(dim:[f64;2])->IntersectEveryDemo{
        let dim2=&[0,dim[0] as isize,0,dim[1] as isize];
        let radius=[5,10];
        let velocity=[1,3];
        let bots=create_world_generator(1000,dim2,radius,velocity).map(|ret|{
            Bot{pos:ret.pos,vel:ret.vel,force:[0.0;2]}
        }).collect();

        IntersectEveryDemo{radius:10.0,bots,dim}
    }
}

impl DemoSys for IntersectEveryDemo{
    fn step(&mut self,cursor:[f64;2],c:&piston_window::Context,g:&mut piston_window::G2d){
        let radius=10.0;
        let bots=&mut self.bots;

        for b in bots.iter_mut(){
            b.update();
            dinotree_geom::wrap_position(&mut b.pos,self.dim);
        }


        let mut tree=DynTree::new(axgeom::XAXISS,(),bots.drain(..).map(|b|{
            BBox::new(Conv::from_rect(aabb_from_pointf64(b.pos,[radius;2])),b)
        }));

        rect::for_all_in_rect_mut(&mut tree,&Conv::from_rect(aabb_from_pointf64(cursor,[100.0;2])),|b|{
            b.inner.repel_mouse(cursor);
        });
        
        for bot in tree.iter(){
            let ((x1,x2),(y1,y2))=bot.get().get();
            //let ((x1,x2),(y1,y2))=((x1 as f64,x2 as f64),(y1 as f64,y2 as f64));
            let ((x1,x2),(y1,y2))=((x1.into_inner(),x2.into_inner()),(y1.into_inner(),y2.into_inner()));
              
            let square = [x1,y1,x2-x1,y2-y1];
            rectangle([0.0,0.0,0.0,0.3], square, c.transform, g);
        }

        {
         
            colfind::query_mut(&mut tree,|a, b| {
                a.inner.repel(&mut b.inner);
                let ((x1,x2),(y1,y2))=a.get().get();
                
                {
                    let ((x1,x2),(y1,y2))=((x1.into_inner(),x2.into_inner()),(y1.into_inner(),y2.into_inner()));
                    let square = [x1,y1,x2-x1,y2-y1];
                    rectangle([1.0,0.0,0.0,0.2], square, c.transform, g);
                }

                let ((x1,x2),(y1,y2))=b.get().get();
                
                {
                    let ((x1,x2),(y1,y2))=((x1.into_inner(),x2.into_inner()),(y1.into_inner(),y2.into_inner()));
                    let square = [x1,y1,x2-x1,y2-y1];
                    rectangle([1.0,0.0,0.0,0.2], square, c.transform, g);
                }
            });
        
        }
        for b in tree.into_iter_orig_order(){
            bots.push(b.inner);
        }
     }
}

