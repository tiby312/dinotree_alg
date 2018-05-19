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
use ordered_float::NotNaN;

fn main() {

    let mut bots=create_bots_f64(|id,pos|Bot{id,col:Vec::new()},&[0,800,0,800],500,[2,20]);

    let mut window: PistonWindow = WindowSettings::new("dinotree test", [800, 800])
        .exit_on_esc(true)
        .build()
        .unwrap();

    let mut cursor=[NotNaN::new(0.0).unwrap(),NotNaN::new(0.0).unwrap()];
    while let Some(e) = window.next() {
        e.mouse_cursor(|x, y| {
            cursor = [NotNaN::new(x).unwrap(), NotNaN::new(y).unwrap()];
        });

        window.draw_2d(&e, |mut c, mut g| {
            clear([1.0; 4], g);

            for bot in bots.iter(){
                let ((x1,x2),(y1,y2))=bot.rect.get();
                let ((x1,x2),(y1,y2))=((x1.into_inner(),x2.into_inner()),(y1.into_inner(),y2.into_inner()));
                let square = [x1,y1,x2-x1,y2-y1];
                                        
                rectangle([0.0,0.0,0.0,0.3], square, c.transform, g);
            }
            
            {
                let mut tree = DinoTree::new(&mut bots, StartAxis::Xaxis);


                let k={
                    
                    let v={
                        
                        #[derive(Copy,Clone,Ord,Eq,PartialEq,PartialOrd,Debug)]
                        struct DisSqr(NotNaN<f64>);
                        struct Kn<'a,'c:'a>{
                            c:&'a Context,
                            g:&'a mut G2d<'c>,
                            //v:&'a mut Vec<(ColSingle<'c,BBox<NotNaN<f64>,Bot>>,DisSqr)>
                        };

                        impl<'a,'c:'a> Knearest for Kn<'a,'c>{
                            type T=BBox<NotNaN<f64>,Bot>;
                            type N=NotNaN<f64>;
                            type D=DisSqr;
                            fn twod_check(&mut self, point:[Self::N;2],aabb:&AABBox<Self::N>)->Self::D{
                                {
                                    let ((x1,x2),(y1,y2))=aabb.get();
                                    
                                    {
                                        let ((x1,x2),(y1,y2))=((x1.into_inner(),x2.into_inner()),(y1.into_inner(),y2.into_inner()));
                                        let square = [x1,y1,x2-x1,y2-y1];
                                        rectangle([0.0,0.0,0.0,0.5], square, self.c.transform, self.g);
                                    }
                                    
                                    
                                }
                                let (px,py)=(point[0],point[1]);

                                let ((a,b),(c,d))=aabb.get();

                                let xx=num::clamp(px,a,b);
                                let yy=num::clamp(py,c,d);

                                DisSqr((xx-px)*(xx-px) + (yy-py)*(yy-py))
                            }

                            fn oned_check(&mut self,p1:Self::N,p2:Self::N)->Self::D{
                                DisSqr((p2-p1)*(p2-p1))
                            }

                            //create a range around n.
                            fn create_range(&mut self,b:Self::N,d:Self::D)->[Self::N;2]{
                                let dis=d.0.sqrt();
                                [b-dis,b+dis]
                            }

                            /*
                            fn handle(&mut self,a:ColSingle<'c,Self::T>,dis_sqr:Self::D){
                                self.v.push((a,dis_sqr))
                            }
                            */
                        }

                        let mut vv=Vec::new();
                        {
                            let mut kn=Kn{c:&c,g};
                            tree.k_nearest([cursor[0] ,cursor[1] ],3,kn,|a,b|{vv.push((a,b))});
                        }
                        vv
                    };

                    let cols=[
                        [1.0,0.0,0.0,0.8], //red closest
                        [0.0,1.0,0.0,0.8], //green second closest
                        [0.0,0.0,1.0,0.8]  //blue third closets
                    
                    ];
                    
                    for (i,a) in v.iter().enumerate(){
                        let ((x1,x2),(y1,y2))=a.0.rect.get();
                        
                        {
                            let ((x1,x2),(y1,y2))=((x1.into_inner(),x2.into_inner()),(y1.into_inner(),y2.into_inner()));
                            let square = [x1,y1,x2-x1,y2-y1];
                                        
                            rectangle(cols[i], square, c.transform, g);
                        }
                    } 
                                       
                };
            }
        });
    }
}
