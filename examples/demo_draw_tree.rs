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


            let ray_point=(cursor[0] as isize,cursor[1] as isize);
            let ray_dir=(-1,-2);
            //https://tavianator.com/fast-branchless-raybounding-box-intersections/


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

                    struct Bla<'a,'b:'a>{
                        c:&'a Context,
                        g:&'a mut G2d<'b>
                    }
                    impl<'a,'b:'a> dinotree::graphics::DividerDrawer for Bla<'a,'b>{
                        type N=isize;
                        fn draw_divider<A:axgeom::AxisTrait>(&mut self,div:isize,length:[isize;2],depth:usize){
                            let div=div as f64;
                            

                            let arr=if A::new().is_xaxis(){
                                [div,length[0] as f64,div,length[1] as f64]
                            }else{
                                [length[0] as f64,div,length[1] as f64,div]
                            };


                            let radius=(5-depth) as f64;

                            line([0.0, 0.0, 0.0, 0.5], // black
                                 radius, // radius of line
                                 arr, // [x0, y0, x1,y1] coordinates of line
                                 self.c.transform,
                                 self.g);
                            
                        }
                    }
                    let mut dd=Bla{c:&c,g};
                    dinotree::graphics::draw(&tree,&mut dd,AABBox::new((0,800),(0,800)));
                    
                    //tree.raycast(ray_point,ray_dir,bb,fast_func,ray_touch_box)
                };

                
            }

        });
    }

}
