use crate::support::prelude::*;
use duckduckgeo;

mod maps{
    use axgeom::vec2;
    use duckduckgeo::grid::*;
    pub const GRID_STR1:Map<'static>= Map{dim:vec2(16,12),str:"\
████████████████
█    █         █
█   █  █ █  █  █
█  █  █  █ █   █
█ █  █  █   ██ █
█   █  █     █ █
█     █     █  █
█ █  █   █ █   █
█   █   █   █  █
█        █   █ █
█         █    █
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

#[derive(PartialOrd,PartialEq,Eq,Ord,Copy,Clone)]
pub struct WallCollisionHash{
    a:usize,
    dir:grid::CardDir
}
fn single_hash<T>(a:&T,dir:grid::CardDir)->WallCollisionHash{
    WallCollisionHash{a:a as *const _ as usize,dir}
    //a as *const _ as usize
}



type WorldNum=f32;

fn cast_ray(grid:&grid::GridViewPort,walls:&grid::Grid2D,point:Vec2<WorldNum>,dir:Vec2<WorldNum>,max_tval:WorldNum)->Option<grid::raycast::CollideCellEvent>{

    let ray=axgeom::Ray{point,dir};
    
    let caster= grid::raycast::RayCaster::new(grid,ray);
    

    if let Some(wall)=walls.get_option(grid.to_grid(point)){
        let grid_mod=grid.to_grid_mod(point);
        if wall{
            return None;
        }
        //assert!(!wall,"We are starting the raycast inside a wall! point:{:?} grid mod:{:?}",point,grid_mod);
    }


    for a in caster{
        if a.tval<=max_tval{                
            match walls.get_option(a.cell){
                Some(wall)=>{
                    if wall{                                
                        
                        if let Some(wall) = walls.get_option(a.cell+a.dir_hit.into_vec()){
                            if wall{
                                panic!("dont know how to handle this case")
                            }
                        }
                    
                        return Some(a);
                    }       
                },
                None=>{
                    return None; //We've ray casted off the wall grid.
                }       
            }
        }else{
            return None;
        }
    }
    unreachable!()
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

    let radius = 3.0;
    let diameter=radius*2.0;
    let diameter2=diameter*diameter;

    let mut bots: Vec<_> = UniformRandGen::new(dim.inner_into())
        .take(num_bot)
        .map(|pos| Bot {
            pos,
            vel: vec2same(0.0)
        })
        .collect();

    //bots[0].pos=vec2(30.0,30.0);

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
            /*
            {
                let liquid_radius=10.0;
                let mut tree2:dinotree_alg::collectable::CollectableDinoTree<_,NotNan<_>,_>=dinotree_alg::collectable::CollectableDinoTree::new(&mut bots,|b| {
                    Rect::from_point(b.pos, vec2same(liquid_radius))
                        .inner_try_into()
                        .unwrap()
                });

                tree2.get_mut().find_collisions_mut_par(|a,b|{
                    let offset=b.pos-a.pos;

                    //  |----c----|       |----c----|
                    let distance=offset.magnitude();

                    if distance>0.001 && distance<liquid_radius*2.0{

                        let normal=offset/distance;

                        let ff=0.001*   (liquid_radius-distance)/(liquid_radius*2.0);

                        let velociy_diff = b.vel - a.vel;
                        let damping_ratio = 0.00001;
                        let spring_dampen = velociy_diff.dot(normal) * damping_ratio;
                            
                        a.vel-=normal*(ff-spring_dampen);
                        b.vel+=normal*(ff-spring_dampen);

                        let visc=0.002;
                        a.vel+=velociy_diff*visc;
                        b.vel+=velociy_diff*visc;
                    }
                });
            }
            */
            

            

            let mut tree=dinotree_alg::collectable::CollectableDinoTree::new(&mut bots,|b| {
                Rect::from_point(b.pos, vec2same(radius))
                    .inner_try_into()
                    .unwrap()
            });

            


            
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

           
            

            let bias_factor=0.2;
            let allowed_penetration=radius*0.5;
            let num_iterations=20;
            
            let a1=now.elapsed().as_micros();
        
            let mut collision_list={
                let ka3 = ka.as_ref();
                tree.collect_collisions_list_par(|a,b|{
                    let offset=b.pos-a.pos;
                    let distance2=offset.magnitude2();
                    if distance2>0.00001 && distance2<diameter2{
                        let distance=distance2.sqrt();
                        let offset_normal=offset/distance;
                        let separation=(diameter-distance)/2.0;
                        let bias=-bias_factor*(1.0/num_iterations as f32)*( (-separation+allowed_penetration).min(0.0));
                        
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
            #[derive(Debug)]
            struct WallCollision{
                collisions:[Option<(f32,Vec2<f32>,grid::CardDir,f32)>;2],
            }

            let mut wall_collisions={
                let ka3 = ka.as_ref();

                tree.collect_all_par(|rect,a|{
                    let arr=duckduckgeo::grid::collide::is_colliding(&walls,&grid_viewport,rect.as_ref(),radius);
                    let create_collision=|bot:&mut Bot,dir:grid::CardDir,seperation:f32,offset_normal:Vec2<f32>|{
                        let bias=-bias_factor*(1.0/num_iterations as f32)*( (-seperation+allowed_penetration).min(0.0));

                        let impulse=if let Some(&impulse)=ka3.and_then(|(_,j)|j.get(&single_hash(bot,dir))){ //TODO inefficient to check if its none every time
                            let k=offset_normal*impulse;
                            bot.vel+=k;
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


            //integrate forvces
            //for b in tree.get_bots_mut().iter_mut() {
            use rayon::prelude::*;
            tree.get_bots_mut().par_iter_mut().for_each(|b|{
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
            });

            let a2=now.elapsed().as_micros();

        
            
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

                wall_collisions.for_every_par(&mut tree,|bot,wall|{
                    //dbg!(&wall);
                    for k in wall.collisions.iter_mut(){
                        if let &mut Some((bias,offset_normal,_dir,ref mut acc))=k{
                            
                            let impulse=bias-bot.vel.dot(offset_normal);

                            let p0=*acc;
                            *acc=(p0+impulse).max(0.0);
                            let impulse=*acc-p0;

                            bot.vel+=offset_normal*impulse;
                        }
                    }; 
                })
            }
            let a3=now.elapsed().as_micros();   

 
            let (ka2,ka3):(BTreeMap<_,_>,BTreeMap<_,_>)=rayon::join(||{
                collision_list.iter(&tree).map(|(a,b,&(_,_,impulse))|{
                    (BotCollisionHash::new(a,b),impulse)
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

            ka=Some((ka2,ka3));

            let a4=now.elapsed().as_micros();

            //integrate position
            for b in tree.get_bots_mut().iter_mut() {
                b.pos+=b.vel;
            }
            
            counter+=0.001;
            println!("yo= {} {} {} {}",a1,a2-a1,a3-a2,a4-a3);

            {
                let ray_start=cursor.inner_into();//vec2(320.0,420.0);
                use axgeom::Ray;

                struct RayT<'a> {
                    _p:core::marker::PhantomData<&'a f32>,
                    pub radius: f32,
                }

                impl<'a> RayCast for RayT<'a> {
                    type N = F32n;
                    type T = BBox<F32n, &'a mut Bot>;

                    fn compute_distance_to_bot(
                        &self,
                        ray: &Ray<Self::N>,
                        bot: &Self::T,
                    ) -> axgeom::CastResult<Self::N> {
                        ray.inner_into::<f32>()
                            .cast_to_circle(bot.inner().pos, self.radius)
                            .map(|a| NotNan::new(a).unwrap())
                    }
                    fn compute_distance_to_rect(
                        &self,
                        ray: &Ray<Self::N>,
                        rect: &Rect<Self::N>,
                    ) -> axgeom::CastResult<Self::N> {
                        ray.cast_to_rect(rect)
                    }
                }
                let mut ray_cast = canvas.lines(1.0);

                for dir in 0..360i32 {
                    let dir = dir as f32 * (std::f32::consts::PI / 180.0);
                    let x = (dir.cos() ) as f32;
                    let y = (dir.sin() ) as f32;

                    let ray = {
                        let k = vec2(x, y).inner_try_into().unwrap();
                        Ray {
                            point: ray_start.inner_try_into().unwrap(),
                            dir: k,
                        }
                    };


                    let res = tree
                        .get_mut()
                        .raycast_fine_mut(ray, &mut RayT { radius,_p:core::marker::PhantomData }, dim);




                    
                    let dis=match res{
                        RayCastResult::Hit((_,dis))=>{
                            let dis:f32=dis.into_inner();
                            if let Some(c)=cast_ray(&grid_viewport,&walls,ray_start,vec2(dir.cos(),dir.sin()),400.0){
                                if c.tval<dis{
                                    c.tval
                                }else{
                                    dis
                                }
                            }else{
                                dis
                            }

                        },
                        RayCastResult::NoHit=>{
                            if let Some(c)=cast_ray(&grid_viewport,&walls,ray_start,vec2(dir.cos(),dir.sin()),400.0){
                                c.tval
                            }else{
                                400.0
                            }
                        }
                    };
                    
                    /*
                    let dis = match res {
                        RayCastResult::Hit((_, dis)) => dis.into_inner(),
                        RayCastResult::NoHit => 800.0,
                    };
                    */

                    let end = ray.inner_into().point_at_tval(dis);
                    ray_cast.add(ray.point.inner_into().into(), end.into());
                }
                ray_cast.send_and_uniforms(canvas).with_color([0.5, 0.5, 0.5, 1.0]).draw();
            }
        }

        wall_save.uniforms(canvas,grid_viewport.spacing).draw();

        //Draw circles
        let mut circles = canvas.circles();
        for b in bots.iter(){
            circles.add(b.pos.into());
        }
        circles.send_and_uniforms(canvas,diameter-2.0).with_color([1.0, 1.0, 0.0, 0.6]).draw();
        
        
        /*
        let mut lines = canvas.lines(1.0);
        for b in bots.iter(){
            lines.add(b.pos.into(),(b.pos+b.vel*100.0).into());
        }
        lines.send_and_uniforms(canvas).with_color([0.0,1.0,0.2,1.0]).draw();
        */

        //Draw arrow
        let dim:Rect<f32>=dim.inner_into();
        let start=[dim.x.distance()/2.0,dim.y.distance()/2.0];
        let end=[start[0]+counter.cos()*200.0,start[1]+counter.sin()*200.0];
        canvas.arrows(20.0).add(start,end).send_and_uniforms(canvas).with_color([0.5,0.3,1.0,0.8]).draw();        
    })
}


