use crate::support::prelude::*;
use duckduckgeo;

mod maps{
    use axgeom::vec2;
    use duckduckgeo::grid::*;
    pub const GRID_STR1:Map<'static>= Map{dim:vec2(16,12),str:"\
████████████████
█    █   █     █
█    █   █  █  █
█        █  █  █
█    █████  █  █
█      █       █
█      █    █  █
█              █
█   █          █
█        █     █
█              █
████████████████
"};
}


use std::collections::BTreeMap;

#[derive(PartialOrd,PartialEq,Eq,Ord,Copy,Clone)]
pub struct BotCollisionHash(usize,usize);
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

fn single_hash<T>(a:&T)->usize{
    a as *const _ as usize
}



#[derive(Copy, Clone)]
pub struct Bot {
    pos: Vec2<f32>,
    vel: Vec2<f32>
}

use std::time::Instant;


pub fn make_demo(dim: Rect<F32n>,canvas:&mut SimpleCanvas) -> Demo {
    let num_bot = 3000;
    //let num_bot=100;

    let radius = 4.0;
    let diameter=radius*2.0;
    let diameter2=diameter*diameter;

    let mut bots: Vec<_> = UniformRandGen::new(dim.inner_into())
        .take(num_bot)
        .map(|pos| Bot {
            pos,
            vel: vec2same(0.0)
        })
        .collect();

    let mut counter: f32=0.0;


    let walls = duckduckgeo::grid::Grid2D::from_str(maps::GRID_STR1);
    let grid_viewport=duckduckgeo::grid::GridViewPort{origin:vec2(0.0,0.0),spacing:dim.x.distance().into_inner()/maps::GRID_STR1.dim.x as f32};


    let wall_save={
        let mut squares=canvas.squares();
         for x in 0..walls.dim().x {
            for y in 0..walls.dim().y {
                let curr=vec2(x,y);
                if walls.get(curr) {
                    let pos=grid_viewport.to_world_center(vec2(x, y));
                    squares.add(pos.into());
                }
            }
        }
        squares.save(canvas)
    };


    let mut ka:Option<(BTreeMap<_,_>,BTreeMap<_,_>)>=None;


    Demo::new(move |cursor, canvas, _check_naive| {
        for _ in 0..1{
            let now = Instant::now();
            

            let mut tree=dinotree_alg::collectable::CollectableDinoTree::new(&mut bots,|b| {
                Rect::from_point(b.pos, vec2same(radius))
                    .inner_try_into()
                    .unwrap()
            });

            let a1=now.elapsed().as_millis();

            
            //TODO move this outside of loop?
            tree.get_mut().for_all_not_in_rect_mut(&dim, |a| {
                let pos=walls.find_closest_empty(grid_viewport.to_grid(a.pos)).unwrap();
                a.pos=grid_viewport.to_world_center(pos);
            });
        

            let vv = vec2same(200.0).inner_try_into().unwrap();
            
            tree.get_mut().for_all_in_rect_mut(&axgeom::Rect::from_point(cursor, vv), | b| {
                let offset=b.pos-cursor.inner_into();
                if offset.magnitude()<200.0*0.5{
                    let k=offset.normalize_to(0.02);
                    b.vel-=k;
                }
            });

           
            let a2=now.elapsed().as_millis();

            let bias_factor=0.3;
            let allowed_penetration=radius*0.1;
            let num_iterations=20;
            
        
            let mut collision_list={
                let ka3 = ka.as_ref();
                tree.collect_collisions_list_par(|a,b|{
                    let offset=b.pos-a.pos;
                    let distance2=offset.magnitude2();
                    if distance2>0.00001 && distance2<diameter2{
                        let distance=distance2.sqrt();
                        let offset_normal=offset/distance;
                        
                        let separation=(diameter-distance)/2.0;
                        assert!(separation>=0.0);
                        let bias=-bias_factor*(1.0/num_iterations as f32)*( (-separation+allowed_penetration).min(0.0));
                        
                        if bias<0.0{
                            //dbg!(bias);
                        }

                        let hash=BotCollisionHash::new(a,b);
                        let impulse=if let Some(&impulse)=ka3.and_then(|(j,_)|j.get(&hash)){ //TODO inefficient to check if its none every time
                            let k=offset_normal*impulse;
                            a.vel-=k;
                            b.vel+=k;
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
            struct WallCollision{
                collisions:[Option<(f32,Vec2<f32>,f32)>;2],
            }

            let mut wall_collisions={
                let ka3 = ka.as_ref();

                tree.collect_all(|rect,a|{
                    let arr=duckduckgeo::grid::collide::is_colliding(&walls,&grid_viewport,rect.as_ref(),radius);
                    let create_collision=|bot:&mut Bot,seperation:f32,offset_normal:Vec2<f32>|{
                        let bias=-bias_factor*(1.0/num_iterations as f32)*( (-seperation+allowed_penetration).max(0.0));

                        let impulse=if let Some(&impulse)=ka3.and_then(|(_,j)|j.get(&single_hash(bot))){ //TODO inefficient to check if its none every time
                            let k=offset_normal*impulse;
                            bot.vel+=k;
                            impulse
                        }else{
                            0.0
                        };
                        (bias,offset_normal,impulse)
                    };
                    match arr[0]{
                        Some((seperation,offset_normal))=>{
                            let first=Some(create_collision(a,seperation,offset_normal));

                            let wall=match arr[1]{
                                Some((seperation,offset_normal))=>{
                                    let second=Some(create_collision(a,seperation,offset_normal));
                                    WallCollision{collisions:[first,second]}
                                },
                                None=>{
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

            //integrate forvces
            for b in tree.get_bots_mut().iter_mut() {
                if b.vel.is_nan(){
                    b.vel=vec2same(0.0);
                }

                let mag2=b.vel.magnitude2();
                let drag_force=mag2*0.005;
                let ff=b.vel/mag2.sqrt()*drag_force;
                let a=b.vel-ff;
                if !a.is_nan(){
                    b.vel=a;
                }
                let g=0.01;
                b.vel+=vec2(g*counter.cos(),g*counter.sin());
            }

            let a3=now.elapsed().as_millis();   
            
            for _ in 0..num_iterations{

                collision_list.for_every_pair_par(&mut tree,|a,b,&mut (offset_normal,bias,ref mut acc)|{
                    
                    let vel=b.vel-a.vel;
                    let impulse=bias-vel.dot(offset_normal);
                    
                    let p0=*acc;
                    *acc=(p0+impulse).max(0.0);
                    let impulse=*acc-p0;
                    
                    let k=offset_normal*impulse;
                    a.vel-=k;
                    b.vel+=k;
                });     

                wall_collisions.for_every(&mut tree,|bot,wall|{
                    for k in wall.collisions.iter_mut(){
                        if let &mut Some((bias,offset_normal,ref mut acc))=k{
                            
                            let impulse=bias-bot.vel.dot(offset_normal);

                            let p0=*acc;
                            *acc=(p0+impulse).max(0.0);
                            let impulse=*acc-p0;

                            bot.vel+=offset_normal*impulse;
                        }
                    }; 
                })
            }


            let mut ka2=BTreeMap::new();
            collision_list.for_every_pair(&mut tree,|a,b,&mut (_,_,impulse)|{
                let hash=BotCollisionHash::new(a,b);
                ka2.insert(hash,impulse);
            });

            let mut ka3=BTreeMap::new();
            wall_collisions.for_every(&mut tree,|bot,wall|{
                for k in wall.collisions.iter_mut(){
                    if let &mut Some((_,_,impulse))=k{
                        ka3.insert(single_hash(bot),impulse);
                    }
                } 
            });
            
            ka=Some((ka2,ka3));

            let a4=now.elapsed().as_millis();

            //integrate position
            for b in bots.iter_mut() {
                b.pos+=b.vel;
            }
            
            counter+=0.01;
            println!("yo= {} {} {} {}",a1,a2-a1,a3-a2,a4-a3);
        }

        wall_save.uniforms(canvas,grid_viewport.spacing).draw();

        //Draw circles
        let mut circles = canvas.circles();
        for b in bots.iter(){
            circles.add(b.pos.into());
        }
        circles.send_and_uniforms(canvas,diameter-4.0).with_color([1.0, 1.0, 0.0, 0.6]).draw();
        
        //Draw arrow
        let dim:Rect<f32>=dim.inner_into();
        let start=[dim.x.distance()/2.0,dim.y.distance()/2.0];
        let end=[start[0]+counter.cos()*200.0,start[1]+counter.sin()*200.0];
        canvas.arrows(20.0).add(start,end).send_and_uniforms(canvas).with_color([0.5,0.3,1.0,0.8]).draw();        
    })
}


