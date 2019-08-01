use crate::support::prelude::*;
use dinotree_alg::k_nearest;
use duckduckgeo;
use dists;

use duckduckgeo::cast_2array;
use duckduckgeo::rect_from_point;

#[derive(Copy,Clone)]
struct Bot{
    id:usize,
    pos:Vector2<f64>,
    radius:Vector2<f64>
}

pub struct KnearestDemo{
    _bots:Vec<Bot>,
    tree:DinoTree<axgeom::XAXISS,BBox<F64n,Bot>>,
    _dim:Vector2<F64n>
}
impl KnearestDemo{
    pub fn new(dim:Vector2<F64n>)->KnearestDemo{

        let dim2:Vector2<f64>=dim.cast().unwrap();
        let border=axgeom::Rect::new(0.0,dim2.x,0.0,dim2.y);
        

        let rand_radius=dists::RandomRectBuilder::new(vec2(2.0,2.0),vec2(6.0,6.0));
        let bots:Vec<_>=dists::uniform_rand::UniformRangeBuilder::new(border).build().
            take(4000).zip(rand_radius).enumerate().map(|(id,(pos,radius))|{
            Bot{id,pos,radius}
        }).collect();


        let tree = DinoTreeBuilder::new(axgeom::XAXISS,&bots,|bot|{rect_from_point(bot.pos,bot.radius).cast().unwrap()}).build_par();
        KnearestDemo{_bots:bots,tree,_dim:dim}
    }
}

impl DemoSys for KnearestDemo{
    fn step(&mut self,cursor:Vector2<F64n>,c:&piston_window::Context,g:&mut piston_window::G2d,check_naive:bool){
        let tree=&mut self.tree;

        for bot in tree.get_bots().iter(){
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
                let rect=bot.get().cast().unwrap();
                let point=cast_2array(point).unwrap();

                draw_rect_f64n([0.0,0.0,1.0,0.5],bot.get(),self.c,self.g);
                
                let dis=rect.distance_squared_to_point(point);
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

                        let point=vec2(point[0],point[1]);
                        -(bot.inner.pos-point).magnitude2()
                    }
                };
                DisSqr(NotNan::new(dis).unwrap())
            }

            fn oned_check(&mut self,p1:Self::N,p2:Self::N)->Self::D{
                let p1=p1.into_inner();
                let p2=p2.into_inner();
                let diff=p2-p1;
                DisSqr(NotNan::new(diff*diff).unwrap())
            }

            //create a range around n.
            fn create_range(&mut self,b:Self::N,d:Self::D)->[Self::N;2]{
                if d.0.into_inner()<0.0{
                    [b,b]
                }else{
                    let b=b.into_inner();
                    let dis=d.0.into_inner().sqrt();
                    [NotNan::new(b-dis).unwrap(),NotNan::new(b+dis).unwrap()]
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
            k_nearest::k_nearest(&tree,[cursor.x,cursor.y],3,kn)
        };

        if check_naive{
            struct Kn2{};

            impl k_nearest::Knearest for Kn2{
                type T=BBox<F64n,Bot>;
                type N=F64n;
                type D=DisSqr;
                fn twod_check(&mut self, point:[Self::N;2],bot:&Self::T)->Self::D{
                    let rect=bot.get().cast().unwrap();
                    let point=cast_2array(point).unwrap();


                    let dis=rect.distance_squared_to_point(point);
                    //let dis=duckduckgeo::distance_squared_point_to_rect(point_notnan_to_inner(point),&bot.get().into_inner());
                    let dis=match dis{
                        Some(dis)=>{
                            dis
                        },
                        None=>{
                            //IMPORTANT THAT THIS NEGATIVE
                            
                            let point=vec2(point[0],point[1]);
                            -(bot.inner.pos-point).magnitude2()
                        }
                    };
                    DisSqr(NotNan::new(dis).unwrap())    
                }

                fn oned_check(&mut self,p1:Self::N,p2:Self::N)->Self::D{
                    let p1=p1.into_inner();
                    let p2=p2.into_inner();
                    DisSqr(NotNan::new((p2-p1)*(p2-p1)).unwrap())
                }

                //create a range around n.
                fn create_range(&mut self,b:Self::N,d:Self::D)->[Self::N;2]{
                    if d.0.into_inner()<0.0{
                        [b,b]
                    }else{
                        let b=b.into_inner();
                        let dis=d.0.into_inner().sqrt();
                        [NotNan::new(b-dis).unwrap(),NotNan::new(b+dis).unwrap()]
                    }
                }
            }
        
            let vv2={
                let kn=Kn2{};
                k_nearest::naive(tree.get_bots().iter(),[cursor.x,cursor.y],3,kn).into_iter()
            };
            

            for ((mut a,color),mut b) in vv.zip(cols.iter()).zip(vv2){
                a.bots.sort_unstable_by(|a,b|a.inner.id.cmp(&b.inner.id));
                b.bots.sort_unstable_by(|a,b|a.inner.id.cmp(&b.inner.id));
                
                for (&a,&b) in a.bots.iter().zip(b.bots.iter()){
                    if a as *const BBox<F64n,Bot> != b as *const BBox<F64n,Bot>{
                        println!("Fail");
                    }    
                }

                if a.mag!=b.mag{
                    println!("Fail");
                }
                
                draw_rect_f64n(*color,(a.bots)[0].get(),c,g);
            }
        
        }else{
            for (a,color) in vv.zip(cols.iter()){
                draw_rect_f64n(*color,(a.bots)[0].get(),c,g);
            }
        }
    }   
}

