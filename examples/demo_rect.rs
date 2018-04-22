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

    let mut bots=create_bots_isize(|id|Bot{id,col:Vec::new()},&[0,800,0,800],1000,[2,20]);

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
                let ((x1,x2),(y1,y2))=((x1 as f64,x2 as f64),(y1 as f64,y2 as f64));

                let square = [x1,y1,x2-x1,y2-y1];
        
                rectangle([0.0,0.0,0.0,0.3], square, c.transform, g);
            }

            let cx=cursor[0] as isize;
            let cy=cursor[1] as isize;
			let r1=AABBox::new((cx-50,cx+50),(cy-50,cy+50));
            {
            	let ((x1,x2),(y1,y2))=r1.get();
                
                {
                    let ((x1,x2),(y1,y2))=((x1 as f64,x2 as f64),(y1 as f64,y2 as f64));
                    let square = [x1,y1,x2-x1,y2-y1];
                    rectangle([0.0,0.0,1.0,0.2], square, c.transform, g);
                }
            }  
            {
                let mut tree = DinoTree::new(&mut bots, StartAxis::Xaxis);

                let mut rects=tree.rects();


                rects.for_all_in_rect(&r1, |a| {
                    let ((x1,x2),(y1,y2))=a.rect.get();
                    
                    {
                        let ((x1,x2),(y1,y2))=((x1 as f64,x2 as f64),(y1 as f64,y2 as f64));
                        let square = [x1,y1,x2-x1,y2-y1];
                        rectangle([1.0,0.0,0.0,1.0], square, c.transform, g);
                    }

	            });

            }

        
        });
    }

}
