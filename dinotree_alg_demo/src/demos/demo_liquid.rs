use crate::support::prelude::*;
use dinotree_alg::rect;
use duckduckgeo;
use dinotree_alg;
use dinotree_alg::colfind;

use axgeom::Rect;

#[derive(Copy,Clone,Debug)]
pub struct Liquid{
    pub pos:Vec2<f32>,
    pub vel:Vec2<f32>,
    pub acc:Vec2<f32>
}

impl Liquid{
    pub fn new(pos:Vec2<f32>)->Liquid{
        let z=vec2same(0.0);

        Liquid{pos,acc:z,vel:z}
    }


    pub fn solve(&mut self,b:&mut Self,radius:f32)->f32{
        let mut diff=b.pos-self.pos;

        let dis_sqr=diff.magnitude2();

        if dis_sqr<0.000001{
            //TODO push them apart instead.
            return 0.0;
        }


        if dis_sqr >= (2.*radius)*(2.*radius) {
            //They not touching (bots are circular).
            return 0.0;
        }


        let dis=dis_sqr.sqrt();

        //d is zero if barely touching, 1 is overlapping.
        //d grows linearly with position of bots
        let d=1.0- (dis/(radius*2.));


        let spring_force_mag=-(d-0.5)*0.02;




        let velociy_diff=b.vel-self.vel;
        let damping_ratio=0.0002;
        let spring_dampen=velociy_diff.dot(diff)*(1./dis)*damping_ratio;


        let spring_force=diff*(1./dis)*(spring_force_mag + spring_dampen );

        self.acc+=spring_force;
        b.acc-=spring_force;
    




        spring_force_mag
        
    }
}


impl duckduckgeo::BorderCollideTrait for Liquid{
    type N=f32;
    fn pos_vel_mut(&mut self)->(&mut Vec2<f32>,&mut Vec2<f32>){
        (&mut self.pos,&mut self.vel)
    }
}



impl duckduckgeo::RepelTrait for Liquid{
    type N=f32;
    fn pos(&self)->Vec2<f32>{
        self.pos
    }
    fn add_force(&mut self,acc:Vec2<f32>){
        self.acc+=acc;
    }
}



pub struct LiquidDemo{
    radius:f32,
    bots:Vec<Liquid>,
    dim:Rect<F32n>
}
impl LiquidDemo{
    pub fn new(dim:Rect<F32n>)->LiquidDemo{
        
        let bots:Vec<_>=UniformRandGen::new(dim.inner_into()).
            take(1000).map(|pos|{
                Liquid::new(pos)
        }).collect();

 
        LiquidDemo{radius:50.0,bots,dim}
    }
}

impl DemoSys for LiquidDemo{
    fn step(&mut self,cursor:Vec2<F32n>,c:&piston_window::Context,g:&mut piston_window::G2d,_check_naive:bool){
        let radius=self.radius;
        

        
        let mut tree=DinoTreeBuilder::new(axgeom::XAXISS,&self.bots,|bot|{
            let p=bot.pos;
            let r=radius;
            Rect::new(p.x-r,p.x+r,p.y-r,p.y+r).inner_try_into::<NotNan<f32>>().unwrap()
        }).build_par(); 
        
        
        
        colfind::QueryBuilder::new(&mut tree).query_par(|a, b| {
            //let _ = duckduckgeo::repel(a,b,0.001,2.0,|a|a.sqrt());

            let arr = [a.inner.pos.x as f64,a.inner.pos.y as f64,b.inner.pos.x as f64,b.inner.pos.y as f64];



            let mag = a.inner.solve(&mut b.inner,radius);

            /*
            let arr=[a.inner.pos.x as f64,a.inner.pos.y as f64,b.inner.pos.x as f64,b.inner.pos.y as f64];

            let col=if mag>0.0{
                [1.0, 0.0, 0.0, 1.0]
            }else{
                [0.0, 0.0, 1.0, 1.0]
            };

            line(col, // black
                 2.0, // radius of line
                 arr, // [x0, y0, x1,y1] coordinates of line
                 c.transform,
                 g);
            */
        });


        let vv=vec2same(100.0).inner_try_into().unwrap();
        let cc=cursor.inner_into();
        rect::for_all_in_rect_mut(&mut tree,&axgeom::Rect::from_point(cursor,vv),|b|{
            let _ =duckduckgeo::repel_one(&mut b.inner,cc,0.001,100.0);
        });
        

        {
            let dim2=self.dim.inner_into();
            dinotree_alg::rect::for_all_not_in_rect_mut(&mut tree,&self.dim,|a|{
                duckduckgeo::collide_with_border(&mut a.inner,&dim2,0.5);
            });
        }        
        
        tree.apply(&mut self.bots,|b,t|*t=b.inner);
        

        //println!("{:?}",self.bots[0].acc);

        for b in self.bots.iter_mut(){
            /*
            let arr=[b.pos.x as f64,b.pos.y as f64,(b.pos.x+b.acc.x*500.0) as f64,(b.pos.y+b.acc.y*500.0) as f64];

            line([0.0,0.0,0.0,0.8], // black
                 2.0, // radius of line
                 arr, // [x0, y0, x1,y1] coordinates of line
                 c.transform,
                 g);

            */
            b.pos+=b.vel;
            b.vel+=b.acc;
            b.acc=vec2same(0.0);
        }

        
        for bot in self.bots.iter(){
            let rect=&axgeom::Rect::from_point(bot.pos,vec2same(2.0));
            draw_rect_f32([0.0,1.0,1.0,1.0],rect,c,g);
        } 
               
    }
}

