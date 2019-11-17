use crate::support::prelude::*;

use duckduckgeo::grid::*;
use duckduckgeo::grid::raycast::*;
    


pub struct RaycastGridDemo {
    dim: Rect<F32n>,
}
impl RaycastGridDemo {
    pub fn new(dim: Rect<F32n>) -> RaycastGridDemo {

        RaycastGridDemo {
            dim,
        }
    }
}

impl DemoSys for RaycastGridDemo {
    fn step(
        &mut self,
        cursor: Vec2<F32n>,
        c: &piston_window::Context,
        g: &mut piston_window::G2d,
        _check_naive: bool,
    ) {
        let dim=self.dim.inner_into();
        let cursor=cursor.inner_into();
        let radius=3.0;
        let viewport=GridViewPort{spacing:vec2(25.0,25.0),origin:vec2(0.0,0.0)};

        for y in 0..100{
            let yy:f32=viewport.origin.y+(y as f32)*viewport.spacing.y;
    
            let rect=axgeom::Rect::new(dim.x.left,dim.x.right,yy,yy+1.0);
            draw_rect_f32([1.0, 0.6, 0.6, 1.0], &rect, c, g);
        
        }

        for x in 0..100{
            let xx:f32=viewport.origin.x+(x as f32)*viewport.spacing.x;
    
            let rect=axgeom::Rect::new(xx,xx+1.0,dim.y.left,dim.y.right);
            draw_rect_f32([0.6, 1.0, 0.6, 1.0], &rect, c, g);
        
        }

        //let point=vec2(300.0,300.0);
        let point=vec2(310.0,310.0);

        let ray=Ray{point,dir:(cursor-point).normalize_to(1.0)};

        let rect = &axgeom::Rect::from_point(ray.point, vec2same(radius));    
        draw_rect_f32([1.0, 0.0, 0.0, 0.5], rect, c, g);
        

        for (count,a) in RayCaster::new(&viewport,ray).unwrap().enumerate().take(50){
            let point = ray.point+ray.dir*a.tval;

            let cell=a.cell;
            let topleft=viewport.to_world_topleft(cell);

            let kk=(count as f32)*0.1;

            let rect = &axgeom::Rect::from_point(point, vec2same(radius));
            draw_rect_f32([0.0, 0.0, kk, 0.5], rect, c, g);


            let cell_rect=axgeom::Rect::new(topleft.x,topleft.x+viewport.spacing.x,topleft.y,topleft.y+viewport.spacing.y);
            draw_rect_f32([0.0, 0.0, kk, 0.5], &cell_rect, c, g);

            use CardDir::*;
            let l=3.0;
            let r=10.0;
            let arr=match a.dir_hit{
                L=>{
                    [point.x-r,point.x,point.y,point.y+l]
                },
                R=>{
                    [point.x,point.x+r,point.y,point.y+l]
                },
                U=>{
                    [point.x,point.x+l,point.y-r,point.y]
                },
                D=>{
                    [point.x,point.x+l,point.y,point.y+r]
                },
            };

            let rect=axgeom::Rect::new(arr[0],arr[1],arr[2],arr[3]);
            draw_rect_f32([0.0, 0.0, 0.0, 0.5], &rect, c, g);


        }


    }
}
