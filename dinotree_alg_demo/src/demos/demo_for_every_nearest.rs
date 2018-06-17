use support::prelude::*;
use dinotree::k_nearest;
use dinotree::for_every_nearest;

use dinotree::for_every_nearest::HasCenter;




pub struct KnearestEveryDemo{
    bots:Vec<Bot>,
    dim:[f64N;2]
}
impl KnearestEveryDemo{
    pub fn new(dim:[f64;2])->KnearestEveryDemo{
        let dim2=[f64n!(dim[0]),f64n!(dim[1])];
        let dim=&[0,dim[0] as isize,0,dim[1] as isize];
        let radius=[5,10];
        let velocity=[1,3];
        let bots=create_world_generator(100,dim,radius,velocity).map(|ret|{
            Bot{id:ret.id,pos:ret.pos,vel:ret.vel,radius:ret.radius,acc:[f64n!(0.0);2]}
        }).collect();

        KnearestEveryDemo{bots,dim:dim2}
    }
}

impl DemoSys for KnearestEveryDemo{
    fn step(&mut self,cursor:[f64N;2],c:&piston_window::Context,g:&mut piston_window::G2d){
        let bots=&mut self.bots;
        for b in bots.iter_mut(){
            b.update();
            b.wrap_position(self.dim);

        }

        {
            
            pub struct BInner{
                center:[f64N;2],
                acc:[f64N;2],
                rect:axgeom::Rect<f64N>
            }

            impl HasCenter for BInner{
                type Num=f64N;
                fn get_center(&self)->&[f64N;2]{
                    &self.center
                }
            }
            impl HasAabb for BInner{
                type Num=f64N;
                fn get(&self)->&axgeom::Rect<f64N>{
                    &self.rect
                }
            }
        
            

            #[derive(Copy,Clone,Ord,Eq,PartialEq,PartialOrd,Debug)]
            struct DisSqr(f64N);
            #[derive(Copy,Clone)]
            struct Kn;

            impl k_nearest::Knearest for Kn{
                type T=BInner;
                type N=f64N;
                type D=DisSqr;
                fn twod_check(&mut self, point:[Self::N;2],bot:&Self::T)->Self::D{

                    let (px,py)=(point[0],point[1]);

                    let ((a,b),(c,d))=bot.get().get();

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
            }

            let mut tree=DynTree::new(axgeom::YAXISS,(),bots.iter().map(|bot|{
                let p=bot.pos;
                let r=bot.radius;
                let acc=bot.acc;
                BInner{acc,rect:axgeom::Rect::new(p[0]-r[0],p[0]+r[0],p[1]-r[1],p[1]+r[1]),center:p}          
            }));

            for a in tree.iter(){
                let ((x1,x2),(y1,y2))=a.get().get();
                
                {
                    let ((x1,x2),(y1,y2))=((x1.into_inner(),x2.into_inner()),(y1.into_inner(),y2.into_inner()));
                    let square = [x1,y1,x2-x1,y2-y1];
                                
                    rectangle([0.0,1.0,0.0,0.5], square, c.transform, g);
                }
            } 

            for_every_nearest::for_every_nearest_mut(&mut tree,|a,b,dis|{
                let p1=*a.get_center();
                let p2=*b.get_center();

                let diff=[(p2[0]-p1[0])*0.0005,
                            (p2[1]-p1[1])*0.0005];
                a.acc[0]+=diff[0];
                a.acc[1]+=diff[1];
                b.acc[0]-=diff[0];
                b.acc[1]-=diff[1];

                let p1=[p1[0].into_inner(),p1[1].into_inner()];
                let p2=[p2[0].into_inner(),p2[1].into_inner()];
                


                let arr=[p1[0],p1[1],p2[0] ,p2[1] ];
                line([0.0, 0.0, 1.0, 0.2], // black
                     1.0, // radius of line
                     arr, // [x0, y0, x1,y1] coordinates of line
                     c.transform,
                     g);
            },Kn);

            //We didnt actualy modify anything in the tree so we dont need to inject
            //changes back into the bots.
            
            for (b,bot) in tree.into_iter_orig_order().zip(bots.iter_mut()){
                bot.acc[0]+=b.acc[0];
                bot.acc[1]+=b.acc[1];
                //bots.push(b);
            }
            
        }

        
    }   
}

