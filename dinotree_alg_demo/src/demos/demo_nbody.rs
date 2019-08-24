use crate::support::prelude::*;
use dinotree_alg::nbody;
use dinotree_alg::colfind;
use duckduckgeo;
use duckduckgeo::GravityTrait;
use dinotree_alg;


#[derive(Copy,Clone)]
struct NodeMass{
    rect:axgeom::Rect<F32n>,
    center:Vec2<f32>,
    mass:f32,
    force:Vec2<f32>
}

impl duckduckgeo::GravityTrait for NodeMass{
    type N=f32;
    fn pos(&self)->Vec2<f32>{
        self.center
    }
    fn mass(&self)->f32{
        self.mass
    }
    fn apply_force(&mut self,a:Vec2<f32>){
        self.force+=a;
    }
}


#[derive(Clone,Copy)]
struct Bla{
    num_pairs_checked:usize
}
impl nbody::NodeMassTraitMut for Bla{
    type T=BBox<F32n,Bot>;
    type No=NodeMass;

    fn get_rect(a:&Self::No)->&axgeom::Rect<F32n>{
        &a.rect
    }

    //gravitate this nodemass with another node mass
    fn handle_node_with_node(&mut self,a:&mut Self::No,b:&mut Self::No){
        
        let _ = duckduckgeo::gravitate(a,b,0.0001,0.004);
    }

    //gravitate a bot with a bot
    fn handle_bot_with_bot(&mut self,a:&mut Self::T,b:&mut Self::T){
        self.num_pairs_checked+=1;
        let _ = duckduckgeo::gravitate(&mut a.inner,&mut b.inner,0.0001,0.004);
    }

    //gravitate a nodemass with a bot
    fn handle_node_with_bot(&mut self,a:&mut Self::No,b:&mut Self::T){
        
        let _ = duckduckgeo::gravitate(a,&mut b.inner,0.0001,0.004);
    }


    fn new<'a,I:Iterator<Item=&'a Self::T>> (&'a mut self,it:I,rect:axgeom::Rect<F32n>)->Self::No{
        let mut total_x=0.0;
        let mut total_y=0.0;
        let mut total_mass=0.0;

        for i in it{
            let m=i.inner.mass();
            total_mass+=m;
            total_x+=m*i.inner.pos.x;
            total_y+=m*i.inner.pos.y;
        }
        
        let center=if total_mass!=0.0{
            vec2(total_x/total_mass,
            total_y/total_mass)
        }else{
            vec2same(0.0)
        };
        NodeMass{center,mass:total_mass,force:vec2same(0.0),rect}
    }

    fn apply_to_bots<'a,I:Iterator<Item=&'a mut Self::T>> (&'a mut self,a:&'a Self::No,it:I){

        if a.mass>0.000_000_1{

            let total_forcex=a.force.x;
            let total_forcey=a.force.y;

            for i in it{
                let forcex=total_forcex*(i.inner.mass/a.mass);
                let forcey=total_forcey*(i.inner.mass/a.mass);
                i.inner.apply_force(vec2(forcex,forcey));
            }
        }
    }

    fn is_far_enough(&self,b:[F32n;2])->bool{
        (b[0].into_inner()-b[1].into_inner()).abs()>200.0
    }

    fn is_far_enough_half(&self,b:[F32n;2])->bool{
        (b[0].into_inner()-b[1].into_inner()).abs()>100.0
    }

}


#[derive(Copy,Clone)]
pub struct Bot{
    pos:Vec2<f32>,
    vel:Vec2<f32>,
    force:Vec2<f32>,
    mass:f32
}
impl Bot{

    
    fn handle(&mut self){
        
        let b=self;

        b.pos+=b.vel;
        
        //F=MA
        //A=F/M
        let acc=b.force/b.mass;

        b.vel+=acc;
        

        b.force=vec2same(0.0);
    }
    fn create_aabb(&self)->axgeom::Rect<F32n>{
        let r=5.0f32.min(self.mass.sqrt()/10.0);
        axgeom::Rect::from_point(self.pos,vec2same(r)).inner_try_into().unwrap()            
    }
}
impl duckduckgeo::GravityTrait for Bot{
    type N=f32;
    fn pos(&self)->Vec2<f32>{
        self.pos
    }
    fn mass(&self)->f32{
        self.mass
    }
    fn apply_force(&mut self,a:Vec2<f32>){
        self.force+=a;
    }
}


