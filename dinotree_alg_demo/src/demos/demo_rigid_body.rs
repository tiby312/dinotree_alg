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


mod grid_collide{
    use super::*;
    use duckduckgeo::grid::*;
    use dinotree_alg::Aabb;
    pub fn is_colliding<T:Aabb>(grid:Grid2D,dim:GridViewPort,bot:&T)->bool{
        unimplemented!()
    }


    #[derive(PartialEq,Copy,Clone)]
    pub struct Foo{
        pub dir:CardDir,
        pub mag:f32
    }
    impl Eq for Foo{}
    pub fn collide_with_cell(grid:Grid2D,dim:GridViewPort,bot:&Bot)->Option<Foo>{
        let grid_coord=dim.to_grid(bot.pos);

        let topleft=dim.to_world_topleft(grid_coord);
        let bottomright=dim.to_world_topleft(grid_coord+vec2(1,1));

        let pos=&bot.pos;


        fn foo(dir:CardDir,mag:f32)->Foo{
            Foo{dir,mag}
        }

        use CardDir::*;
        let arr=[foo(U,topleft.y-pos.y),foo(L,topleft.x-pos.x),foo(D,bottomright.y-pos.y),foo(R,bottomright.x-pos.x)];

        let min=arr.iter().min_by(|Foo{mag:a,..},Foo{mag:b,..}|a.partial_cmp(b).unwrap());


        match min{
            Some(foo)=>{
                if  grid.get(grid_coord+foo.dir.into_vec()){
                    let min=arr.iter().filter(|aa|**aa!=*foo).min_by(|Foo{mag:a,..},Foo{mag:b,..}|a.partial_cmp(b).unwrap());
                    
                    min.map(|a|*a)

                }else{
                    Some(*foo)
                }


            },
            None=>{
                None
            }
        }
    }

}


#[derive(Copy, Clone)]
pub struct Bot {
    pos: Vec2<f32>,
    vel: Vec2<f32>, 
}

use std::time::{Instant};


pub fn make_demo(dim: Rect<F32n>) -> Demo {
    let num_bot = 3000;

    let radius = 5.0;
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


            let num_iterations=10;
            let num_iterations_inv=1.0/num_iterations as f32;
            


            let mut collision_list =  tree.create_collision_list_par(|a,b|{
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

                collision_list.for_every_pair_par_mut(|a,b,&mut (offset_normal,bias)|{
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


