use crate::support::prelude::*;

use duckduckgeo;
use std::sync::atomic::AtomicPtr;
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




use test::CollectableDinoTree;
mod test{
    use super::*;
    pub struct CollectableDinoTree<'a,A:Axis,N:Num,T>{
        bots:&'a mut [T],
        tree:DinoTreeOwned<A,BBoxPtr<N,T>>
    }
    impl<'a,N:Num,T> CollectableDinoTree<'a,DefaultA,N,T>{
        pub fn new(bots:&'a mut [T],mut func:impl FnMut(&mut T)->Rect<N>)->CollectableDinoTree<'a,DefaultA,N,T>{
            let  bboxes:Vec<_>=bots.iter_mut().map(|a|BBoxPtr::new(func(a),unsafe{std::ptr::NonNull::new_unchecked(a as *mut _)})).collect();

            let tree=DinoTreeOwned::new(bboxes);

            CollectableDinoTree{bots,tree}
        }
    }
    impl<'a,A:Axis,N:Num,T> CollectableDinoTree<'a,A,N,T>{

        pub fn get_bots_mut(&mut self)->&mut [T]{
            self.bots
        }

        pub fn get_tree_mut(&mut self)->&mut DinoTree<A,NodePtr<BBoxPtr<N,T>>>{
            self.tree.as_tree_mut()
        }

        pub fn collect_all<D>(&mut self,mut func:impl FnMut(&Rect<N>,&mut T)->Option<D>)->SingleCollisionList<'a,T,D>{
             
            let a=self.tree.as_tree_mut().collect_all(|a,b|{
                match func(a,b){
                    Some(d)=>{
                        Some((b as *mut _,d))
                    },
                    None=>{
                        None
                    }
                }
            });
            SingleCollisionList{_p:PhantomData,a}
        }
    }


    #[derive(Copy,Clone)]
    pub struct Collision<T>{
        pub a:*mut T,
        pub b:*mut T,
    }
    unsafe impl<T> Send for Collision<T>{}
    unsafe impl<T> Sync for Collision<T>{}

    impl<'a,A:Axis+Send+Sync,N:Num+Send+Sync,T:Send+Sync> CollectableDinoTree<'a,A,N,T>{

        pub fn collect_collisions_list_par <D:Send+Sync>(&mut self,func:impl Fn(&mut T,&mut T)->Option<D>+Send+Sync+Copy)->BotCollision<'a,T,D>{
        
            let cols=self.tree.as_tree_mut().collect_collisions_list_par(|a,b|{
                match func(a,b){
                    Some(d)=>{
                        Some((Collision{a,b:b},d))
                    },
                    None=>{
                        None
                    }
                }
            });
            BotCollision{cols,_p:PhantomData}
        }
    }

    use core::marker::PhantomData;
    pub struct SingleCollisionList<'a,T,D>{
        _p:PhantomData<&'a mut T>,
        a:Vec<(*mut T,D)>
    }
    impl<'a,T,D> SingleCollisionList<'a,T,D>{
        pub fn for_every<'b,A:Axis,N:Num>(&'b mut self,_:&'b mut CollectableDinoTree<'a,A,N,T>,mut func:impl FnMut(&mut T,&mut D)){
            for (a,d) in self.a.iter_mut(){
                func(unsafe{&mut **a},d)
            }
        }
    }

    pub struct BotCollision<'a,T,D>{
        _p:PhantomData<&'a mut T>,
        cols:CollisionList<(Collision<T>,D)>
    }

    impl<'a,T,D> BotCollision<'a,T,D>{
        pub fn for_every_pair<'b,A:Axis,N:Num>(&'b mut self,_:&'b mut CollectableDinoTree<'a,A,N,T>,mut func:impl FnMut(&mut T,&mut T,&mut D)){
            
            self.cols.for_every_pair_mut(|(Collision{a,b},d)|{
                let a=unsafe{&mut **a};
                let b=unsafe{&mut **b};
                func(a,b,d)
            })
        }
    }
    impl<'a,T:Send+Sync,D:Send+Sync> BotCollision<'a,T,D>{
        pub fn for_every_pair_par<'b,A:Axis,N:Num>(&'b mut self,_:&'b mut CollectableDinoTree<'a,A,N,T>,func:impl Fn(&mut T,&mut T,&mut D)+Send+Sync+Copy){
            
            self.cols.for_every_pair_par_mut(|(Collision{a,b},d)|{
                let a=unsafe{&mut **a};
                let b=unsafe{&mut **b};
                func(a,b,d)
            })
        }
    }
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
    vel: Vec2<f32>
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
            

            let mut tree=CollectableDinoTree::new(&mut bots,|b| {
                Rect::from_point(b.pos, vec2same(radius))
                    .inner_try_into()
                    .unwrap()
            });

            let a1=now.elapsed().as_millis();

            
            tree.get_tree_mut().for_all_not_in_rect_mut(&dim, |a| {
                duckduckgeo::collide_with_border(&mut a.pos,&mut a.vel, dim.as_ref(), 0.2);
            });
        

            let vv = vec2same(200.0).inner_try_into().unwrap();
            
            tree.get_tree_mut().for_all_in_rect_mut(&axgeom::Rect::from_point(cursor, vv), | b| {
                let offset=b.pos-cursor.inner_into();
                if offset.magnitude()<200.0*0.5{
                    let k=offset.normalize_to(0.02);
                    b.vel-=k;
                }
            });

           
            let a2=now.elapsed().as_millis();

            let bias_factor=0.0002;
            let allowed_penetration=radius;
            let num_iterations=12;
            

            

            let mut collision_list={
                let ka3 = ka.as_ref();
                tree.collect_collisions_list_par(|a,b|{
                    let offset=b.pos-a.pos;
                    let distance2=offset.magnitude2();
                    if distance2>0.00001 && distance2<diameter2{
                        let distance=distance2.sqrt();
                        let offset_normal=offset/distance;
                        
                        let separation=diameter-distance;
                        let bias=bias_factor*(1.0/num_iterations as f32)*( (separation-allowed_penetration).max(0.0));
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
                    let arr=grid_collide::is_colliding(&walls,&grid_viewport,rect,radius);
                    let create_collision=|bot:&mut Bot,seperation:f32,offset_normal:Vec2<f32>|{
                        let bias=bias_factor*(1.0/num_iterations as f32)*( (seperation+allowed_penetration).max(0.0));

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
                        
            let mag=0.01*(1.0/num_iterations as f32) - 0.01;
                    
            for _ in 0..num_iterations{


                collision_list.for_every_pair_par(&mut tree,|a,b,&mut (offset_normal,bias,ref mut acc)|{
                    
                    let vel=b.vel-a.vel;
                    let impulse=bias+vel.dot(offset_normal)*mag;
                    
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
                            
                            let impulse=bias+bot.vel.dot(offset_normal)*mag;

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


