use support::prelude::*;
use dinotree::k_nearest;
use dinotree_geom;
use dinotree;


#[derive(Copy,Clone)]
struct Bot{
    pos:[f64;2],
    radius:[f64;2]
}

pub struct KnearestDemo{
    bots:Vec<Bot>,
    tree:DynTree<axgeom::XAXISS,(),BBox<F64n,Bot>>,
    dim:[f64;2]
}
impl KnearestDemo{
    pub fn new(dim:[f64;2])->KnearestDemo{
        let dim2=&[0,dim[0] as isize,0,dim[1] as isize];
        let radius=[5,20];
        let velocity=[1,3];
        let bots:Vec<Bot>=create_world_generator(500,dim2,radius,velocity).map(|ret|{
            let p=ret.pos;
            let r=ret.radius;
            Bot{pos:ret.pos,radius:ret.radius}
        }).collect();

        let tree = DynTree::new(axgeom::XAXISS,(),&bots,|bot|{Conv::from_rect(aabb_from_pointf64(bot.pos,bot.radius))});
        KnearestDemo{bots,tree,dim}
    }
}

impl DemoSys for KnearestDemo{
    fn step(&mut self,cursor:[f64;2],c:&piston_window::Context,g:&mut piston_window::G2d){
        let tree=&mut self.tree;

        for bot in tree.iter_every_bot(){
            draw_rect_f64n([0.0,0.0,0.0,0.3],bot.get(),c,g);
        }

        #[derive(Copy,Clone,Ord,Eq,PartialEq,PartialOrd,Debug)]
        struct DisSqr(F64n);
        struct Kn<'a,'c:'a>{
            c:&'a Context,
            g:&'a mut G2d<'c>,
        };

        impl<'a,'c:'a> k_nearest::Knearest for Kn<'a,'c>{
            type T=BBox<F64n,Bot>;
            type N=F64n;
            type D=DisSqr;
            fn twod_check(&mut self, point:[Self::N;2],bot:&Self::T)->Self::D{
                
                draw_rect_f64n([0.0,0.0,1.0,0.5],bot.get(),self.c,self.g);
                
                let dis=dinotree_geom::distance_squared_point_to_rect(Conv::point_to_inner(point),&Conv::rect_to_inner(*bot.get()));
                let dis=match dis{
                    Some(dis)=>{
                        dis
                    },
                    None=>{
                        //If a point is insert a rect, the distance to it is zero.
                        //So if multiple points are inside of a rect, its not clear the order in which
                        //they should be returned.
                        //So in the case that a point is in the rect, we establish our own ordering,
                        //by falling back on the distance between the center of a rect and the point.
                        //Since the distance between a rect and a point that is outside of the rect is 
                        //guarenteeded to be positive, we have all the negative numbers in which to
                        //apply our custom ordering for bots that are inside of the rect.
                        
                        //The main reason that we are doing this is so that there arn't
                        //multiple solutions to the k_nearest problem so that we can easily
                        //verify the solution against the naive implementation.

                        //If you don't care about a single solution existing, you can simply return zero
                        //for the cases that the point is inside of the rect.

                        let point=Conv::point_to_inner(point);
                        -dinotree_geom::distance_squred_point(bot.inner.pos,point)
                    }
                };
                DisSqr(f64n!(dis))
            }

            fn oned_check(&mut self,p1:Self::N,p2:Self::N)->Self::D{
                let p1=p1.into_inner();
                let p2=p2.into_inner();
                let diff=p2-p1;
                DisSqr(f64n!(diff*diff))
            }

            //create a range around n.
            fn create_range(&mut self,b:Self::N,d:Self::D)->[Self::N;2]{
                if d.0.into_inner()<0.0{
                    [b,b]
                }else{
                    let b=b.into_inner();
                    let dis=d.0.into_inner().sqrt();
                    [f64n!(b-dis),f64n!(b+dis)]
                }
            }
        }

            
        let cols=[
            [1.0,0.0,0.0,0.8], //red closest
            [0.0,1.0,0.0,0.8], //green second closest
            [0.0,0.0,1.0,0.8]  //blue third closets
        ];
        
        let vv={
            let kn=Kn{c:&c,g};
            let point=[f64n!(cursor[0]),f64n!(cursor[1])];
            k_nearest::k_nearest(tree,point,1,kn)
        };

        let check_naive=true;
        if check_naive{

            struct Kn2{};

            impl k_nearest::Knearest for Kn2{
                type T=BBox<F64n,Bot>;
                type N=F64n;
                type D=DisSqr;
                fn twod_check(&mut self, point:[Self::N;2],bot:&Self::T)->Self::D{
                    let dis=dinotree_geom::distance_squared_point_to_rect(Conv::point_to_inner(point),&Conv::rect_to_inner(*bot.get()));
                    let dis=match dis{
                        Some(dis)=>{
                            dis
                        },
                        None=>{
                            //IMPORTANT THAT THIS NEGATIVE
                            let point=Conv::point_to_inner(point);
                            -dinotree_geom::distance_squred_point(bot.inner.pos,point)
                        }
                    };
                    DisSqr(f64n!(dis))    
                }

                fn oned_check(&mut self,p1:Self::N,p2:Self::N)->Self::D{
                    let p1=p1.into_inner();
                    let p2=p2.into_inner();
                    DisSqr(f64n!((p2-p1)*(p2-p1)))
                }

                //create a range around n.
                fn create_range(&mut self,b:Self::N,d:Self::D)->[Self::N;2]{
                    if d.0.into_inner()<0.0{
                        [b,b]
                    }else{
                        let b=b.into_inner();
                        let dis=d.0.into_inner().sqrt();
                        [f64n!(b-dis),f64n!(b+dis)]
                    }
                }
            }
        
            let vv2={
                let kn=Kn2{};
                let point=[f64n!(cursor[0]),f64n!(cursor[1])];
                k_nearest::naive(tree.iter_every_bot(),point,1,kn)
            };
            

            for ((a,color),b) in vv.zip(cols.iter()).zip(vv2){
                if a.0 as *const BBox<F64n,Bot> != b.0 as *const BBox<F64n,Bot>{
                    println!("Fail");
                }
                draw_rect_f64n(*color,a.0.get(),c,g);
            }
        
        }else{
            for (a,color) in vv.zip(cols.iter()){
                draw_rect_f64n(*color,a.0.get(),c,g);
            }
        }
    }   
}

