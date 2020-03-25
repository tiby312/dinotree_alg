use axgeom::*;
use axgeom::Axis;
use std::collections::BTreeMap;
use axgeom::ordered_float::*;
use duckduckgeo::grid;

#[derive(PartialOrd,PartialEq,Eq,Ord,Copy,Clone)]
struct BotCollisionHash(usize,usize);
impl BotCollisionHash{
    fn new<T>(a:&T,b:&T)->BotCollisionHash{                
        let a=a as *const _ as usize;
        let b=b as *const _ as usize;
        let [a,b]=if a<b{
            [a,b]
        }else{
            [b,a]
        };
        BotCollisionHash(a,b)
    }
}

#[derive(PartialOrd,PartialEq,Eq,Ord,Copy,Clone)]
struct WallCollisionHash{
    a:usize,
    dir:grid::CardDir
}

fn single_hash<T>(a:&T,dir:grid::CardDir)->WallCollisionHash{
    WallCollisionHash{a:a as *const _ as usize,dir}
}

pub trait VelocitySolvable{
    fn pos(&self)->&Vec2<f32>;
    fn vel_mut(&mut self)->&mut Vec2<f32>;
}

pub struct CollisionVelocitySolver{
    last_bot_col:BTreeMap<BotCollisionHash,f32>,
    last_wall_col:BTreeMap<WallCollisionHash,f32>
}

impl CollisionVelocitySolver{
    pub fn new()->CollisionVelocitySolver{
        CollisionVelocitySolver{last_bot_col:BTreeMap::new(),last_wall_col:BTreeMap::new()}
    }
    pub fn solve<A:Axis,T:VelocitySolvable+Send+Sync>(&mut self,radius:f32,grid_viewport:&grid::GridViewPort,walls:&grid::Grid2D,tree:&mut dinotree_alg::collectable::CollectableDinoTree<A,NotNan<f32>,T>){
        
        let diameter=radius*2.0;
        let diameter2=diameter*diameter;
        let bias_factor=0.2;
        let allowed_penetration=radius*0.5;
        let num_iterations=20;
        
    
        let mut collision_list={
            let ka3 = &self.last_bot_col;
            tree.collect_intersections_list_par(|a,b|{
                let offset=*b.pos()-*a.pos();
                let distance2=offset.magnitude2();
                if distance2>0.00001 && distance2<diameter2{
                    let distance=distance2.sqrt();
                    let offset_normal=offset/distance;
                    let separation=(diameter-distance)/2.0;
                    let bias=-bias_factor*(1.0/num_iterations as f32)*( (-separation+allowed_penetration).min(0.0));
                    
                    let hash=BotCollisionHash::new(a,b);
                    let impulse=if let Some(&impulse)=ka3.get(&hash){ //TODO inefficient to check if its none every time
                        let k=offset_normal*impulse;
                        *a.vel_mut()-=k;
                        *b.vel_mut()+=k;
                        impulse
                    }else{
                        0.0
                    };

                    Some((offset_normal,bias,impulse))
                }else{
                    None
                }
            })
        };

        //Package in one struct
        //so that there is no chance of mutating it twice
        #[derive(Debug)]
        struct WallCollision{
            collisions:[Option<(f32,Vec2<f32>,grid::CardDir,f32)>;2],
        }

        let mut wall_collisions={
            let ka3 = &self.last_wall_col;

            tree.collect_all_par(|rect,a|{
                let arr=duckduckgeo::grid::collide::is_colliding(&walls,&grid_viewport,rect.as_ref(),radius);
                let create_collision=|bot:&mut T,dir:grid::CardDir,seperation:f32,offset_normal:Vec2<f32>|{
                    let bias=-bias_factor*(1.0/num_iterations as f32)*( (-seperation+allowed_penetration).min(0.0));

                    let impulse=if let Some(&impulse)=ka3.get(&single_hash(bot,dir)){ //TODO inefficient to check if its none every time
                        let k=offset_normal*impulse;
                        *bot.vel_mut()+=k;
                        impulse
                    }else{
                        0.0
                    };
                    (bias,offset_normal,dir,impulse)
                };
                match arr[0]{
                    Some((seperation,dir,offset_normal))=>{
                        
                        let wall=match arr[1]{
                            Some((seperation,dir,offset_normal))=>{
                                let seperation=seperation*2.0f32.sqrt(); //Since we are pushing diagonally dont want to over push.
                                let first=Some(create_collision(a,dir,seperation,offset_normal));
                                let second=Some(create_collision(a,dir,seperation,offset_normal));
                                WallCollision{collisions:[first,second]}
                            },
                            None=>{
                                let first=Some(create_collision(a,dir,seperation,offset_normal));
                                WallCollision{collisions:[first,None]}
                            }
                        };
                        Some(wall)
                    },
                    None=>{
                        None
                    }
                }
            })
        };

        for _ in 0..num_iterations{

            collision_list.for_every_pair_mut_par(tree,|a,b,&mut (offset_normal,bias,ref mut acc)|{
                
                let vel=*b.vel_mut()-*a.vel_mut();
                let impulse=bias-vel.dot(offset_normal);
                
                let p0=*acc;
                *acc=(p0+impulse).max(0.0);
                let impulse=*acc-p0;
                
                let k=offset_normal*impulse;
                *a.vel_mut()-=k;
                *b.vel_mut()+=k;
            });     

            wall_collisions.for_every_mut_par(tree,|bot,wall|{
                //dbg!(&wall);
                for k in wall.collisions.iter_mut(){
                    if let &mut Some((bias,offset_normal,_dir,ref mut acc))=k{
                        
                        let impulse=bias-bot.vel_mut().dot(offset_normal);

                        let p0=*acc;
                        *acc=(p0+impulse).max(0.0);
                        let impulse=*acc-p0;

                        *bot.vel_mut()+=offset_normal*impulse;
                    }
                }; 
            })
        }  


        self.last_bot_col.clear();
        self.last_wall_col.clear();

        let (ka2,ka3):(BTreeMap<_,_>,BTreeMap<_,_>)=rayon::join(||{
            collision_list.get(&tree).iter().flat_map(|a|a.iter()).map(|(a,b,(_,_,impulse))|{
                (BotCollisionHash::new(a,b),*impulse)
            }).collect()
        },
        ||{
            wall_collisions.get(&tree).iter().flat_map(|(bot,wall)|{
                let k=wall.collisions.iter().filter(|a|a.is_some()).map(|a|a.unwrap());
                k.map(move |(_,_,dir,impulse)|{
                    (single_hash(bot,dir),impulse)
                })
            }).collect()
        });

        self.last_bot_col=ka2;
        self.last_wall_col=ka3;
    }
}
