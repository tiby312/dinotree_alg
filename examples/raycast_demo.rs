extern crate piston_window;
extern crate axgeom;
extern crate num;
extern crate rand;
extern crate dinotree;

use piston_window::*;

mod support;
use dinotree::*;
use dinotree::support::*;
use support::*;


fn main() {
    let mut p = PointGenerator::new(
        &support::make_rect((0, 800), (0, 800)),
        &[100, 42, 6],
    );

    let mut bots = Vec::new();
    for id in 0..1000 {
        let ppp = p.random_point();
        let k = support::create_rect_from_point(ppp);
        bots.push(BBox::new(
            Bot {
                id,
                col: Vec::new(),
            },
            k,
        ));
    }

    //let height = compute_tree_height(bots.len());



    let mut window: PistonWindow = WindowSettings::new("raycast test", [800, 800])
        .exit_on_esc(true)
        .build()
        .unwrap();

    let mut cursor=[0.0,0.0];
    while let Some(e) = window.next() {
        e.mouse_cursor(|x, y| {
            cursor = [x, y];
            //println!("Mouse moved '{} {}'", x, y);
        });

        window.draw_2d(&e, |c, g| {
            clear([1.0; 4], g);


            let ray=Ray{point:Vec2{x:cursor[0] as isize,y:cursor[1] as isize},dir:Vec2{x:-1,y:-1},tmax:None};

            //https://tavianator.com/fast-branchless-raybounding-box-intersections/

            {
                let mut tree = DinoTree::new(&mut bots, StartAxis::Xaxis);


                let k={
                    let ray_touch_box=|a:ColSingle<BBox<isize,Bot>>|->Option<isize>{
                        let ((x1,x2),(y1,y2))=a.rect.get();
                        {
                            //let ((x1,x2),(y1,y2))=bot.rect.get();
                            let arr=[x1 as f64,y1 as f64,x2 as f64,y2 as f64];
                            let square = rectangle::square(x1 as f64, y1 as f64, 8.0);
                    

                            rectangle([0.0,0.0,1.0,0.8], square, c.transform, g);
                        }
                        let point=ray.point;
                        let dir=ray.dir;


                        //top and bottom
                        //s(t)=point+t*dir
                        let mut tmin=isize::min_value();
                        let mut tmax=isize::max_value();

                        if dir.x!=0{
                            let tx1=(x1-point.x)/dir.x;
                            let tx2=(x2-point.x)/dir.x;

                            tmin=tmin.max(tx1.min(tx2));
                            tmax=tmax.min(tx1.max(tx2));
                            
                        }else{
                            if point.x < x1 || point.x > x2 {
                                return None; // parallel AND outside box : no intersection possible
                            }
                        }
                        if dir.y!=0{
                            let ty1=(y1-point.y)/dir.y;
                            let ty2=(y2-point.y)/dir.y;

                            tmin=tmin.max(ty1.min(ty2));
                            tmax=tmax.min(ty1.max(ty2));
                        }else{
                            if point.y < y1 || point.y > y2 {
                                return None; // parallel AND outside box : no intersection possible
                            }
                        }
                        if tmax>=tmin && tmax>=0{
                            //println!("bla=max:{:?} min:{:?}",tmax,tmin);
                            return Some(tmin.max(0));
                        }
                        
                        return None
                    };

                    let bb=RectInf{xdiv:(0,800),ydiv:(0,800)};
                    tree.raycast(ray,bb,ray_touch_box)
                };

                let (ppx,ppy)=if let Some(k)=k{
                    let ppx=ray.point.x+ray.dir.x*k.1;
                    let ppy=ray.point.y+ray.dir.y*k.1;
                    (ppx,ppy)
                }else{
                    let ppx=ray.point.x+ray.dir.x*800;
                    let ppy=ray.point.y+ray.dir.y*800;
                    (ppx,ppy)
                };
                //println!("{:?} {:?}",(ray.point.x,ray.point.y),(ppx,ppy));

                let arr=[ray.point.x as f64,ray.point.y as f64,ppx as f64,ppy as f64];
                line([0.0, 0.0, 0.0, 1.0], // black
                     2.0, // radius of line
                     arr, // [x0, y0, x1,y1] coordinates of line
                     c.transform,
                     g);
            }

            for bot in bots.iter(){
                let ((x1,x2),(y1,y2))=bot.rect.get();
                let arr=[x1 as f64,y1 as f64,x2 as f64,y2 as f64];
                let square = rectangle::square(x1 as f64, y1 as f64, 8.0);
        

                rectangle([0.0,0.0,0.0,0.3], square, c.transform, g);
                /*
                line([0.0, 0.0, 0.0, 1.0], // black
                     2.0, // radius of line
                     arr, // [x0, y0, x1,y1] coordinates of line
                     c.transform,
                     g);
                */
            }
        
        });
    }

}
