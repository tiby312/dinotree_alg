use crate::support::prelude::*;
use dinotree_alg::rect;
use duckduckgeo;
use dinotree_alg;

use axgeom::Rect;

#[derive(Copy,Clone,Debug)]
pub struct RigidBody{
    pub pos:Vec2<f32>,
    pub push_vec:Vec2<f32>,
}

impl RigidBody{
    pub fn new(pos:Vec2<f32>)->RigidBody{
        let push_vec=vec2same(0.0);
        RigidBody{pos,push_vec}
    }
    pub fn create_loose(&self,radius:f32)->Rect<F32n>{
        axgeom::Rect::from_point(self.pos,vec2same(radius)).inner_try_into().unwrap()
    }
    pub fn push_away(&mut self,b:&mut Self,radius:f32,max_amount:f32){
        let mut diff=b.pos-self.pos;

        let dis=diff.magnitude();

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
        self.push_vec=vec2same(0.0);
    }
}

pub fn handle_rigid_body(bodies:&mut [RigidBody],ball_size:f32,push_unit:f32,num_iteration:usize){
    
    let push_rate=push_unit/ (num_iteration as f32);

    for _ in 0..num_iteration{        
        let mut tree=DinoTreeBuilder::new(axgeom::YAXISS,bodies,|a|a.create_loose(ball_size+push_rate)).build_par();

        dinotree_alg::colfind::QueryBuilder::new(&mut tree).query_par(|a,b|{
            a.inner.push_away(&mut b.inner,ball_size,push_rate);
        });    

        tree.apply(bodies,|a,b|*b=a.inner);

        for body in bodies.iter_mut(){
            let mm=body.push_vec.magnitude();
            if mm>0.0000001{
                if mm>push_rate{
                    body.push_vec.normalize_to(push_rate);
                }
                body.apply_push_vec();
            }
        }
    }
}






pub struct RigidBodyDemo{
    radius:f32,
    bots:Vec<RigidBody>,
    dim:Rect<F32n>
}
impl RigidBodyDemo{
    pub fn new(dim:Rect<F32n>)->RigidBodyDemo{
        
        let bots:Vec<_>=UniformRandGen::new(dim.inner_into()).
            take(1000).map(|pos|{
                RigidBody::new(pos)
        }).collect();

 
        RigidBodyDemo{radius:10.0,bots,dim}
    }
}

impl DemoSys for RigidBodyDemo{
    fn step(&mut self,cursor:Vec2<F32n>,c:&piston_window::Context,g:&mut piston_window::G2d,_check_naive:bool){
        let radius=self.radius;
        


        handle_rigid_body(&mut self.bots,self.radius,self.radius*2.0,20);

        
        let mut tree=DinoTreeBuilder::new(axgeom::XAXISS,&self.bots,|bot|{
            bot.create_loose(radius)
        }).build_par(); 
        
        rect::for_all_in_rect_mut(&mut tree,&axgeom::Rect::from_point(cursor,vec2same(100.0+radius).inner_try_into().unwrap()),|b|{
            let diff=cursor.inner_into()-b.inner.pos;

            let dis=diff.magnitude();
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
            
            duckduckgeo::stop_wall(&mut b.pos,self.dim.inner_into());
            
        }

        /*
        //If you dont care about the order, you can do this instead.
        //But in this case, this will cause the colors to not be assigned to the correct bots.
        for (a,b) in tree.iter_every_bot().zip(bots.iter_mut()){
            *b=a.inner;
        }
        */
        
        for bot in self.bots.iter(){
            let rect=&axgeom::Rect::from_point(bot.pos,vec2same(radius));
            draw_rect_f32([0.0,1.0,1.0,1.0],rect,c,g);
        }        
    }
}

