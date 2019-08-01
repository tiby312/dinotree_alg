use crate::support::prelude::*;
use dinotree_alg::nbody;
use dinotree_alg::colfind;
use duckduckgeo;
use duckduckgeo::GravityTrait;
use dinotree_alg;

#[derive(Copy,Clone)]
struct NodeMass{
    rect:axgeom::Rect<F64n>,
    center:Vector2<f64>,
    mass:f64,
    force:Vector2<f64>
}

impl duckduckgeo::GravityTrait for NodeMass{
    type N=f64;
    fn pos(&self)->Vector2<f64>{
        self.center
    }
    fn mass(&self)->f64{
        self.mass
    }
    fn apply_force(&mut self,a:Vector2<f64>){
        self.force+=a;
    }
}


#[derive(Clone,Copy)]
struct Bla{
    num_pairs_checked:usize
}
impl nbody::NodeMassTraitMut for Bla{
    type T=BBox<F64n,Bot>;
    type No=NodeMass;

    fn get_rect(a:&Self::No)->&axgeom::Rect<F64n>{
        &a.rect
    }

    //gravitate this nodemass with another node mass
    fn handle_node_with_node(&mut self,a:&mut Self::No,b:&mut Self::No){
        
        let _ = duckduckgeo::gravitate(a,b,0.0001,0.004,|a|a.sqrt());
    }

    //gravitate a bot with a bot
    fn handle_bot_with_bot(&mut self,a:&mut Self::T,b:&mut Self::T){
        self.num_pairs_checked+=1;
        let _ = duckduckgeo::gravitate(&mut a.inner,&mut b.inner,0.0001,0.004,|a|a.sqrt());
    }

    //gravitate a nodemass with a bot
    fn handle_node_with_bot(&mut self,a:&mut Self::No,b:&mut Self::T){
        
        let _ = duckduckgeo::gravitate(a,&mut b.inner,0.0001,0.004,|a|a.sqrt());
    }


    fn new<'a,I:Iterator<Item=&'a Self::T>> (&'a mut self,it:I,rect:axgeom::Rect<F64n>)->Self::No{
        let mut total_x=0.0;
        let mut total_y=0.0;
        let mut total_mass=0.0;

        for i in it{
            let m=i.inner.mass();
            total_mass+=m;
            total_x+=m*i.inner.pos[0];
            total_y+=m*i.inner.pos[1];
        }
        
        let center=if total_mass!=0.0{
            [total_x/total_mass,
            total_y/total_mass]
        }else{
            [0.0;2]
        };
        NodeMass{center,mass:total_mass,force:[0.0;2],rect}
    }

    fn apply_to_bots<'a,I:Iterator<Item=&'a mut Self::T>> (&'a mut self,a:&'a Self::No,it:I){

        if a.mass>0.000_000_1{

            let total_forcex=a.force[0];
            let total_forcey=a.force[1];

            for i in it{
                let forcex=total_forcex*(i.inner.mass/a.mass);
                let forcey=total_forcey*(i.inner.mass/a.mass);
                i.inner.apply_force([forcex,forcey]);
            }
        }
    }

    fn is_far_enough(&self,b:[F64n;2])->bool{
        (b[0].into_inner()-b[1].into_inner()).abs()>200.0
    }

    fn is_far_enough_half(&self,b:[F64n;2])->bool{
        (b[0].into_inner()-b[1].into_inner()).abs()>100.0
    }

}


