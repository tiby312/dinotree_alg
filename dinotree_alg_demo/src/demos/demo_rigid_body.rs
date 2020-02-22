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


            let num_iterations=16;
            let num_iterations_inv=1.0/num_iterations as f32;
            

            mod foo{
                use super::*;
                use compt::Visitor;
                    
                fn parallelize<T:Visitor+Send+Sync>(a:T,func:impl Fn(T::Item)+Sync+Send+Copy) where T::Item:Send+Sync{
                    let (n,l)=a.next();
                    func(n);
                    if let Some([left,right])=l{
                        rayon::join(||parallelize(left,func),||parallelize(right,func));
                    }
                }
                pub struct CollisionList<T,D>{
                    nodes:Vec<Vec<Collision<T,D>>>
                }
                impl<T:Send+Sync,D:Send+Sync> CollisionList<T,D>{
                    pub fn for_every_pair_seq_mut(&mut self,mut func:impl FnMut(&mut T,&mut T,&mut D)+Send+Sync+Copy){
                        for a in self.nodes.iter_mut(){
                            for c in a.iter_mut(){
                                let a=unsafe{&mut *c.a};
                                let b=unsafe{&mut *c.b};
                                func(a,b,&mut c.d)
                            }
                        }
                    }
                    pub fn for_every_pair_par_mut(&mut self,func:impl Fn(&mut T,&mut T,&mut D)+Send+Sync+Copy){
                        /*
                        for a in self.nodes.iter(){
                            print!("{},",a.len());
                        }
                        println!();
                        */
                        let mtree=compt::dfs_order::CompleteTree::from_preorder_mut(&mut self.nodes).unwrap();

                        parallelize(mtree.vistr_mut(),|a|{
                            for c in a.iter_mut(){
                                let a=unsafe{&mut *c.a};
                                let b=unsafe{&mut *c.b};
                                func(a,b,&mut c.d)
                            }
                        })
                    }
                }

                pub fn create_collision_list<A:Axis,T:Aabb+Send+Sync,D>
                (tree:&mut DinoTree<A,NodeMut<T>>,func:impl Fn(&mut T::Inner,&mut T::Inner)->Option<D>+Send+Sync)->CollisionList<T::Inner,D>
                where T:HasInner+Send+Sync{

                    struct Foo<T:Visitor>{
                        current:T::Item,
                        next:Option<[T;2]>,
                    }
                    impl<T:Visitor> Foo<T>{
                        fn new(a:T)->Foo<T>{
                            let (n,f)=a.next();
                            Foo{current:n,next:f}
                        }
                    }

                    let height=1+dinotree_alg::par::compute_level_switch_sequential(par::SWITCH_SEQUENTIAL_DEFAULT,tree.get_height()).get_depth_to_switch_at();
                    //dbg!(tree.get_height(),height);
                    let mut nodes:Vec<Vec<Collision<T::Inner,D>>>=(0..compt::compute_num_nodes(height)).map(|_|Vec::new()).collect();
                    let mtree=compt::dfs_order::CompleteTree::from_preorder_mut(&mut nodes).unwrap();
                    
                    tree.find_collisions_mut_par_ext(|a|{
                        let next=a.next.take();
                        if let Some([left,right])=next{
                            let l=Foo::new(left);
                            let r=Foo::new(right);
                            *a=l;
                            r
                        }else{
                            unreachable!()
                        }
                    },|_a,_b|{},|c,a,b|{
                        if let Some(d)=func(a,b){
                            c.current.push(Collision::new(a,b,d));
                        }
                    },Foo::new(mtree.vistr_mut()));

                    CollisionList{nodes}
                }

                struct Collision<T,D>{
                    a:*mut T,
                    b:*mut T,
                    d:D
                }
                impl<T,D> Collision<T,D>{
                    fn new(a:&mut T,b:&mut T,d:D)->Self{
                        Collision{a:a as *mut _,b:b as *mut _,d}
                    }
                }
                unsafe impl<T,D> Send for Collision<T,D>{}
                unsafe impl<T,D> Sync for Collision<T,D>{}


            }

            let mut collision_list =  foo::create_collision_list(&mut tree,|a,b|{
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


