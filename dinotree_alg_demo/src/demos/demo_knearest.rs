use crate::support::prelude::*;
use dinotree_alg::k_nearest;
use std::cell::RefCell;
use k_nearest::SliceSplitMut;
use k_nearest::SliceSplit;
use k_nearest::Unit;

#[derive(Copy,Clone)]
struct Bot{
    id:usize,
    pos:Vec2<f32>,
    radius:Vec2<f32>
}

pub struct KnearestDemo{
    _bots:Vec<Bot>,
    tree:DinoTree<XAXISS,BBox<F32n,Bot>>,
    dim:Rect<F32n>
}

impl KnearestDemo{
    pub fn new(dim:Rect<F32n>)->KnearestDemo{

        let bots:Vec<_>=UniformRandGen::new(dim.inner_into()).with_radius(2.0,10.0).
            take(2000).enumerate().map(|(id,(pos,radius))|{
            Bot{id,pos,radius}
        }).collect();


        let tree = DinoTreeBuilder::new(axgeom::XAXISS,&bots,|bot|{Rect::from_point(bot.pos,bot.radius).inner_try_into().unwrap()}).build_par();
        KnearestDemo{_bots:bots,tree,dim}
    }
}

impl DemoSys for KnearestDemo{
    fn step(&mut self,cursor:Vec2<F32n>,c:&piston_window::Context,g:&mut piston_window::G2d,check_naive:bool){
        let tree=&mut self.tree;

        for bot in tree.get_bots().iter(){
            draw_rect_f32([0.0,0.0,0.0,0.3],bot.get().as_ref(),c,g);
        }

        struct Kn<'a,'c:'a>{
            draw:bool,
            c:&'a Context,
            g:RefCell<&'a mut G2d<'c>>,
        };

        impl<'a,'c:'a> k_nearest::Knearest for Kn<'a,'c>{
            type T=BBox<F32n,Bot>;
            type N=F32n;

            fn distance_to_bot(&self,point:Vec2<Self::N>,bot:&Self::T)->Self::N{
                if self.draw{
                    draw_rect_f32([0.0,0.0,1.0,0.5],bot.get().as_ref(),self.c,&mut self.g.borrow_mut());
                }
                self.distance_to_rect(point,bot.get())
            }
            fn distance_to_rect(&self, point:Vec2<Self::N>,rect:&Rect<Self::N>)->Self::N{
                
                let dis=rect.as_ref().distance_squared_to_point(point.inner_into());
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

                        0.0
                        //-(bot.inner.pos-point.inner_into()).magnitude2()
                    }
                };
                NotNan::new(dis).unwrap()
            }
        }

            
        let cols=[
            [1.0,0.0,0.0,0.8], //red closest
            [0.0,1.0,0.0,0.8], //green second closest
            [0.0,0.0,1.0,0.8]  //blue third closets
        ];
        
        let mut vv={
            let kn=Kn{c:&c,g:RefCell::new(g),draw:true};
            k_nearest::k_nearest(&tree,cursor,3,kn,self.dim)
        };

        /*
        if vv.len()>3{
            for i in vv.iter(){
                print!("{:?},",i.mag);    
            }
            println!();
        }
        */

        if check_naive{
            
        
            let mut vv2={
                let kn=Kn{c:&c,g:RefCell::new(g),draw:false};
                k_nearest::naive(tree.get_bots().iter(),cursor,3,kn)
            };
        

            assert_eq!(vv.len(),vv2.len());

            let vv_iter=SliceSplitMut::new(&mut vv,|a,b|a.mag==b.mag);
            let vv2_iter=SliceSplitMut::new(&mut vv2,|a,b|a.mag==b.mag);

            for (a,b) in vv_iter.zip(vv2_iter){
                a.sort_unstable_by(|a,b|a.bot.inner.id.cmp(&b.bot.inner.id));
                b.sort_unstable_by(|a,b|a.bot.inner.id.cmp(&b.bot.inner.id));
                

                for (a,b) in a.iter().zip(b.iter()){
                    assert_eq!(a.mag,b.mag);
                    if a.bot as *const BBox<F32n,Bot> != b.bot as *const BBox<F32n,Bot>{
                        println!("Fail");
                    }    
                }

            }
            /*
            for ((mut a,color),mut b) in vv.iter().zip(cols.iter()).zip(vv2.iter()){
                
                
                draw_rect_f32(*color,(a.bots)[0].get().as_ref(),c,g);
            }
            */
        
        }else{
            let vv_iter=SliceSplit::new(&mut vv,|a,b|a.mag==b.mag);
                        
            for (a,color) in vv_iter.zip(cols.iter()){
                for b in a.iter(){
                    draw_rect_f32(*color,(b.bot).get().as_ref(),c,g);
                }
            }
            
        }
    }   
}

