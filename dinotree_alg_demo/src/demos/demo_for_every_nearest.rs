use support::prelude::*;
use dinotree_alg::k_nearest;
use dinotree_alg::for_every_nearest;
use dinotree_alg::for_every_nearest::IsPoint;
use dinotree_geom;







#[derive(Copy,Clone)]
pub struct Bot{
    pub id:usize,
    pub pos:[f64;2],
    pub vel:[f64;2],
    pub acc:[f64;2],
}


impl IsPoint for Bot{
    type Num=F64n;
    fn get_center(&self)->[F64n;2]{
        [f64n!(self.pos[0]),f64n!(self.pos[1])]
    }
}

impl Bot{


    pub fn update(&mut self){
        self.vel[0]+=self.acc[0];
        self.vel[1]+=self.acc[1];
        self.pos[0]+=self.vel[0];
        self.pos[1]+=self.vel[1];
        self.acc[0]=0.0;
        self.acc[1]=0.0;
    }
}


pub struct KnearestEveryDemo{
    bots:Vec<Bot>,
    dim:[F64n;2]
}
impl KnearestEveryDemo{
    pub fn new(dim:[F64n;2])->KnearestEveryDemo{
        let dim2=&[0,dim[0] as isize,0,dim[1] as isize];
        let radius=[5,6];
        let velocity=[1,3];
        let bots=create_world_generator(100,dim2,radius,velocity).map(|ret|{
            Bot{id:ret.id,pos:ret.pos,vel:ret.vel,acc:[0.0;2]}
        }).collect();

        KnearestEveryDemo{bots,dim}
    }
}

impl DemoSys for KnearestEveryDemo{
    fn step(&mut self,_cursor:[f64;2],c:&piston_window::Context,g:&mut piston_window::G2d,_check_naive:bool){
        let bots=&mut self.bots;
        for b in bots.iter_mut(){
            b.update();
            dinotree_geom::wrap_position(&mut b.pos,self.dim);
            
        }

        {

            #[derive(Copy,Clone,Ord,Eq,PartialEq,PartialOrd,Debug)]
            struct DisSqr(F64n);
            #[derive(Copy,Clone)]
            struct Kn;

            impl k_nearest::Knearest for Kn{
                type T=BBox<F64n,Bot>;
                type N=F64n;
                type D=DisSqr;
                fn twod_check(&mut self, point:[Self::N;2],bot:&Self::T)->Self::D{
                    let k=dinotree_geom::distance_squared_point_to_rect(point,bot.get()).unwrap();

                    DisSqr(k)
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

            let mut tree=DinoTree::new(axgeom::YAXISS,(),&bots,|b|{
                Conv::from_rect(aabb_from_pointf64(b.pos,[0.0;2]))
            });


            for a in tree.iter(){
                let p=Conv::point_to_inner(a.inner.get_center());
                let r=5.0;
                let r=Conv::from_rect(aabb_from_pointf64(p,[r;2]));
                draw_rect_f64n([0.0,1.0,0.0,0.5],&r,c,g);
            } 
            /*
            for_every_nearest::for_every_nearest_mut(&mut tree,|a,b,_dis|{
                let a=&mut a.inner;
                let b=&mut b.inner;
                let p1=a.get_center();
                let p2=b.get_center();
                let p1=[p1[0].into_inner(),p1[1].into_inner()];
                let p2=[p2[0].into_inner(),p2[1].into_inner()];
                
                let diff=[(p2[0]-p1[0])*0.001,
                            (p2[1]-p1[1])*0.001];
                            
                a.acc[0]+=diff[0];
                a.acc[1]+=diff[1];
                b.acc[0]-=diff[0];
                b.acc[1]-=diff[1];



                let arr=[p1[0],p1[1],p2[0] ,p2[1] ];
                line([0.0, 0.0, 1.0, 0.2], // black
                     1.0, // radius of line
                     arr, // [x0, y0, x1,y1] coordinates of line
                     c.transform,
                     g);
            },Kn);
            */
            //We didnt actualy modify anything in the tree so we dont need to inject
            //changes back into the bots.
            

            tree.apply(bots,|b,t|{
                t.acc[0]+=b.inner.acc[0];
                t.acc[1]+=b.inner.acc[1];
            })


        }

        
    }   
}

