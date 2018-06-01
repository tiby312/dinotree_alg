extern crate piston_window;
extern crate axgeom;
extern crate num;
extern crate rand;
extern crate dinotree;

extern crate ordered_float;
use piston_window::*;

mod support;
use dinotree::*;



fn main() {

    let mut bots=support::create_bots_isize(|id|support::Bot{id,col:Vec::new()},&[0,800,0,800],500,[2,20]);


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
        
        
            {
                let tree = DinoTree::new(&mut bots, StartAxis::Xaxis);
            
                struct Bla<'a,'b:'a>{
                    c:&'a Context,
                    g:&'a mut G2d<'b>
                }
                impl<'a,'b:'a> dinotree::graphics::DividerDrawer for Bla<'a,'b>{
                    type N=isize;
                    fn draw_divider<A:axgeom::AxisTrait>(&mut self,div:isize,cont:Option<[isize;2]>,length:[isize;2],depth:usize){
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

                        match cont{
                            Some(cont)=>{
                                let [x1,y1,w1,w2]=if A::new().is_xaxis(){
                                    [cont[0],length[0],cont[1]-cont[0],length[1]-length[0]]
                                }else{
                                    [length[0],cont[0],length[1]-length[0],cont[1]-cont[0]]
                                };
                                //let ((x1,x2),(w1,w2))=((x1 as f64,x2 as f64),(w1 as f64,w2 as f64));
                
                                let square = [x1 as f64,y1 as f64,w1 as f64,w2 as f64];
                                rectangle([0.0,0.0,1.0,0.3], square, self.c.transform, self.g);
                            
                            },
                            None=>{

                            }
                        }
                        
                    }
                }
                let mut dd=Bla{c:&c,g};
                dinotree::graphics::draw(&tree,&mut dd,AABBox::new((0,800),(0,800)));
                                
            }

        });
    }

}