pub struct DemoNbody{
    dim:Rect<F32n>,
    bots:Vec<Bot>,
    no_mass_bots:Vec<Bot>,
    max_percentage_error:f32
}
impl DemoNbody{
    pub fn new(dim:Rect<F32n>)->DemoNbody{
        

        let mut bots:Vec<_>=UniformRandGen::new(dim.inner_into()).
            take(4000).map(|pos|{
            Bot{mass:100.0,pos,vel:vec2same(0.0),force:vec2same(0.0)}
        }).collect();

        //Make one of the bots have a lot of mass.
        bots.last_mut().unwrap().mass=10000.0;

        let no_mass_bots:Vec<Bot>=Vec::new();

        DemoNbody{dim,bots,no_mass_bots,max_percentage_error:0.0}
    }
}

impl DemoSys for DemoNbody{
    fn step(&mut self,cursor:Vec2<F32n>,c:&piston_window::Context,g:&mut piston_window::G2d,check_naive:bool){
        let no_mass_bots=&mut self.no_mass_bots;
        let bots=&mut self.bots;
        
        let mut tree={
            //let n=NodeMass{center:[0.0;2],mass:0.0,force:[0.0;2],rect:axgeom::Rect::new(f32n!(0.0),f32n!(0.0),f32n!(0.0),f32n!(0.0))};

            DinoTreeBuilder::new(axgeom::XAXISS,&bots,|b|{b.create_aabb()}).build_par()
        };
        //println!("tree height={:?}",tree.get_height());

        /*
        fn n_choose_2(n:usize)->usize{
            ((n-1)*n)/2
        }
        */

        let border=self.dim;

        if !check_naive{
            nbody::nbody(&mut tree,&mut Bla{num_pairs_checked:0},border);
        }else{
            let mut bla=Bla{num_pairs_checked:0};
            nbody::nbody(&mut tree,&mut bla,border);
            let num_pair_alg=bla.num_pairs_checked;
            
            let (bots2,num_pair_naive)={
                let mut bots2:Vec<BBoxMut<F32n,Bot>>=bots.iter().map(|bot|BBoxMut::new(bot.create_aabb(),*bot)).collect();
                let mut num_pairs_checked=0;
                nbody::naive_mut(&mut bots2,|a,b|{
                    let _ = duckduckgeo::gravitate(&mut a.inner,&mut b.inner,0.00001,0.004);
                    num_pairs_checked+=1;
                });
                //assert_eq!(num_pairs_checked,n_choose_2(bots2.len()));
                (bots2,num_pairs_checked)
            };
            

            let mut bots3=bots.clone();
            for b in bots3.iter_mut(){
                b.force=vec2same(0.0);
            }
            tree.apply(&mut bots3,|b,t|*t=b.inner);

            {
                let mut max_diff=None;
            
                for (a,bb) in bots3.iter().zip(bots2.iter()){
                    let b=&bb.inner;

                    let dis_sqr1=a.force.magnitude2();
                    let dis_sqr2=b.force.magnitude2();
                    let dis1=dis_sqr1.sqrt();
                    let dis2=dis_sqr2.sqrt();

                    let acc_dis1=dis1/a.mass;
                    let acc_dis2=dis2/a.mass;

                    let diff=(acc_dis1-acc_dis2).abs();
                    
                    
                    let error:f32=(acc_dis2-acc_dis1).abs()/acc_dis2;
                                
                    match max_diff{
                        None=>{
                            max_diff=Some((diff,bb,error))
                        },
                        Some(max)=>{
                            if diff>max.0{
                                max_diff=Some((diff,bb,error))
                            }
                        }
                    }
                }
                let max_diff=max_diff.unwrap();
                self.max_percentage_error=max_diff.2*100.0;
             
                let f={
                    let a:f32=num_pair_alg as f32;
                    let b:f32=num_pair_naive as f32;
                    a/b
                };
                
                println!("absolute acceleration err={:06.5} percentage err={:06.2}% current bot not checked ratio={:05.2}%",max_diff.0,self.max_percentage_error,f*100.0);

                draw_rect_f32([1.0,0.0,1.0,1.0],max_diff.1.aabb.as_ref(),c,g);
            }
        }
              
        colfind::QueryBuilder::new(&mut tree).query_seq(|a, b| {
            let (a,b)=if a.inner.mass>b.inner.mass{
                (a,b)
            }else{
                (b,a)
            };

            if b.inner.mass!=0.0{
                
                let ma=a.inner.mass;
                let mb=b.inner.mass;
                let ua=a.inner.vel;
                let ub=b.inner.vel;

                //Do perfectly inelastic collision.
                let vx=(ma*ua.x+mb*ub.x)/(ma+mb);
                let vy=(ma*ua.y+mb*ub.y)/(ma+mb);
                assert!(!vx.is_nan()&&!vy.is_nan());
                a.inner.mass+=b.inner.mass;
                
                a.inner.force+=b.inner.force;
                a.inner.vel=vec2(vx,vy);


                b.inner.mass=0.0;
                b.inner.force=vec2same(0.0);
                b.inner.vel=vec2same(0.0);
                b.inner.pos=vec2same(0.0);
            }
        });

        if check_naive{
            struct Bla<'a,'b:'a>{
                c:&'a Context,
                g:&'a mut G2d<'b>
            }
            impl<'a,'b:'a> dinotree_alg::graphics::DividerDrawer for Bla<'a,'b>{
                type N=F32n;
                fn draw_divider<A:axgeom::AxisTrait>(&mut self,axis:A,div:F32n,cont:[F32n;2],length:[F32n;2],depth:usize){
                    let div=div.into_inner();
                    

                    let arr=if axis.is_xaxis(){
                        [div as f64,length[0].into_inner() as f64,div as f64,length[1].into_inner() as f64]
                    }else{
                        [length[0].into_inner() as f64,div as f64,length[1].into_inner() as f64,div as f64]
                    };


                    let radius=(1isize.max(5-depth as isize)) as f32;

                    line([0.0, 0.0 , 0.0 , 0.5 ], // black
                         radius as f64, // radius of line
                         arr, // [x0, y0, x1,y1] coordinates of line
                         self.c.transform,
                         self.g);

                    let [x1,y1,w1,w2]=if axis.is_xaxis(){
                        [cont[0],length[0],cont[1]-cont[0],length[1]-length[0]]
                    }else{
                        [length[0],cont[0],length[1]-length[0],cont[1]-cont[0]]
                    };

                    let square = [x1.into_inner() as f64,y1.into_inner() as f64,w1.into_inner()as f64,w2.into_inner() as f64];
                    rectangle([0.0,1.0,1.0,0.2], square, self.c.transform, self.g);
                
                    
                    
                }
            }

            let mut dd=Bla{c:&c,g};
            dinotree_alg::graphics::draw(&tree,&mut dd,&border);
        }

        //Draw bots.
        for bot in tree.get_bots().iter(){
            draw_rect_f32([0.0,0.5,0.0,1.0],bot.get().as_ref(),c,g);
        }


        tree.apply(bots,|b,t|*t=b.inner);
        
        {
            let mut new_bots=Vec::new();
            for b in bots.drain(..){
                if b.mass==0.0{
                    no_mass_bots.push(b);
                }else{
                    new_bots.push(b);
                }
            }
            bots.append(&mut new_bots);
        };


        
        //Update bot locations.
        for bot in bots.iter_mut(){
            Bot::handle(bot);  
            duckduckgeo::wrap_position(&mut bot.pos,*self.dim.as_ref());  
        }


        while let Some(mut b)=no_mass_bots.pop(){
            b.mass=20.0;     
            b.pos=cursor.inner_into();
            b.force=vec2same(0.0);
            b.vel=vec2(1.0,0.0);
            bots.push(b);
            //break;
        }     
    }
}
