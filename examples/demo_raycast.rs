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



fn intersects_box(point:[isize;2],dir:[isize;2],rect:&AABBox<isize>)->Option<isize>{
    let ((x1,x2),(y1,y2))=rect.get();


    let mut tmin=isize::min_value();
    let mut tmax=isize::max_value();

    if dir[0]!=0{
        let tx1=(x1-point[0])/dir[0];
        let tx2=(x2-point[0])/dir[0];

        tmin=tmin.max(tx1.min(tx2));
        tmax=tmax.min(tx1.max(tx2));
        
    }else{
        if point[0] < x1 || point[0] > x2 {
            return None; // parallel AND outside box : no intersection possible
        }
    }
    if dir[1]!=0{
        let ty1=(y1-point[1])/dir[1];
        let ty2=(y2-point[1])/dir[1];

        tmin=tmin.max(ty1.min(ty2));
        tmax=tmax.min(ty1.max(ty2));
    }else{
        if point[1] < y1 || point[1] > y2 {
            return None; // parallel AND outside box : no intersection possible
        }
    }
    if tmax>=tmin && tmax>=0{
        return Some(tmin.max(0));
    }else{
        return None;
    }
                
}

fn main() {

    let mut bots=create_bots_isize(|id|Bot{id,col:Vec::new()},&[0,800,0,800],500,[2,20]);


    let mut window: PistonWindow = WindowSettings::new("dinotree test", [800, 800])
        .exit_on_esc(true)
        .build()
        .unwrap();

    let mut cursor=[0.0,0.0];
    while let Some(e) = window.next() {
        e.mouse_cursor(|x, y| {
            cursor = [x, y];
        });

        window.draw_2d(&e, |c, g| {
            clear([1.0; 4], g);

            let ray_point=[cursor[0] as isize,cursor[1] as isize];
            let ray_dir=[-1,-2];           

            for bot in bots.iter(){
                let ((x1,x2),(y1,y2))=bot.rect.get();
                let ((x1,x2),(y1,y2))=((x1 as f64,x2 as f64),(y1 as f64,y2 as f64));
                    
                let square = [x1,y1,x2-x1,y2-y1];
                rectangle([0.0,0.0,0.0,0.3], square, c.transform, g);
            }
        
        
            {
                let mut tree = DinoTree::new(&mut bots, StartAxis::Xaxis);


                let k={
                    let bb=AABBox::new((0+100,800-100),(0+100,800-100));
                    {
                        let ((x1,x2),(y1,y2))=bb.get();//(bb.xdiv,bb.ydiv);
                        let ((x1,x2),(y1,y2))=((x1 as f64,x2 as f64),(y1 as f64,y2 as f64));
                        let square = [x1,y1,x2-x1,y2-y1];
                        rectangle([0.0,1.0,0.0,0.2], square, c.transform, g);
                    }



                    let fast_func=|rect:&AABBox<isize>|->Option<isize>{
                        let ((x1,x2),(y1,y2))=rect.get();

                        intersects_box(ray_point,ray_dir,rect)
                    };


                    let ray_touch_box=|a:ColSingle<BBox<isize,Bot>>|->Option<isize>{
                        let ((x1,x2),(y1,y2))=a.rect.get();
                        
                        {
                            let ((x1,x2),(y1,y2))=((x1 as f64,x2 as f64),(y1 as f64,y2 as f64));
                            let square = [x1,y1,x2-x1,y2-y1];
                            rectangle([0.0,0.0,1.0,0.8], square, c.transform, g);
                        }

                        intersects_box(ray_point,ray_dir,a.rect)
                    };

                    
                    tree.raycast(ray_point,ray_dir,bb,fast_func,ray_touch_box)
                };

                let (ppx,ppy)=if let Some(k)=k{
                    let ppx=ray_point[0]+ray_dir[0]*k.1;
                    let ppy=ray_point[1]+ray_dir[1]*k.1;
                    (ppx,ppy)
                }else{
                    let ppx=ray_point[0]+ray_dir[0]*800;
                    let ppy=ray_point[1]+ray_dir[1]*800;
                    (ppx,ppy)
                };

                let arr=[ray_point[0] as f64,ray_point[1] as f64,ppx as f64,ppy as f64];
                line([0.0, 0.0, 0.0, 1.0], // black
                     2.0, // radius of line
                     arr, // [x0, y0, x1,y1] coordinates of line
                     c.transform,
                     g);
            }

        });
    }

}