#[derive(Copy,Clone)]
pub struct Bot{
    pos:[f64;2],
    vel:[f64;2],
    force:[f64;2],
    mass:f64
}
impl Bot{

    
    fn handle(&mut self){
        
        let b=self;

        b.pos[0]+=b.vel[0];
        b.pos[1]+=b.vel[1];
    
        
        //F=MA
        //A=F/M
        let accx=b.force[0]/b.mass;
        let accy=b.force[1]/b.mass;

        b.vel[0]+=accx;
        b.vel[1]+=accy;            

        

        b.force=[0.0;2];
    }
    fn create_aabb(&self)->axgeom::Rect<F64n>{
        let r=5.0f64.min(self.mass.sqrt()/10.0);
        axgeom::Rect::from_point(self.pos,[r;2]).into_notnan().unwrap()            
    }
}
impl duckduckgeo::GravityTrait for Bot{
    type N=f64;
    fn pos(&self)->Vector2<f64>{
        self.pos
    }
    fn mass(&self)->f64{
        self.mass
    }
    fn apply_force(&mut self,a:Vector2<f64>){
        self.force+=a;
    }
}


pub struct DemoNbody{
    dim:[F64n;2],
    bots:Vec<Bot>,
    no_mass_bots:Vec<Bot>,
    max_percentage_error:f64
}
impl DemoNbody{
    pub fn new(dim:[F64n;2])->DemoNbody{
        let dim1=dim;
        let dim=&[0,dim[0] as isize,0,dim[1] as isize];
        let radius=[5,20];
        let velocity=[1,3];
        let mut bots:Vec<Bot>=create_world_generator(500,dim,radius,velocity).map(|ret|{
            Bot{pos:ret.pos,vel:ret.vel,force:[0.0;2],mass:100.0} //used to be 20
        }).collect();

        //Make one of the bots have a lot of mass.
        bots.last_mut().unwrap().mass=10000.0;

        let no_mass_bots:Vec<Bot>=Vec::new();

        DemoNbody{dim:dim1,bots,no_mass_bots,max_percentage_error:0.0}
    }
}

impl DemoSys for DemoNbody{
    fn step(&mut self,cursor:[F64n;2],c:&piston_window::Context,g:&mut piston_window::G2d,check_naive:bool){
        let no_mass_bots=&mut self.no_mass_bots;
        let bots=&mut self.bots;
        
        let mut tree={
            //let n=NodeMass{center:[0.0;2],mass:0.0,force:[0.0;2],rect:axgeom::Rect::new(f64n!(0.0),f64n!(0.0),f64n!(0.0),f64n!(0.0))};

            DinoTreeBuilder::new(axgeom::XAXISS,&bots,|b|{b.create_aabb()}).build_par()
        };
        //println!("tree height={:?}",tree.get_height());

        /*
        fn n_choose_2(n:usize)->usize{
            ((n-1)*n)/2
        }
        */


        let border=axgeom::Rect::new(NotNan::<_>::zero(),self.dim[0],NotNan::<_>::zero(),self.dim[1]);

        if !check_naive{
            nbody::nbody(&mut tree,&mut Bla{num_pairs_checked:0},border);
        }else{
            let mut bla=Bla{num_pairs_checked:0};
            nbody::nbody(&mut tree,&mut bla,border);
            let num_pair_alg=bla.num_pairs_checked;
            
            let (bots2,num_pair_naive)={
                let mut bots2:Vec<BBoxMut<F64n,Bot>>=bots.iter().map(|bot|BBoxMut::new(bot.create_aabb(),*bot)).collect();
                let mut num_pairs_checked=0;
                nbody::naive_mut(&mut bots2,|a,b|{
                    let _ = duckduckgeo::gravitate(&mut a.inner,&mut b.inner,0.00001,0.004,|a|a.sqrt());
                    num_pairs_checked+=1;
                });
                //assert_eq!(num_pairs_checked,n_choose_2(bots2.len()));
                (bots2,num_pairs_checked)
            };
            

            let mut bots3=bots.clone();
            for b in bots3.iter_mut(){
                b.force[0]=0.0;
                b.force[1]=0.0;
            }
            tree.apply(&mut bots3,|b,t|*t=b.inner);

            {
                let mut max_diff=None;
            
                for (a,bb) in bots3.iter().zip(bots2.iter()){
                    let b=&bb.inner;
                    //TODO what is this assertion?
                    //assert_eq!(a.mass,b.mass);
                    let dis_sqr1=a.force[0]*a.force[0]+a.force[1]*a.force[1];
                    let dis_sqr2=b.force[0]*b.force[0]+b.force[1]*b.force[1];
                    let dis1=dis_sqr1.sqrt();
                    let dis2=dis_sqr2.sqrt();
                    let acc_dis1=dis1/a.mass;
                    let acc_dis2=dis2/a.mass;

                    let diff=(acc_dis1-acc_dis2).abs();
                    
                    
                    let error:f64=(acc_dis2-acc_dis1).abs()/acc_dis2;
                                
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
                    let a:f64=num_pair_alg.as_();
                    let b:f64=num_pair_naive.as_();
                    a/b
                };
                
                println!("absolute acceleration err={:06.5} percentage err={:06.2}% current bot not checked ratio={:05.2}%",max_diff.0,self.max_percentage_error,f*100.0);

                draw_rect_f64n([1.0,0.0,1.0,1.0],&max_diff.1.aabb,c,g);
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
                let vx=(ma*ua[0]+mb*ub[0])/(ma+mb);
                let vy=(ma*ua[1]+mb*ub[1])/(ma+mb);
                assert!(!vx.is_nan()&&!vy.is_nan());
                a.inner.mass+=b.inner.mass;
                
                a.inner.force[0]+=b.inner.force[0];
                a.inner.force[1]+=b.inner.force[1];
                a.inner.vel[0]=vx;
                a.inner.vel[1]=vy;


                b.inner.mass=0.0;
                b.inner.force[0]=0.0;
                b.inner.force[1]=0.0;
                b.inner.vel[0]=0.0;
                b.inner.vel[1]=0.0;
                b.inner.pos[0]=0.0;
                b.inner.pos[1]=0.0;
            }
        });

