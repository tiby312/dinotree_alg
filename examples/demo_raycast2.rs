extern crate piston_window;
extern crate axgeom;
extern crate num;
extern crate rand;
extern crate dinotree;

extern crate ordered_float;
use piston_window::*;

mod support;
use dinotree::*;
use dinotree::support::*;
use support::*;
use ordered_float::*;


fn intersects_box(point:[NotNaN<f64>;2],dir:[NotNaN<f64>;2],rect:&AABBox<NotNaN<f64>>)->Option<NotNaN<f64>>{
    let ((x1,x2),(y1,y2))=rect.get();

    let x1=x1.into_inner();
    let x2=x2.into_inner();
    let y1=y1.into_inner();
    let y2=y2.into_inner();

    let mut tmin=std::f64::MIN;
    let mut tmax=std::f64::MAX;

    let point=[point[0].into_inner(),point[1].into_inner()];
    let dir=[dir[0].into_inner(),dir[1].into_inner()];

    if dir[0]!=0.0{
        let tx1=(x1-point[0])/dir[0];
        let tx2=(x2-point[0])/dir[0];

        tmin=tmin.max(tx1.min(tx2));
        tmax=tmax.min(tx1.max(tx2));
        
    }else{
        if point[0] < x1 || point[0] > x2 {
            return None; // parallel AND outside box : no intersection possible
        }
    }

    if dir[1]!=0.0{
        let ty1=(y1-point[1])/dir[1];
        let ty2=(y2-point[1])/dir[1];

        tmin=tmin.max(ty1.min(ty2));
        tmax=tmax.min(ty1.max(ty2));
    }else{
        if point[1] < y1 || point[1] > y2 {
            return None; // parallel AND outside box : no intersection possible
        }
    }
    if tmax>=tmin && tmax>=0.0{
        return Some(NotNaN::new(tmin.max(0.0)).unwrap());
    }else{
        return None;
    }                
}



fn main() {

    let mut bots=create_bots_f64(|id|Bot{id,col:Vec::new()},&[0,800,0,800],500,[2,20]);


    let mut window: PistonWindow = WindowSettings::new("dinotree test", [800, 800])
        .exit_on_esc(true)
        .samples(4)
        .build()
        .unwrap();

    let mut cursor=[0.0,0.0];
    while let Some(e) = window.next() {
        e.mouse_cursor(|x, y| {
            cursor = [x, y];
        });

        window.draw_2d(&e, |c, g| {
            clear([0.0; 4], g);

            for bot in bots.iter(){
                let ((x1,x2),(y1,y2))=bot.rect.get();
                let ((x1,x2),(y1,y2))=((x1.into_inner(),x2.into_inner()),(y1.into_inner(),y2.into_inner()));
                    
                let square = [x1,y1,x2-x1,y2-y1];
                rectangle([1.0,1.0,1.0,0.3], square, c.transform, g);
            }
        
            {
                let mut tree = DinoTree::new(&mut bots, StartAxis::Xaxis);

                let bb=AABBox::new((NotNaN::new(0.0).unwrap(),NotNaN::new(800.0).unwrap()),(NotNaN::new(0.0).unwrap(),NotNaN::new(800.0).unwrap()));
                
                for i in 0..360{
                    let i=i as f64*(std::f64::consts::PI/180.0);
                    let x=(i.cos()*20.0) as f64 ;
                    let y=(i.sin()*20.0) as f64;

                    let ray_point=[cursor[0] as f64,cursor[1] as f64];
                    let ray_dir=[x,y];

                    let ray_point=[NotNaN::new(ray_point[0]).unwrap(),NotNaN::new(ray_point[1]).unwrap()];
                    let ray_dir=[NotNaN::new(ray_dir[0]).unwrap(),NotNaN::new(ray_dir[1]).unwrap()];


                    let fast_func=|rect:&AABBox<NotNaN<f64>>|->Option<NotNaN<f64>>{
                        let ((x1,x2),(y1,y2))=rect.get();
  
                        intersects_box(ray_point,ray_dir,rect)
                    };


                    let ray_touch_box=|a:ColSingle<BBox<NotNaN<f64>,Bot>>|->Option<NotNaN<f64>>{
                        let ((x1,x2),(y1,y2))=a.rect.get();
                        intersects_box(ray_point,ray_dir,a.rect)
                    };

                    
                    let k=tree.raycast(ray_point,ray_dir,bb,fast_func,ray_touch_box);

                    let (ppx,ppy)=if let Some(k)=k{
                        let ppx=ray_point[0]+ray_dir[0]*k.1;
                        let ppy=ray_point[1]+ray_dir[1]*k.1;
                        (ppx,ppy)
                    }else{
                        let ppx=ray_point[0]+ray_dir[0]*800.0;
                        let ppy=ray_point[1]+ray_dir[1]*800.0;
                        (ppx,ppy)
                    };

                    let arr=[ray_point[0].into_inner() ,ray_point[1].into_inner() ,ppx.into_inner() ,ppy.into_inner() ];
                    line([1.0, 1.0, 1.0, 0.2], // black
                         1.0, // radius of line
                         arr, // [x0, y0, x1,y1] coordinates of line
                         c.transform,
                         g);
                }
            }
        });
    }
}
