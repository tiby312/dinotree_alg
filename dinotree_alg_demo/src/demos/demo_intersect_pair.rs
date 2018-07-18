use support::prelude::*;
use dinotree::colfind;
use dinotree::rect;
use dinotree_geom;

#[derive(Copy,Clone)]
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
        let radius=self.radius;
        let bots=&mut self.bots;

        for b in bots.iter_mut(){
            b.update();
            dinotree_geom::wrap_position(&mut b.pos,self.dim);
        }


        let mut tree=DynTree::new(axgeom::XAXISS,(),&bots,|b|{Conv::from_rect(aabb_from_pointf64(b.pos,[radius;2]))});


        rect::for_all_in_rect_mut(&mut tree,&Conv::from_rect(aabb_from_pointf64(cursor,[100.0;2])),|b|{
            b.inner.repel_mouse(cursor);
        });
        
        for bot in tree.iter_every_bot(){
            draw_rect_f64n([0.0,0.0,0.0,0.3],bot.get(),c,g);
        }
    
        colfind::query_seq_mut(&mut tree,|a, b| {
            a.inner.repel(&mut b.inner);
            draw_rect_f64n([1.0,0.0,0.0,0.2],a.get(),c,g);
            draw_rect_f64n([1.0,0.0,0.0,0.2],b.get(),c,g);
        });
    
    
        for (b,ff) in tree.into_iter_orig_order().zip(bots.iter_mut()){
            *ff=b.inner;
        }
     }
}

