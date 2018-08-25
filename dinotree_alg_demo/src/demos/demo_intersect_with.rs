use support::prelude::*;
use dinotree_alg::colfind;
use dinotree_alg::rect;
use dinotree_alg::intersect_with;
use dinotree_geom;


#[derive(Copy,Clone)]
pub struct Bot{
    pos:[f64;2],
    vel:[f64;2],
    force:[f64;2],
    wall_move:[Option<(F64n,f64)>;2]
}
impl dinotree_geom::RepelTrait for Bot{
    type N=f64;
    fn pos(&self)->[f64;2]{
        self.pos
    }
    fn add_force(&mut self,force:[f64;2]){
        self.force[0]+=force[0];
        self.force[1]+=force[1];
    }
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
}


struct Wall(axgeom::Rect<F64n>);

unsafe impl HasAabb for Wall{
    type Num=F64n;
    fn get(&self)->&axgeom::Rect<F64n>{
        &self.0
    }
}

pub struct IntersectWithDemo{
    radius:f64,
    bots:Vec<Bot>,
    walls:Vec<Wall>,
    dim:[f64;2]
}
impl IntersectWithDemo{
    pub fn new(dim:[f64;2])->IntersectWithDemo{
        let dim2=&[0,dim[0] as isize,0,dim[1] as isize];
        let radius=[3,5];
        let velocity=[1,3];
        let bots=create_world_generator(4000,dim2,radius,velocity).map(|ret|{
            Bot{pos:ret.pos,vel:ret.vel,force:[0.0;2],wall_move:[None;2]}
        }).collect();

        let radius=[10,60];
        let walls=create_world_generator(40,dim2,radius,velocity).map(|ret|{
            let rect=aabb_from_pointf64(ret.pos,ret.radius);
            Wall(Conv::from_rect(rect))
        }).collect();

        IntersectWithDemo{radius:5.0,bots,walls,dim}
    }
}

impl DemoSys for IntersectWithDemo{
    fn step(&mut self,cursor:[f64;2],c:&piston_window::Context,g:&mut piston_window::G2d,_check_naive:bool){
        let radius=self.radius;
        let bots=&mut self.bots;
        let walls=&mut self.walls;

        for b in bots.iter_mut(){
            b.update();
            
            match b.wall_move[0]{
                Some((pos,vel))=>{
                    b.pos[0]=pos.into_inner();
                    b.vel[0]=vel;
                },
                None=>{}
            }
            match b.wall_move[1]{
                Some((pos,vel))=>{
                    b.pos[1]=pos.into_inner();
                    b.vel[1]=vel;
                },
                None=>{}
            }
            b.wall_move[0]=None;
            b.wall_move[1]=None;
            
            dinotree_geom::wrap_position(&mut b.pos,self.dim);
        }
        bots[0].pos=cursor;


        let mut tree=DynTree::new(axgeom::XAXISS,(),&bots,|bot|{
           Conv::from_rect(aabb_from_pointf64(bot.pos,[radius;2]))
        }); 

        intersect_with::intersect_with_mut(&mut tree,walls,|bot,wall|{
            let fric=0.8;


            let wallx=wall.get().get_range(axgeom::XAXISS);
            let wally=wall.get().get_range(axgeom::YAXISS);
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
            bot.inner.wall_move=ret;
        });

        /*    
        let cont=tree.compute_tree_health().collect::<Vec<f64>>();
        let sum=cont.iter().fold(0.0,|sum,a|sum+a);
        println!("tree health={:?} sum={:?}",cont,sum);
        */
        
        rect::for_all_in_rect_mut(&mut tree,&Conv::from_rect(aabb_from_pointf64(cursor,[100.0;2])),|b|{
            //b.inner.repel_mouse(cursor);
            let _ =dinotree_geom::repel_one(&mut b.inner,cursor,0.001,20.0,|a|a.sqrt());
        });
        

        for wall in walls.iter(){
            draw_rect_f64n([0.0,0.0,1.0,0.3],wall.get(),c,g);
        }
        for bot in tree.iter_every_bot(){
            draw_rect_f64n([0.0,0.0,0.0,0.3],bot.get(),c,g);
        }
 
        colfind::query_mut(&mut tree,|a, b| {
            let _ = dinotree_geom::repel(&mut a.inner,&mut b.inner,0.001,2.0,|a|a.sqrt());
        });
    
        

        tree.apply_orig_order(bots,|b,t|*t=b.inner);
        
     }
}

