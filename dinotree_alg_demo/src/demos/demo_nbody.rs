use support::prelude::*;
use dinotree::nbody;
use dinotree::colfind;
use dinotree_geom;
use dinotree_geom::GravityTrait;


#[derive(Copy,Clone)]
struct NodeMass{
    center:[f64;2],
    mass:f64,
    force:[f64;2]
}

impl dinotree_geom::GravityTrait for NodeMass{
    type N=f64;
    fn pos(&self)->[f64;2]{
        self.center
    }
    fn mass(&self)->f64{
        self.mass
    }
    fn apply_force(&mut self,a:[f64;2]){
        self.force[0]+=a[0];
        self.force[1]+=a[1];
    }
}


#[derive(Clone,Copy)]
struct Bla;
impl nbody::NodeMassTrait for Bla{
    type T=BBox<F64n,Bot>;
    //type N=NotNaN<f64>;
    type No=NodeMass;

    //gravitate this nodemass with another node mass
    fn handle_node_with_node(&self,a:&mut Self::No,b:&mut Self::No){
        let _ = dinotree_geom::gravitate(a,b,0.0001,0.004,|a|a.sqrt());
    }

    //gravitate a bot with a bot
    fn handle_bot_with_bot(&self,a:&mut Self::T,b:&mut Self::T){
        let _ = dinotree_geom::gravitate(&mut a.inner,&mut b.inner,0.0001,0.004,|a|a.sqrt());
    }

    //gravitate a nodemass with a bot
    fn handle_node_with_bot(&self,a:&mut Self::No,b:&mut Self::T){
        let _ = dinotree_geom::gravitate(a,&mut b.inner,0.0001,0.004,|a|a.sqrt());
    }


    fn new<'a,I:Iterator<Item=&'a Self::T>> (&'a self,it:I)->Self::No{
        let mut total_x=0.0;
        let mut total_y=0.0;
        let mut total_mass=0.0;

        for i in it{
            let m=i.inner.mass();
            total_mass+=m;
            total_x+=m*i.inner.pos[0];
            total_y+=m*i.inner.pos[1];
        }
        //println!("total={:?}");
        let center=if total_mass!=0.0{
            [total_x/total_mass,
            total_y/total_mass]
        }else{
            [0.0;2]
        };
        NodeMass{center,mass:total_mass,force:[0.0;2]}
    }


    fn apply_to_bots<'a,I:Iterator<Item=&'a mut Self::T>> (&'a self,a:&'a Self::No,it:I){

        let len_sqr=(a.force[0]*a.force[0])+(a.force[1]*a.force[1]);

        if len_sqr>0.000001{
            let total_forcex=a.force[0];
            let total_forcey=a.force[1];

            //let forcex=total_forcex/len as f64;
            //let forcey=total_forcey/len as f64;
            for i in it{
                let forcex=total_forcex*(i.inner.mass/a.mass);
                let forcey=total_forcey*(i.inner.mass/a.mass);
                i.inner.apply_force([forcex,forcey]);
            }
        }else{
            //No acceleration was applied to this node mass.
        }
    }

    fn is_far_enough(&self,depth:usize,b:[F64n;2])->bool{
                
        let a=b[0];

        let x=(depth+1) as f64;
        
        (a.into_inner()-b[1].into_inner()).abs()>800.0/x
    }

    fn is_far_enough_half(&self,depth:usize,b:[F64n;2])->bool{
        
        let a=b[0];
        let x=(depth+1) as f64;
        (a.into_inner()-b[1].into_inner()).abs()>400.0/x
    }

}



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
        Conv::from_rect(aabb_from_pointf64(self.pos,[r;2]))             
    }
}
impl dinotree_geom::GravityTrait for Bot{
    type N=f64;
    fn pos(&self)->[f64;2]{
        self.pos
    }
    fn mass(&self)->f64{
        self.mass
    }
    fn apply_force(&mut self,a:[f64;2]){
        self.force[0]+=a[0];
        self.force[1]+=a[1];
    }
}


pub struct DemoNbody{
    dim:[f64;2],
    bots:Vec<Bot>,
    no_mass_bots:Vec<Bot>
}
impl DemoNbody{
    pub fn new(dim:[f64;2])->DemoNbody{
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

        DemoNbody{dim:dim1,bots,no_mass_bots}
    }
}

impl DemoSys for DemoNbody{
    fn step(&mut self,cursor:[f64;2],c:&piston_window::Context,g:&mut piston_window::G2d){
        let no_mass_bots=&mut self.no_mass_bots;
        let bots=&mut self.bots;
        
        let mut tree={
            let n=NodeMass{center:[0.0;2],mass:0.0,force:[0.0;2]};
            DynTree::new(axgeom::XAXISS,n,bots.drain(..).map(
                |a|{
                    BBox::new(a.create_aabb(),a)
                }))
        };
        
        nbody::nbody_par(&mut tree,Bla);
        

        let mut tree=tree.with_extra(());                
      
        colfind::query_par_mut(&mut tree,|a, b| {
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


        //Draw bots.
        for bot in tree.iter(){
            draw_rect_f64n([0.0,0.5,0.0,1.0],bot.get(),c,g);
        }

        for b in tree.into_iter_orig_order(){
            bots.push(b.inner);
        }
        
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
            dinotree_geom::wrap_position(&mut bot.pos,self.dim);  
        }


        match no_mass_bots.pop(){
            Some(mut b)=>{
                b.mass=20.0;                
                b.pos[0]=cursor[0];
                b.pos[1]=cursor[1];
                b.force=[0.0;2];
                b.vel=[1.0,0.0];
                bots.push(b);
            },
            None=>{}
        }     
    }
}
