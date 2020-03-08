use crate::support::prelude::*;

use duckduckgeo;

/*
    sequential impulse solver
    http://myselph.de/gamePhysics/equalityConstraints.html
 Constraint Solver

    As mentioned during the overvie, modern physic engines seem to use mostly iterative solvers that work as follows (pseudocode): 

for i = 1 to nIterations
    for c in constraints
        P = computeImpulses(c);
        c.bodies.applyImpulses(P);

*/


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
    fn new(a:usize,b:usize)->BotCollisionHash{                
        let [a,b]=if a<b{
            [a,b]
        }else{
            [b,a]
        };
        BotCollisionHash(a,b)
    }
}




mod grid_collide{
    use super::*;
    use duckduckgeo::grid::*;
    use dinotree_alg::Aabb;

    fn find_corner_offset(grid:&Grid2D,dim:&GridViewPort,radius:f32,corner:&Vec2<f32>)->Option<(f32,Vec2<f32>)>{
        let grid_coord:Vec2<f32>=corner.inner_into();
        if let Some(d)=grid.get_option(dim.to_grid(grid_coord)){
            if d{
                #[derive(PartialEq,Copy,Clone)]
                pub struct Foo{
                    pub dir:CardDir,
                    pub dis:f32
                }
                impl Eq for Foo{}
                
                let corner=grid_coord;

                fn foo(dir:CardDir,dis:f32)->Foo{
                    Foo{dir,dis}
                }


                let cu=corner.y-radius;
                let cl=corner.x-radius;

                let cd=corner.y+radius;
                let cr=corner.x+radius;
                let grid_coord=dim.to_grid(corner);
                let topleft=dim.to_world_topleft(grid_coord);
                let bottomright=dim.to_world_topleft(grid_coord+vec2(1,1));
                use CardDir::*;
                let arr=[foo(U,cd-topleft.y),foo(L,cr-topleft.x),foo(D,bottomright.y-cu),foo(R,bottomright.x-cl)];

                let min=arr.iter().filter(|a|a.dis>0.0).min_by(|a,b|a.dis.partial_cmp(&b.dis).unwrap());

                let min = match min{
                    Some(foo)=>{
                        if let Some(wall)=grid.get_option(grid_coord+foo.dir.into_vec()){                    
                            if wall{
                                let min=arr.iter().filter(|a|a.dis>0.0).filter(|aa|**aa!=*foo).min_by(|a,b|a.dis.partial_cmp(&b.dis).unwrap());
                                min.map(|a|*a)
                            }else{
                                Some(*foo)
                            }
                        }else{
                            Some(*foo)
                        }
                    },
                    None=>{
                        None
                    }
                };

                if let Some(Foo{dir,dis})=min{
               
                    let offset_normal=match dir{
                        U=>{
                            vec2(0.0,-1.0)
                        },
                        D=>{
                            vec2(0.0,1.0)
                        },
                        L=>{
                            vec2(-1.0,0.0)
                        },
                        R=>{
                            vec2(1.0,0.0)
                        }
                    };
    
                    return Some((dis,offset_normal))
                }
            }
        }
        return None;
    }
    pub fn is_colliding<T:Aabb<Num=NotNan<f32>>>(grid:&Grid2D,dim:&GridViewPort,bot:&T,radius:f32)->[Option<(f32,Vec2<f32>)>;2]{
        
        let corners=bot.get().get_corners();

        let mut offsets:Vec<_>=corners.iter().map(|a|{
            find_corner_offset(grid,dim,radius,a.as_ref())
        }).collect();

        let mut offsets:Vec<_>=offsets.drain(..).filter(|a|a.is_some()).map(|a|a.unwrap()).collect();
        offsets.sort_by(|(a,_),(b,_)|a.partial_cmp(b).unwrap());


        let min=offsets.iter().min_by(|&(a,_),&(b,_)|a.partial_cmp(b).unwrap());


        match min{
            Some(&min)=>{
                let second_min=offsets.iter().filter(|(_,a)|a!=&min.1).min_by(|&(a,_),&(b,_)|a.partial_cmp(b).unwrap());
                match second_min{
                    Some(&second_min)=>{
                        [Some(min),Some(second_min)]
                    },
                    None=>{
                        [Some(min),None]
                    }
                }
            },
            None=>{
                [None,None]
            }
        }
    }
}


