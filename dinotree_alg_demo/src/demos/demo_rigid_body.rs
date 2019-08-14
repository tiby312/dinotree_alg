use crate::support::prelude::*;
use dinotree_alg::rect;
use duckduckgeo;
use dinotree_alg;

use axgeom::Rect;

#[derive(Copy,Clone,Debug)]
pub struct RigidBody{
    pub pos:Vec2<f32>,
    pub push_vec:Vec2<f32>,
    pub vel:Vec2<f32>,
    pub acc:Vec2<f32>
}



impl duckduckgeo::BorderCollideTrait for RigidBody{
    type N=f32;
    fn pos_vel_mut(&mut self)->(&mut Vec2<f32>,&mut Vec2<f32>){
        (&mut self.pos,&mut self.vel)
    }
}

impl RigidBody{
    pub fn new(pos:Vec2<f32>)->RigidBody{
        let a=vec2same(0.0);
        RigidBody{pos,push_vec:a,vel:a,acc:a}
    }
    pub fn create_loose(&self,radius:f32)->Rect<F32n>{
        axgeom::Rect::from_point(self.pos,vec2same(radius)).inner_try_into().unwrap()
    }
    pub fn push_away(&mut self,b:&mut Self,radius:f32,max_amount:f32)->bool{
        let mut diff=b.pos-self.pos;

        let dis=diff.magnitude();


        if dis>=radius*2.0{
            return false;
        }


        if dis<0.000001{
            self.push_vec+=vec2(0.01,0.0);
            b.push_vec-=vec2(0.01,0.0);
            return false;
        }


        let fff=radius*2.0-dis+0.0001;
        
        let (moved,mag)=if fff<max_amount{
            (true,fff)
        }else{
            (false,max_amount)
        };

        //let mag=max_amount.min( radius*2.0-dis  );
        if mag<0.0{
            panic!("impossible");
            return false;
        }
        //let mag=max_amount;
        diff*=mag/dis;

        self.push_vec-=diff;
        b.push_vec+=diff;

        moved
    }
    pub fn apply_push_vec(&mut self){
        self.pos+=self.push_vec;
        self.push_vec=vec2same(0.0);
    }
}

pub fn handle_rigid_body(
        dim:&Rect<F32n>,
        bodies:&mut [RigidBody],
        ball_size:f32,
        push_rate:f32,
        num_rebal:usize,
        num_query:usize,
        func:impl Fn(&mut RigidBody,&mut RigidBody)+Sync){
    
    for _ in 0..num_rebal{        
        let mut tree=DinoTreeBuilder::new(axgeom::YAXISS,bodies,|a|a.create_loose(ball_size+push_rate*(num_query as f32))).build_par();

        for _ in 0..num_query{
            dinotree_alg::colfind::QueryBuilder::new(&mut tree).query_par(|a,b|{
                let moved_apart = a.inner.push_away(&mut b.inner,ball_size,push_rate);
                if moved_apart{
                    func(&mut a.inner,&mut b.inner);
                }
            });    


            dinotree_alg::rect::for_all_not_in_rect_mut(&mut tree,dim,|a|{
                duckduckgeo::collide_with_border(&mut a.inner,dim.as_ref(),0.5);
            });
        

            for body in tree.get_bots_mut().iter_mut(){
                let body=&mut body.inner;
                let mm=body.push_vec.magnitude();
                if mm>0.0000001{
                    if mm>push_rate{
                        body.push_vec.normalize_to(push_rate);
                    }
                    body.apply_push_vec();
                }
            }

        }

        tree.apply(bodies,|a,b|*b=a.inner);


    }
}






pub struct RigidBodyDemo{
    radius:f32,
    bots:Vec<RigidBody>,
    dim:Rect<F32n>
}
impl RigidBodyDemo{
    pub fn new(dim:Rect<F32n>)->RigidBodyDemo{
        
        let mut bots:Vec<_>=UniformRandGen::new(dim.inner_into()).
            take(400).map(|pos|{
                RigidBody::new(pos)
        }).collect();

        bots[0].vel=vec2(1.,1.);
 
        RigidBodyDemo{radius:10.0,bots,dim}
    }
}

impl DemoSys for RigidBodyDemo{
    fn step(&mut self,cursor:Vec2<F32n>,c:&piston_window::Context,g:&mut piston_window::G2d,_check_naive:bool){
        let radius=self.radius;
        


        handle_rigid_body(&self.dim,&mut self.bots,self.radius,self.radius*0.2,2,3,|a,b|{
            let rect1=&axgeom::Rect::from_point(a.pos,vec2same(radius));
            let rect2=&axgeom::Rect::from_point(b.pos,vec2same(radius));
            

            let cc=0.5;

            let pos_diff=b.pos-a.pos;

            let pos_diff_norm=pos_diff.normalize_to(1.0);

            let vel_diff=b.vel-a.vel;

            let im1=1.0;
            let im2=1.0;

            let vn=vel_diff.dot(pos_diff_norm);
            if vn>0.0{
                return;
            }

            let i = (-(1.0 + cc) * vn) / (im1 + im2);
            let impulse = pos_diff_norm*i;


            //draw_rect_f32([1.0,0.0,0.0,1.0],rect1,c,g);
            //draw_rect_f32([1.0,0.0,0.0,1.0],rect2,c,g);


            a.vel-=impulse*im1;
            b.vel+=impulse*im2;

        });

        
        let mut tree=DinoTreeBuilder::new(axgeom::XAXISS,&self.bots,|bot|{
            bot.create_loose(radius)
        }).build_par(); 
        
        rect::for_all_in_rect_mut(&mut tree,&axgeom::Rect::from_point(cursor,vec2same(100.0+radius).inner_try_into().unwrap()),|b|{
            let diff=cursor.inner_into()-b.inner.pos;

            let dis=diff.magnitude();
            if dis<100.0{
                b.inner.acc-=diff*0.01;
                /*
                let mag=100.0-dis;
                if mag>0.0{
                    b.inner.pos-=diff*(mag/dis);    
                }
                */
            }
        });
        
        /*
        colfind::QueryBuilder::new(tree.as_ref_mut()).query_par(|a, b| {
            let _ = duckduckgeo::repel(a,b,0.001,2.0,|a|a.sqrt());
        });
        */
        
        tree.apply(&mut self.bots,|b,t|*t=b.inner);
        

        for b in self.bots.iter_mut(){
            b.pos+=b.vel;
            b.vel+=b.acc;
            b.acc=vec2same(0.0);

            duckduckgeo::collide_with_border(b,self.dim.as_ref(),0.5);
            
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
            draw_rect_f32([0.0,1.0,1.0,0.4],rect,c,g);
        }        
    }
}

