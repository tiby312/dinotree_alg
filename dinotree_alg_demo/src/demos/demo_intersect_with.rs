use support::prelude::*;
use dinotree::colfind;
use dinotree::rect;
use dinotree::intersect_with;
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
        a.force[0]-=diff[0]*0.01;
        a.force[1]-=diff[1]*0.01;
    }
    fn repel(&mut self,other:&mut Bot){
        let a=self;
        let b=other;
        let mut diff=[b.pos[0]-a.pos[0],b.pos[1]-a.pos[1]];

        let mut len_sqr=diff[0]*diff[0]+diff[1]*diff[1];

        if len_sqr<0.0001{
            diff=[1.0,1.0];
            len_sqr=2.0;
        }
        let len=len_sqr.sqrt();
        let mag=2.0/len;

        let norm=[diff[0]/len,diff[1]/len];

        a.force[0]-=norm[0]*mag;
        a.force[1]-=norm[1]*mag;
        b.force[0]+=norm[0]*mag;
        b.force[1]+=norm[1]*mag;
    }
}
pub struct IntersectWithDemo{
    radius:f64,
    bots:Vec<Bot>,
    walls:Vec<BBox<f64N,()>>,
    dim:[f64;2]
}
impl IntersectWithDemo{
    pub fn new(dim:[f64;2])->IntersectWithDemo{
        let dim2=&[0,dim[0] as isize,0,dim[1] as isize];
        let radius=[3,5];
        let velocity=[1,3];
        let bots=create_world_generator(2000,dim2,radius,velocity).map(|ret|{
            Bot{pos:ret.pos,vel:ret.vel,force:[0.0;2]}
        }).collect();

        let radius=[10,60];
        let walls=create_world_generator(40,dim2,radius,velocity).map(|ret|{
            let rect=aabb_from_pointf64(ret.pos,ret.radius);
            BBox::new(rectf64_to_notnan(rect),())//{pos:ret.pos,vel:ret.vel,force:[0.0;2]}
        }).collect();

        IntersectWithDemo{radius:10.0,bots,walls,dim}
    }
}

impl DemoSys for IntersectWithDemo{
    fn step(&mut self,cursor:[f64;2],c:&piston_window::Context,g:&mut piston_window::G2d){
        let radius=5.0;
        let bots=&mut self.bots;
        let walls=&mut self.walls;

        for b in bots.iter_mut(){
            b.update();
            dinotree_geom::wrap_position(&mut b.pos,self.dim);
        }
        bots[0].pos=cursor;


        let mut tree=DynTree::new(axgeom::XAXISS,(),bots.drain(..).map(|b|{
            let p=b.pos;
            let rect=aabb_from_pointf64(p,[radius;2]);
            BBox::new(rectf64_to_notnan(rect),b)
        }));

        use axgeom::*;
        

        intersect_with::intersect_with_mut(&mut tree,walls,|bot,wall|{
            let fric=0.8;


            let wallx=wall.get().as_axis().get(axgeom::XAXISS);
            let wally=wall.get().as_axis().get(axgeom::YAXISS);
            let vel=bot.inner.vel;

            let ret=match dinotree_geom::collide_with_rect(bot.get(),wall.get()){
                dinotree_geom::WallSide::Above=>{
                    [None,Some((wally.left-radius,-vel[1]*fric))]
                },
                dinotree_geom::WallSide::Below=>{
                    [None,Some((wally.right+radius,-vel[1]*fric))]
                },
                dinotree_geom::WallSide::LeftOf=>{
                    [Some((wallx.left-radius,-vel[0]*fric)),None]
                },
                dinotree_geom::WallSide::RightOf=>{
                    [Some((wallx.right+radius,-vel[0]*fric)),None]
                }
            };

            match ret[0]{
                Some((pos,vel))=>{
                    bot.inner.pos[0]=pos.into_inner();
                    bot.inner.vel[0]=vel;
                },
                None=>{}
            }
            match ret[1]{
                Some((pos,vel))=>{
                    bot.inner.pos[1]=pos.into_inner();
                    bot.inner.vel[1]=vel;
                },
                None=>{}
            }
        });

        
        for b in tree.into_iter_orig_order(){
            bots.push(b.inner);
        }

        //Update the aabbs to match the new positions.
        let mut tree=DynTree::new(axgeom::XAXISS,(),bots.drain(..).map(|b|{
            let p=b.pos;
            let rect=aabb_from_pointf64(p,[radius;2]);
            BBox::new(rectf64_to_notnan(rect),b)
        }));
    


        
        rect::for_all_in_rect_mut(&mut tree,&rectf64_to_notnan(aabb_from_pointf64(cursor,[100.0;2])),|b|{
            b.inner.repel_mouse(cursor);
        });
        

        for wall in walls.iter(){
            let ((x1,x2),(y1,y2))=wall.get().get();
            //let ((x1,x2),(y1,y2))=((x1 as f64,x2 as f64),(y1 as f64,y2 as f64));
            let ((x1,x2),(y1,y2))=((x1.into_inner(),x2.into_inner()),(y1.into_inner(),y2.into_inner()));
              
            let square = [x1,y1,x2-x1,y2-y1];
            rectangle([0.0,0.0,1.0,0.3], square, c.transform, g);
        }
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

