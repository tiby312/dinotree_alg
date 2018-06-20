use support::prelude::*;
use dinotree::colfind;
use dinotree::rect;
use dinotree::intersect_with;

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
        {
            let a=&mut self.pos;
            if a[0]>800.0{
                a[0]=0.0
            }
            if a[0]<0.0{
                a[0]=800.0;
            }
            if a[1]>800.0{
                a[1]=0.0
            }
            if a[1]<0.0{
                a[1]=800.0;
            }
        }
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
        let diff=[b.pos[0]-a.pos[0],b.pos[1]-a.pos[1]];

        a.force[0]-=diff[0]*0.01;
        a.force[1]-=diff[1]*0.01;
        b.force[0]+=diff[0]*0.01;
        b.force[1]+=diff[1]*0.01;
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
        let radius=[5,10];
        let velocity=[1,3];
        let bots=create_world_generator(1000,dim2,radius,velocity).map(|ret|{
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
        let radius=10.0;
        let bots=&mut self.bots;
        let walls=&mut self.walls;

        for b in bots.iter_mut(){
            b.update();

        }
        bots[0].pos=cursor;


        let mut tree=DynTree::new(axgeom::XAXISS,(),bots.drain(..).map(|b|{
            let p=b.pos;
            let rect=aabb_from_pointf64(p,[radius;2]);
            BBox::new(rectf64_to_notnan(rect),b)
        }));

        use axgeom::*;
        
        intersect_with::intersect_with_mut(&mut tree,walls,|bot,wall|{
            //let radius=radius/2.0;
            fn sub(a:[f64;2],b:[f64;2])->[f64;2]{
                [b[0]-a[0],b[1]-a[1]]
            }
            fn derive_center(a:Rect<f64>)->[f64;2]{
                let ((a,b),(c,d))=a.get();
                [a+(b-a)/2.0,c+(d-c)/2.0]
            }
            fn dot(a:[f64;2],b:[f64;2])->f64{
               a[0]*b[0]+a[1]*b[1] 
            }

            let botr=rectNaN_to_f64(*bot.get());
            let wallr=rectNaN_to_f64(*wall.get());
            let wallx=wallr.as_axis().get(XAXISS);
            let wally=wallr.as_axis().get(YAXISS);

            
            let center_bot=derive_center(botr);
            let center_wall=derive_center(wallr);

            //bottom_left_to_top_right
            //let p1=[1.0,-1.0];
            //top_left_to_bottom_right
            //let p2=[1.0,1.0];
            let p2=[-1.0,1.0];
            let p1=[1.0,1.0];

            let diff=sub(center_wall,center_bot);

            let d1=f64n!(dot(p1,diff));
            let d2=f64n!(dot(p2,diff));
            let zero=f64n!(0.0);

            use std::cmp::Ordering::*;

            let pos=center_bot;
            let vel=bot.inner.vel;
            let fric=0.6;
            let ret=match (d1.cmp(&zero),d2.cmp(&zero)){
                (Less,Less)=>{
                    //top
                    ([None,Some(wally.left-radius)],
                     [None,Some(-vel[1]*fric)])
                }
                (Less,Equal)=>{
                    //top left
                    ([Some(wallx.left-radius),Some(wally.left-radius)],
                     [Some(-vel[0]*fric),Some(-vel[1]*fric)])
                }
                (Less,Greater)=>{
                    //left
                    ([Some(wallx.left-radius),None],
                     [Some(-vel[0]*fric),None])
                }
                (Greater,Less)=>{
                    //right
                    ([Some(wallx.right+radius),None],
                     [Some(-vel[0]*fric),None])
                }
                (Greater,Equal)=>{
                    //bottom right
                    ([Some(wallx.right+radius),Some(wally.right+radius)],
                     [Some(-vel[0]*fric),Some(-vel[1]*fric)])
                }
                (Greater,Greater)=>{
                    //bottom
                    ([None,Some(wally.right+radius)],
                     [None,Some(-vel[1]*fric)])
                }
                (Equal,Less)=>{
                    //top right
                    ([Some(wallx.right+radius),Some(wally.left-radius)],
                     [Some(-vel[0]*fric),Some(-vel[1]*fric)])
                }
                (Equal,Equal)=>{
                    //center
                    panic!("Sooo unlikely. TODO fix");
                }
                (Equal,Greater)=>{
                    //bottom left
                    ([Some(wallx.left-radius),Some(wally.right+radius)],
                     [Some(-vel[0]*fric),Some(-vel[1]*fric)])
                }
            };

            match ret.0{
                [Some(a),Some(b)]=>{
                    bot.inner.pos[0]=a;
                    bot.inner.pos[1]=b;
                },
                [Some(a),None]=>{
                    bot.inner.pos[0]=a;
                },
                [None,Some(a)]=>{
                    bot.inner.pos[1]=a;
                },
                [None,None]=>{

                }
            }

            match ret.1{
                [Some(a),Some(b)]=>{
                    bot.inner.vel[0]=a;
                    bot.inner.vel[1]=b;
                },
                [Some(a),None]=>{
                    bot.inner.vel[0]=a;
                },
                [None,Some(a)]=>{
                    bot.inner.vel[1]=a;
                },
                [None,None]=>{

                }
            }
            //bot.inner.pos=ret.0;
            //bot.inner.vel=ret.1;
              
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

