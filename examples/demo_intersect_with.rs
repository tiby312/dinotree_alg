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
    let mut bots1=create_bots_isize(|id|Bot{id,col:Vec::new()},&[0,800,0,800],500,[2,20]);
    let mut bots2=create_bots_isize_seed(&[50,9,20],|id|Bot{id,col:Vec::new()},&[0,800,0,800],50,[2,20]);

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
            
            for bot in bots1.iter(){
                let ((x1,x2),(y1,y2))=bot.rect.get();
                let ((x1,x2),(y1,y2))=((x1 as f64,x2 as f64),(y1 as f64,y2 as f64));
                    
                let square = [x1,y1,x2-x1,y2-y1];
                rectangle([1.0,0.0,0.0,0.5], square, c.transform, g);
            }

            for bot in bots2.iter(){
                let ((x1,x2),(y1,y2))=bot.rect.get();
                let ((x1,x2),(y1,y2))=((x1 as f64,x2 as f64),(y1 as f64,y2 as f64));
                    
                let square = [x1,y1,x2-x1,y2-y1];
                rectangle([0.0,1.0,0.0,0.5], square, c.transform, g);
            }

            {
                let mut tree = DinoTree::new(&mut bots1, StartAxis::Xaxis);


                let k={
                    
                                
                    tree.intersect_with_seq(&mut bots2,|a, b| {
                       
                        let mid1={
                            let ((x1,x2),(y1,y2))=a.rect.get();
                            let ((x1,x2),(y1,y2))=((x1 as f64,x2 as f64),(y1 as f64,y2 as f64));
                            
                            [x1+(x2-x1)/2.0,y1+(y2-y1)/2.0]
                        };

                        let mid2={
                            let ((x1,x2),(y1,y2))=b.rect.get();
                            let ((x1,x2),(y1,y2))=((x1 as f64,x2 as f64),(y1 as f64,y2 as f64));
                            
                            [x1+(x2-x1)/2.0,y1+(y2-y1)/2.0]
                        };
                        
                        let arr=[mid1[0],mid1[1],mid2[0],mid2[1]];
                        line([0.0, 0.0, 0.0, 1.0], // black
                             1.0, // radius of line
                             arr, // [x0, y0, x1,y1] coordinates of line
                             c.transform,
                             g);
                    });
                };
            }
        });
    }
}
