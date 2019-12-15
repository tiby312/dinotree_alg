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
        mut sys: very_simple_2d::DrawSession,
        _check_naive: bool,
    ) {
        let dim=self.dim.inner_into();
        let radius=3.0;
        let viewport=GridViewPort{spacing:60.0,origin:vec2(0.0,0.0)};

        let mut rects=sys.rects([1.0,0.6,0.6,1.0]);
        for y in 0..100{
            let yy:f32=viewport.origin.y+(y as f32)*viewport.spacing;
    
            let rect=axgeom::Rect::new(dim.x.start,dim.x.end,yy,yy+1.0);
            rects.add(rect);
        }

        for x in 0..100{
            let xx:f32=viewport.origin.x+(x as f32)*viewport.spacing;
    
            let rect=axgeom::Rect::new(xx,xx+1.0,dim.y.start,dim.y.end);
            rects.add(rect);
        }
        rects.draw();
        drop(rects);

        //let point=vec2(300.0,300.0);
        let point=viewport.origin+vec2same(viewport.spacing*5.0);//vec2(310.0,310.0);

        //let pos=vec2(10.0,10.0);
        //let pos=vec2(86.70752,647.98);
        //let vel=vec2(-0.03991765,0.22951305);
            

        let cursor=cursor.inner_into();
        let ray=axgeom::Ray{point,dir:(cursor-point).normalize_to(1.0)};
        //let ray=Ray{point:pos,dir:vel};

        let rect = axgeom::Rect::from_point(ray.point, vec2same(radius));    
        sys.rects([1.0,0.0,0.0,0.5]).add(rect).draw();
        

        let mut rects=sys.rects([1.0,1.0,0.5,0.2]);
        for (count,a) in RayCaster::new(&viewport,ray).enumerate().take(50){
            let point = ray.point+ray.dir*a.tval;

            let cell=a.cell;
            let topstart=viewport.to_world_topleft(cell);

            let kk=(count as f32)*0.8;

            let rect = axgeom::Rect::from_point(point, vec2same(radius));
            rects.add(rect);


            let cell_rect=axgeom::Rect::new(topstart.x,topstart.x+viewport.spacing,topstart.y,topstart.y+viewport.spacing);
            rects.add(cell_rect);

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
            rects.add(rect);


        }
        rects.draw();

    }
}
