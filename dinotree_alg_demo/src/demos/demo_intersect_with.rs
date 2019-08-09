use crate::support::prelude::*;
use dinotree_alg::colfind;
use dinotree_alg::rect;
use dinotree_alg::intersect_with;
use duckduckgeo;


#[derive(Copy,Clone)]
pub struct Bot{
    pos:Vec2<f32>,
    vel:Vec2<f32>,
    force:Vec2<f32>,
    wall_move:[Option<(F32n,f32)>;2]
}
impl duckduckgeo::RepelTrait for Bot{
    type N=f32;
    fn pos(&self)->Vec2<f32>{
        self.pos
    }
    fn add_force(&mut self,force:Vec2<f32>){
        self.force+=force;
    }
}


impl Bot{
    fn update(&mut self){
        self.vel+=self.force;
        //non linear drag
        self.vel*=0.9;

        self.pos+=self.vel;

        self.force=vec2same(0.0);
    }
}

#[derive(Copy,Clone)]
struct Wall(axgeom::Rect<F32n>);




pub struct IntersectWithDemo{
    radius:f32,
    bots:Vec<Bot>,
    walls:Vec<Wall>,
    dim:Rect<F32n>
}
impl IntersectWithDemo{
    pub fn new(dim:Rect<F32n>)->IntersectWithDemo{


        let bots:Vec<_>=UniformRandGen::new(dim.inner_into()).
            take(4000).map(|pos|{
            Bot{pos,vel:vec2same(0.0),force:vec2same(0.0),wall_move:[None;2]}
        }).collect();

        let walls=UniformRandGen::new(dim.inner_into()).with_radius(10.0,60.0).take(40).map(|(pos,radius)|{
            Wall(Rect::from_point(pos,radius).inner_try_into().unwrap())
        }).collect();


        IntersectWithDemo{radius:5.0,bots,walls,dim}
    }
}

impl DemoSys for IntersectWithDemo{
    fn step(&mut self,cursor:Vec2<F32n>,c:&piston_window::Context,g:&mut piston_window::G2d,_check_naive:bool){
        let radius=self.radius;
        let bots=&mut self.bots;
        let walls=&mut self.walls;

        for b in bots.iter_mut(){
            b.update();
            
            if let Some((pos,vel))=b.wall_move[0]{
                b.pos.x=pos.into_inner();
                b.vel.x=vel;
            }

            if let Some((pos,vel))=b.wall_move[1]{    
                b.pos.y=pos.into_inner();
                b.vel.y=vel;
            }

            b.wall_move[0]=None;
            b.wall_move[1]=None;
            
            duckduckgeo::wrap_position(&mut b.pos,self.dim.inner_into());
        }
        bots[0].pos=cursor.inner_into();


        let mut tree=DinoTreeBuilder::new(axgeom::XAXISS,&bots,|bot|{
           axgeom::Rect::from_point(bot.pos,vec2(radius,radius)).inner_try_into().unwrap()
        }).build_par(); 

        intersect_with::intersect_with_mut(&mut tree,walls,|wall|{wall.0},|bot,wall|{
            let fric=0.8;


            let wallx=&wall.get().x;
            let wally=&wall.get().y;
            let vel=bot.inner.vel;

            let ret=match duckduckgeo::collide_with_rect::<f32>(bot.get().as_ref(),wall.get().as_ref()).unwrap(){
                duckduckgeo::WallSide::Above=>{
                    [None,Some((wally.left-radius,-vel.y*fric))]
                },
                duckduckgeo::WallSide::Below=>{
                    [None,Some((wally.right+radius,-vel.y*fric))]
                },
                duckduckgeo::WallSide::LeftOf=>{
                    [Some((wallx.left-radius,-vel.x*fric)),None]
                },
                duckduckgeo::WallSide::RightOf=>{
                    [Some((wallx.right+radius,-vel.x*fric)),None]
                }
            };
            bot.inner.wall_move=ret;
        });

        let cc=cursor.inner_into();
        rect::for_all_in_rect_mut(&mut tree,&axgeom::Rect::from_point(cc,vec2same(100.0)).inner_try_into().unwrap(),|b|{
            let _ =duckduckgeo::repel_one(&mut b.inner,cc,0.001,20.0);
        });
        

        for wall in walls.iter(){
            draw_rect_f32([0.0,0.0,1.0,0.3],wall.0.as_ref(),c,g);
        }
        for bot in tree.get_bots().iter(){
            draw_rect_f32([0.0,0.0,0.0,0.3],bot.get().as_ref(),c,g);
        }
 
        colfind::QueryBuilder::new(&mut tree).query_par(|a, b| {
            let _ = duckduckgeo::repel(&mut a.inner,&mut b.inner,0.001,2.0);
        });
    
        

        tree.apply(bots,|b,t|*t=b.inner);
        
     }
}

