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


mod grid_collide{
    use super::*;
    use duckduckgeo::grid::*;
    use dinotree_alg::Aabb;
    pub fn is_colliding<T:Aabb<Num=NotNan<f32>>>(grid:&Grid2D,dim:&GridViewPort,bot:&T,radius:f32,num_iterations_inv:f32)->Option<(f32,Vec2<f32>)>{
        
        let corners=bot.get().get_corners();

        for a in corners.iter(){
            let grid_coord=*a.as_ref();
            if let Some(d)=grid.get_option(dim.to_grid(grid_coord)){
                if d{
                    //grid_coord

                    //return Some(grid_coord);

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
                        let bias=0.5*dis*num_iterations_inv;
        
                        return Some((bias,offset_normal))
                    }





                }
            }
        }
        None
    }


    #[derive(PartialEq,Copy,Clone)]
    pub struct Foo{
        pub dir:CardDir,
        pub dis:f32
    }
    impl Eq for Foo{}
    pub fn collide_with_cell(grid:&Grid2D,dim:&GridViewPort,bot:&mut super::Bot,num_iterations_inv:f32,corner:Vec2<f32>,radius:f32){
        /*
        

       
        */
        

        

    
    }

}


#[derive(Copy, Clone)]
pub struct Bot {
    pos: Vec2<f32>,
    vel: Vec2<f32>, 
}

use std::time::{Instant};


pub fn make_demo(dim: Rect<F32n>,canvas:&mut SimpleCanvas) -> Demo {
    let num_bot = 8000;
    //let num_bot=100;

    let radius = 2.0;
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



    Demo::new(move |cursor, canvas, _check_naive| {
        for _ in 0..4{
            let now = Instant::now();
            let mut k = bbox_helper::create_bbox_mut(&mut bots, |b| {
                Rect::from_point(b.pos, vec2same(radius))
                    .inner_try_into()
                    .unwrap()
            });


            let mut tree = DinoTree::new_par(&mut k);



            let a1=now.elapsed().as_millis();

            {
                let dim2 = dim.inner_into();
                tree.for_all_not_in_rect_mut(&dim, |a| {
    
                    duckduckgeo::collide_with_border(&mut a.pos,&mut a.vel, &dim2, 0.2);
                });
            }

            let vv = vec2same(200.0).inner_try_into().unwrap();
            let cc = cursor.inner_into();
            tree.for_all_in_rect_mut(&axgeom::Rect::from_point(cursor, vv), |b| {
                let offset=b.pos-cursor.inner_into();
                if offset.magnitude()<200.0*0.5{
                    let _ = duckduckgeo::repel_one(b.pos,&mut b.vel, cc, 0.001, 2.0);
                }
            });


            let a2=now.elapsed().as_millis();




            let num_iterations=20;
            let num_iterations_inv=1.0/num_iterations as f32;



            let mut wall_collisions=tree.collect_all(|rect,_|{
                if let Some(corner)=grid_collide::is_colliding(&walls,&grid_viewport,rect,radius,num_iterations_inv){
                   Some(corner)
                }else{
                    None
                }
            });
            let mut collision_list =  tree.collect_collisions_list_par(|a,b|{
                let offset=b.pos-a.pos;
                let distance2=offset.magnitude2();
                if distance2>0.00001 && distance2<diameter2{
                    let distance=distance2.sqrt();
                    let offset_normal=offset/distance;
                    let bias=0.5*(diameter-distance)*num_iterations_inv;
                    Some((offset_normal,bias))
                }else{
                    None
                }
            });



            
            let a3=now.elapsed().as_millis();
                        
            let mag=0.03*num_iterations_inv - 0.01;
                    
            for _ in 0..num_iterations{

                wall_collisions.for_every_par(&mut k,|bot,&mut (bias,offset_normal)|{
                    bot.vel+=offset_normal*((bias+bot.vel.dot(offset_normal)*mag)*0.5); //Unlike bot collision we only affect one bot so half everything.
                });

                collision_list.for_every_pair_par_mut(&mut k,|a,b,&mut (offset_normal,bias)|{
                    let vel=b.vel-a.vel;
                    let k=offset_normal*(bias+vel.dot(offset_normal)*mag);
                    a.vel-=k;
                    b.vel+=k;
                });     
            }

            let a4=now.elapsed().as_millis();


            counter+=0.001;
            for b in bots.iter_mut() {
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
                b.pos+=b.vel;
                
            }
            println!("yo= {} {} {} {}",a1,a2-a1,a3-a2,a4-a3);
        }

       

        wall_save.uniforms(canvas,grid_viewport.spacing).draw();


        //Draw circles
        let mut circles = canvas.circles();
        for b in bots.iter(){
            circles.add(b.pos.into());
        }
        circles.send_and_uniforms(canvas,diameter).with_color([1.0, 1.0, 0.0, 0.6]).draw();
        
        //Draw arrow
        let dim:Rect<f32>=dim.inner_into();
        let start=[dim.x.distance()/2.0,dim.y.distance()/2.0];
        let end=[start[0]+counter.cos()*200.0,start[1]+counter.sin()*200.0];
        canvas.arrows(20.0).add(start,end).send_and_uniforms(canvas).with_color([0.5,0.3,1.0,0.8]).draw();
        

        
    })
}


