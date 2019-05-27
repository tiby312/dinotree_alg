use crate::support::prelude::*;
use dinotree_alg::colfind;
use dinotree_alg::rect;
use duckduckgeo;
use dinotree_alg;

use axgeom::Rect;
use duckduckgeo::vec2f64::Vec2;

#[derive(Copy,Clone,Debug)]
pub struct RigidBody{
    pub pos:Vec2,
    pub push_vec:Vec2,
}

impl RigidBody{
    pub fn new(pos:Vec2)->RigidBody{
        let push_vec=Vec2::new(0.0,0.0);
        RigidBody{pos,push_vec}
    }
    pub fn create_loose(&self,radius:f64)->Rect<F64n>{
        Conv::from_rect(aabb_from_pointf64(self.pos.0,[radius;2]))
    }
    pub fn push_away(&mut self,b:&mut Self,radius:f64,max_amount:f64){
        let mut diff=b.pos-self.pos;

        let dis=diff.dis();

        if dis<0.000001{
            return;
        }

        let mag=max_amount.min(radius*2.0-dis);
        if mag<0.0{
            return;
        }
        //let mag=max_amount;
        diff*=mag/dis;

        self.push_vec-=diff;
        b.push_vec+=diff;

        //TODO if we have moved too far away, move back to point of collision!!!
        {

        }
    }
    pub fn apply_push_vec(&mut self){
        self.pos+=self.push_vec;
        self.push_vec.set_zero();
    }
}

pub fn handle_rigid_body(bodies:&mut [RigidBody],ball_size:f64,push_unit:f64,num_iteration:usize){
    
    let push_rate=push_unit/ (num_iteration as f64);

    for _ in 0..num_iteration{        
        let mut tree=DinoTreeBuilder::new(axgeom::YAXISS,bodies,|a|a.create_loose(ball_size+push_rate)).build_par();

        dinotree_alg::colfind::QueryBuilder::new(&mut tree).query_par(|a,b|{
            a.inner.push_away(&mut b.inner,ball_size,push_rate);
        });    

        tree.apply(bodies,|a,b|*b=a.inner);

        for body in bodies.iter_mut(){
            if body.push_vec.dis()>0.0000001{
                body.push_vec.truncate(push_rate);
                body.apply_push_vec();
            }
        }
    }
}






pub struct RigidBodyDemo{
    radius:f64,
    bots:Vec<RigidBody>,
    dim:[f64;2]
}
impl RigidBodyDemo{
    pub fn new(dim:[f64;2])->RigidBodyDemo{
        let dim=[dim[0],dim[1]-100.0];
        let dim2=&[0,dim[0] as isize,0,dim[1] as isize];
        let radius=[2,5];
        let velocity=[0,1];
        let bots=create_world_generator(1300,dim2,radius,velocity).enumerate().map(|(id,ret)|{
            RigidBody::new(Vec2(ret.pos))//{pos:ret.pos,vel:ret.vel,force:[0.0;2],id,aabb:Conv::from_rect(aabb_from_pointf64(ret.pos,[5.0;2]))}
        }).collect();
 
        RigidBodyDemo{radius:10.0,bots,dim}
    }
}

impl DemoSys for RigidBodyDemo{
    fn step(&mut self,cursor:[f64;2],c:&piston_window::Context,g:&mut piston_window::G2d,check_naive:bool){
        let radius=self.radius;
        


        handle_rigid_body(&mut self.bots,self.radius,self.radius*2.0,20);

        
        let mut tree=DinoTreeBuilder::new(axgeom::XAXISS,&self.bots,|bot|{
            bot.create_loose(radius)
        }).build_par(); 
        
        rect::for_all_in_rect_mut(&mut tree,&Conv::from_rect(aabb_from_pointf64(cursor,[100.0+radius;2])),|b|{
            let diff=Vec2(cursor)-b.inner.pos;

            let dis=diff.dis();
            if dis<100.0{
                let mag=100.0-dis;
                if mag>0.0{
                    b.inner.pos-=diff*(mag/dis);    
                }
            }
        });
        
        /*
        colfind::QueryBuilder::new(tree.as_ref_mut()).query_par(|a, b| {
            let _ = duckduckgeo::repel(a,b,0.001,2.0,|a|a.sqrt());
        });
        */
        
        tree.apply(&mut self.bots,|b,t|*t=b.inner);
        

        for b in self.bots.iter_mut(){
            //b.update();
            //b.aabb=Conv::from_rect(aabb_from_pointf64(b.pos,[radius;2]));
            duckduckgeo::stop_wall(&mut b.pos.0,self.dim);
        }

        /*
        //If you dont care about the order, you can do this instead.
        //But in this case, this will cause the colors to not be assigned to the correct bots.
        for (a,b) in tree.iter_every_bot().zip(bots.iter_mut()){
            *b=a.inner;
        }
        */
        
        for bot in self.bots.iter(){
            let rect=&Conv::from_rect(aabb_from_pointf64(bot.pos.0,[radius;2]));
            draw_rect_f64n([0.0,1.0,1.0,1.0],rect,c,g);
        }        
    }
}

