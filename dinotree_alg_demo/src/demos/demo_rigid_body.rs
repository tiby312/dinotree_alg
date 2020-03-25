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







#[derive(Copy, Clone)]
pub struct Bot {
    pos: Vec2<f32>,
    vel: Vec2<f32>
}
impl super::seq_impulse::VelocitySolvable for Bot{
    #[inline(always)]
    fn pos(&self)->&Vec2<f32>{
        &self.pos
    }
    #[inline(always)]
    fn vel_mut(&mut self)->&mut Vec2<f32>{
        &mut self.vel
    }
}


pub fn make_demo(dim: Rect<F32n>,canvas:&mut SimpleCanvas) -> Demo {
    let num_bot = 3000;

    let radius = 3.0;
    let diameter=radius*2.0;

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


    let mut solver=super::seq_impulse::CollisionVelocitySolver::new();


    Demo::new(move |cursor, canvas, _check_naive| {
        
        
        let mut tree=dinotree_alg::collectable::CollectableDinoTree::new(&mut bots,|b| {
            Rect::from_point(b.pos, vec2same(radius))
                .inner_try_into()
                .unwrap()
        });


        //Reset the position of bots that have an invalid position
        tree.get_mut().for_all_not_in_rect_mut(&dim, |a| {
            let pos=walls.find_closest_empty(grid_viewport.to_grid(a.pos)).unwrap();
            a.pos=grid_viewport.to_world_center(pos);
        });
    

        let vv = vec2same(200.0).inner_try_into().unwrap();
        
        //Apply forces
        tree.get_mut().for_all_in_rect_mut(&axgeom::Rect::from_point(cursor, vv), | b| {
            let offset=b.pos-cursor.inner_into();
            if offset.magnitude()<200.0*0.5{
                let k=offset.normalize_to(0.02);
                b.vel-=k;
            }
        });

        //integrate forces
        use rayon::prelude::*;
        tree.get_bots_mut().par_iter_mut().for_each(|b|{
            let vel=&mut b.vel;
            if vel.is_nan(){
                *vel=vec2same(0.0);
            }

            let mag2=vel.magnitude2();
            let drag_force=mag2*0.005;
            let ff=*vel/mag2.sqrt()*drag_force;
            let a=*vel-ff;
            if !a.is_nan(){
                *vel=a;
            }
            let g=0.01;
            *vel+=vec2(g*counter.cos(),g*counter.sin());
        });

        //Solve velocities from collisions
        solver.solve(radius,&grid_viewport,&walls,&mut tree);
        
        //integrate positions
        for b in tree.get_bots_mut().iter_mut() {
            b.pos+=b.vel;
        }
        
        counter+=0.001;
        
        //raycast::cast_everywhere(&grid_viewport,&walls,radius,dim,cursor.inner_into(),canvas,&mut tree);
        

        wall_save.uniforms(canvas,grid_viewport.spacing).draw();

        //Draw circles
        let mut circles = canvas.circles();
        for b in bots.iter(){
            circles.add(b.pos.into());
        }
        circles.send_and_uniforms(canvas,diameter-2.0).with_color([1.0, 1.0, 0.0, 0.6]).draw();
        

        //Draw arrow
        let dim:Rect<f32>=dim.inner_into();
        let start=[dim.x.distance()/2.0,dim.y.distance()/2.0];
        let end=[start[0]+counter.cos()*200.0,start[1]+counter.sin()*200.0];
        canvas.arrows(20.0).add(start,end).send_and_uniforms(canvas).with_color([0.5,0.3,1.0,0.8]).draw();        
    })
}

/*
mod raycast{
    use duckduckgeo::grid::GridViewPort;
    use duckduckgeo::grid::Grid2D;
    use dinotree_alg::collectable::CollectableDinoTree;
    use super::*;
    type WorldNum=f32;
    use crate::demos::demo_rigid_body::Bot;
    pub fn cast_everywhere<A:Axis>(grid_viewport:&GridViewPort,walls:&Grid2D,radius:f32,dim:Rect<F32n>,cursor:Vec2<f32>,canvas:&mut SimpleCanvas,tree:&mut CollectableDinoTree<A,NotNan<f32>,Bot>){
        let ray_start=cursor;

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
            
            let end = ray.inner_into().point_at_tval(dis);
            ray_cast.add(ray.point.inner_into().into(), end.into());
        }
        ray_cast.send_and_uniforms(canvas).with_color([0.5, 0.5, 0.5, 1.0]).draw();
    }
    fn cast_ray(grid:&grid::GridViewPort,walls:&grid::Grid2D,point:Vec2<WorldNum>,dir:Vec2<WorldNum>,max_tval:WorldNum)->Option<grid::raycast::CollideCellEvent>{

        let ray=axgeom::Ray{point,dir};
        
        let caster= grid::raycast::RayCaster::new(grid,ray);
        

        if let Some(wall)=walls.get_option(grid.to_grid(point)){
            let _grid_mod=grid.to_grid_mod(point);
            if wall{
                return None;
            }
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

}
*/