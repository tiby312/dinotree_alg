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

            for bot in bots.iter(){
                let ((x1,x2),(y1,y2))=bot.rect.get();
                let arr=[x1 as f64,y1 as f64,x2 as f64,y2 as f64];
                let square = rectangle::square(x1 as f64, y1 as f64, 8.0);
        
                rectangle([0.0,0.0,0.0,0.3], square, c.transform, g);
            }
            
            {
                let mut tree = DinoTree::new(&mut bots, StartAxis::Xaxis);


                let k={
                    
                    let v={
                        //Compute distance sqr
                        let min_rect=|point:[isize;2],aabb:&AABBox<isize>|{
                            {
                                let ((x1,x2),(y1,y2))=aabb.get();
                            
                                {
                                    let ((x1,x2),(y1,y2))=((x1 as f64,x2 as f64),(y1 as f64,y2 as f64));
                                    let square = [x1,y1,x2-x1,y2-y1];
                                    rectangle([0.0,0.0,0.0,0.5], square, c.transform, g);
                                }
                            }
                            let (px,py)=(point[0],point[1]);

                            let ((a,b),(c,d))=aabb.get();

                            let xx=num::clamp(px,a,b);
                            let yy=num::clamp(py,c,d);

                            (xx-px)*(xx-px) + (yy-py)*(yy-py)
                        };

                        //Compute distance sqr in 1d cases.
                        let min_oned=&|p1:isize,p2:isize|{
                            (p2-p1)*(p2-p1)
                        };


                        let mut v=Vec::new();
                        tree.k_nearest([cursor[0] as isize,cursor[1] as isize],3,|a,dis|{v.push((a,dis))},min_rect,min_oned);
                        v
                    };

                    let cols=[
                        [1.0,0.0,0.0,0.8], //red closest
                        [0.0,1.0,0.0,0.8], //green second closest
                        [0.0,0.0,1.0,0.8]  //blue third closets
                    
                    ];

                    for (i,a) in v.iter().enumerate(){
                        let ((x1,x2),(y1,y2))=a.0.rect.get();
                        
                        {
                            let ((x1,x2),(y1,y2))=((x1 as f64,x2 as f64),(y1 as f64,y2 as f64));
                            let square = [x1,y1,x2-x1,y2-y1];
                            rectangle(cols[i], square, c.transform, g);
                        }
                    }                    
                };
            }
        });
    }
}