        if check_naive{
            struct Bla<'a,'b:'a>{
                c:&'a Context,
                g:&'a mut G2d<'b>
            }
            impl<'a,'b:'a> dinotree_alg::graphics::DividerDrawer for Bla<'a,'b>{
                type N=F64n;
                fn draw_divider<A:axgeom::AxisTrait>(&mut self,axis:A,div:F64n,cont:[F64n;2],length:[F64n;2],depth:usize){
                    let div=div.into_inner();
                    

                    let arr=if axis.is_xaxis(){
                        [div,length[0].into_inner(),div,length[1].into_inner()]
                    }else{
                        [length[0].into_inner(),div,length[1].into_inner(),div]
                    };


                    let radius=(1isize.max(5-depth as isize)) as f64;

                    line([0.0, 0.0, 0.0, 0.5], // black
                         radius, // radius of line
                         arr, // [x0, y0, x1,y1] coordinates of line
                         self.c.transform,
                         self.g);

                    let [x1,y1,w1,w2]=if axis.is_xaxis(){
                        [cont[0],length[0],cont[1]-cont[0],length[1]-length[0]]
                    }else{
                        [length[0],cont[0],length[1]-length[0],cont[1]-cont[0]]
                    };

                    let square = [x1.into_inner(),y1.into_inner(),w1.into_inner(),w2.into_inner()];
                    rectangle([0.0,1.0,1.0,0.2], square, self.c.transform, self.g);
                
                    
                    
                }
            }

            let mut dd=Bla{c:&c,g};
            dinotree_alg::graphics::draw(&tree,&mut dd,&axgeom::Rect::new(NotNan::<_>::zero(),self.dim[0],NotNan::<_>::zero(),self.dim[1]));
        }

        //Draw bots.
        for bot in tree.get_bots().iter(){
            draw_rect_f64n([0.0,0.5,0.0,1.0],bot.get(),c,g);
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
            duckduckgeo::wrap_position(&mut bot.pos,self.dim);  
        }


        while let Some(mut b)=no_mass_bots.pop(){
            b.mass=20.0;                
            b.pos[0]=cursor[0];
            b.pos[1]=cursor[1];
            b.force=[0.0;2];
            b.vel=[1.0,0.0];
            bots.push(b);
            //break;
        }     
    }
}