#[derive(Copy, Clone)]
pub struct Bot {
    pos: Vec2<f32>,
    vel: Vec2<f32>, 
    pseudo_vel:Vec2<f32>
}

use std::time::{Instant};


pub fn make_demo(dim: Rect<F32n>,canvas:&mut SimpleCanvas) -> Demo {
    let num_bot = 1000;
    //let num_bot=100;

    let radius = 6.0;
    let diameter=radius*2.0;
    let diameter2=diameter*diameter;

    let mut bots: Vec<_> = UniformRandGen::new(dim.inner_into())
        .take(num_bot)
        .map(|pos| Bot {
            pseudo_vel:vec2same(0.0),
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


    struct Converter<T>(*mut T);
    unsafe impl<T> Send for Converter<T>{}
    unsafe impl<T> Sync for Converter<T>{}

    impl<T> Copy for Converter<T>{}
    impl<T> Clone for Converter<T>{
        #[inline(always)]
        fn clone(&self)->Self{
            Converter(self.0)
        }
    }
    impl<T> Converter<T>{
        fn new(a:&mut [T])->Converter<T>{
            Converter(a.as_mut_ptr())
        }
        #[inline(always)]
        unsafe fn index_mut(&self,index:usize)->&mut T{
            &mut *self.0.offset(index as isize)
        }
    }

    Demo::new(move |cursor, canvas, _check_naive| {
        for _ in 0..4{
            let now = Instant::now();
            


            let mut k:Vec<_>=bots.iter().enumerate().map(|(i,a)|{
                bbox::BBox::new(Rect::from_point(a.pos,vec2same(radius)).inner_try_into().unwrap(),i)
            }).collect();


            let mut tree = DinoTree::new_par(&mut k);



            let a1=now.elapsed().as_millis();

            
            tree.for_all_not_in_rect_mut(&dim, |&mut a| {
                let a=&mut bots[a];
                duckduckgeo::collide_with_border(&mut a.pos,&mut a.vel, dim.as_ref(), 0.2);
            });
        

            let vv = vec2same(200.0).inner_try_into().unwrap();
            
            tree.for_all_in_rect_mut(&axgeom::Rect::from_point(cursor, vv), |&mut b| {
                let b=&mut bots[b];
                let offset=b.pos-cursor.inner_into();
                if offset.magnitude()<200.0*0.5{
                    let k=offset.normalize_to(0.02);
                    b.vel-=k;
                }
            });

           
            let a2=now.elapsed().as_millis();

            //let bias_factor=0.00003;
            let bias_factor=0.0;
            let allowed_penetration=radius;
            let num_iterations=20;//14;
            //let num_iterations_inv=1.0/num_iterations as f32;
            

            

            let mut collision_list={
                let ka3 = ka.as_ref();
                let c=Converter::new(&mut bots);
                tree.collect_collisions_list_par(|aa,bb|{
                    let a=unsafe{c.index_mut(aa.inner)};
                    let b=unsafe{c.index_mut(bb.inner)};
                    let offset=b.pos-a.pos;
                    let distance2=offset.magnitude2();
                    if distance2>0.00001 && distance2<diameter2{
                        let distance=distance2.sqrt();
                        let offset_normal=offset/distance;
                        
                        let separation=diameter-distance;
                        let bias=bias_factor*(1.0/num_iterations as f32)*( (separation-allowed_penetration).max(0.0));
                        let hash=BotCollisionHash::new(aa.inner,bb.inner);
                        let impulse=if let Some(&impulse)=ka3.and_then(|(j,_)|j.get(&hash)){ //TODO inefficient to check if its none every time
                            let k=offset_normal*impulse;
                            a.vel-=k;
                            b.vel+=k;
                            impulse
                        }else{
                            0.0
                        };

                        Some((offset_normal,separation,impulse))
                    }else{
                        None
                    }
                })
            };



            let mut wall_collisions={
                let ka3 = ka.as_ref();

                let mut wall_collisions=Vec::new();
                for e in tree.get_bots().iter(){
                    let a=&mut bots[e.inner];

                    let arr=grid_collide::is_colliding(&walls,&grid_viewport,e.get(),radius);

                    for (seperation,offset_normal) in arr.iter().filter(|a|a.is_some()).map(|a|a.unwrap()){
                        //let seperation=seperation/2.0; //TODO why necessary
                        //dbg!(seperation);
                        let bias=bias_factor*(1.0/num_iterations as f32)*( (seperation+allowed_penetration).max(0.0));

                        let impulse=if let Some(&impulse)=ka3.and_then(|(_,j)|j.get(&e.inner)){ //TODO inefficient to check if its none every time
                            let k=offset_normal*impulse;
                            a.vel+=k;
                            impulse
                        }else{
                            0.0
                        };

                        wall_collisions.push((e.inner,seperation,offset_normal,impulse));
                        
                    }
                };
                wall_collisions
            };

            //integrate forvces
            for b in bots.iter_mut() {
                //let b=&mut bots[b.inner];
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
                b.vel+=vec2(0.01*counter.cos(),0.01*counter.sin());
             }


            
            let a3=now.elapsed().as_millis();
                        
            let mag=0.1*(1.0/num_iterations as f32) - 0.01;
                    
            for _ in 0..num_iterations{

                let c=Converter::new(&mut bots);
                
                collision_list.for_every_pair_par_mut(move |a,b,&mut (offset_normal,_,ref mut acc)|{
                    let a=unsafe{c.index_mut(a.inner)};
                    let b=unsafe{c.index_mut(b.inner)};
                    let vel=b.vel-a.vel;
                    let impulse=/*bias+*/vel.dot(offset_normal)*mag;
                    
                    let p0=*acc;
                    *acc=(p0+impulse).max(0.0);
                    let impulse=*acc-p0;
                    
                    let k=offset_normal*impulse;
                    a.vel-=k;
                    b.vel+=k;
                });     

                for &mut (e,_,offset_normal,ref mut acc) in wall_collisions.iter_mut(){
                    let bot=&mut bots[e];

                    let impulse=/*bias+*/bot.vel.dot(offset_normal)*mag;

                    let p0=*acc;
                    *acc=(p0+impulse).max(0.0);
                    let impulse=*acc-p0;

                    bot.vel+=offset_normal*impulse;
                }
            }


            let mut ka2=BTreeMap::new();
            collision_list.for_every_pair_mut(|a,b,&mut (_,_,impulse)|{
                let hash=BotCollisionHash::new(a.inner,b.inner);
                ka2.insert(hash,impulse);
            });
            let mut ka3=BTreeMap::new();

            for &(e,_,_,impulse) in wall_collisions.iter(){
                ka3.insert(e,impulse);
            }
            ka=Some((ka2,ka3));

            let a4=now.elapsed().as_millis();
            



            let num_iterations=5;//2;

            let c=Converter::new(&mut bots);
            
            let pseudo_vel_constant=0.03*(1.0/num_iterations as f32);
            let mag=0.7*(1.0/num_iterations as f32);
            for _ in 0..num_iterations{

                
                collision_list.for_every_pair_par_mut(move |a,b,&mut (offset_normal,seperation,_)|{
                    let a=unsafe{c.index_mut(a.inner)};
                    let b=unsafe{c.index_mut(b.inner)};
                    let pseudo_vel=b.pseudo_vel-a.pseudo_vel;
                    let pseudo_impulse=pseudo_vel.dot(offset_normal)*mag+pseudo_vel_constant*seperation;
                    let pseudo_impulse=pseudo_impulse.max(0.0);
                    let k=offset_normal*pseudo_impulse;
                    assert!(!k.is_nan(),"yooo={:?}",(pseudo_impulse,pseudo_vel,offset_normal,seperation));
                    if !k.is_nan(){
                        a.pseudo_vel-=k;
                        b.pseudo_vel+=k;
                    }
                });

                for &mut (e,seperation,offset_normal,_) in wall_collisions.iter_mut(){
                    let bot=&mut bots[e];

                    let pseudo_impulse=bot.pseudo_vel.dot(offset_normal)*mag+pseudo_vel_constant*seperation;
                    let pseudo_impulse=pseudo_impulse.max(0.0);
                    let k=offset_normal*pseudo_impulse;
                    assert!(!k.is_nan(),"yooo={:?}",(bot.pseudo_vel,pseudo_impulse,offset_normal,seperation));
                    if !k.is_nan(){
                        bot.pseudo_vel+=k;
                    }
                }
            }


            





            
            //integrate position
            for b in bots.iter_mut() {
                b.pos+=b.vel;
                b.pos+=b.pseudo_vel;
                b.pseudo_vel=vec2same(0.0);
                
            }
            
            counter+=0.001;
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


