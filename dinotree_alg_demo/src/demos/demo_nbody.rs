use support::prelude::*;
use dinotree::nbody;
use dinotree::k_nearest;
use dinotree;
use dinotree::colfind;
use rand;

#[derive(Copy,Clone)]
struct NodeMass{
    center:[f64;2],
    mass:f64,
    force:[f64;2]
}

impl GravityTrait for NodeMass{
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
    type T=BBox<NotNaN<f64>,Bot>;
    //type N=NotNaN<f64>;
    type No=NodeMass;

    //gravitate this nodemass with another node mass
    fn handle_node_with_node(&self,a:&mut Self::No,b:&mut Self::No){
        gravity::gravitate(a,b);
    }

    //gravitate a bot with a bot
    fn handle_bot_with_bot(&self,a:&mut Self::T,b:&mut Self::T){
        gravity::gravitate(&mut a.inner,&mut b.inner);
    }

    //gravitate a nodemass with a bot
    fn handle_node_with_bot(&self,a:&mut Self::No,b:&mut Self::T){
        gravity::gravitate(a,&mut b.inner);
    }


    fn new<'a,I:Iterator<Item=&'a Self::T>> (&'a self,it:I)->Self::No{
        let mut total_x=0.0;
        let mut total_y=0.0;
        let mut total_mass=0.0;

        let mut total=0;
        for i in it{
            let m=i.inner.mass();
            total_mass+=m;
            total_x+=m*i.inner.pos[0];
            total_y+=m*i.inner.pos[1];
            total+=1;
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

    fn is_far_enough(&self,depth:usize,b:[NotNaN<f64>;2])->bool{
                
        let a=b[0];

        let x=(depth+1) as f64;
        
        (a-b[1].into_inner()).abs()>800.0/x
    }

    fn is_far_enough_half(&self,depth:usize,b:[NotNaN<f64>;2])->bool{
        
        let a=b[0];
        let x=(depth+1) as f64;
        (a-b[1].into_inner()).abs()>400.0/x
    }

}



pub struct Bot{
    pos:[f64;2],
    vel:[f64;2],
    force:[f64;2],
    mass:f64
}
impl Bot{

    fn wrap_position(a:&mut [f64;2]){
        if a[0]>800.0{
            a[0]=0.0
        }
        if a[0]<0.0{
            a[0]=800.0;
        }
        if a[1]>800.0{
            a[1]=0.0
        }
        if a[1]<0.0{
            a[1]=800.0;
        }
    }
    fn handle(&mut self){
        
        let b=self;

        b.pos[0]+=b.vel[0];
        b.pos[1]+=b.vel[1];
    
        Self::wrap_position(&mut b.pos);

        //F=MA
        //A=F/M
        let accx=b.force[0]/b.mass;
        let accy=b.force[1]/b.mass;

        b.vel[0]+=accx;
        b.vel[1]+=accy;            

        

        b.force=[0.0;2];
    }
    fn create_aabb(&self)->axgeom::Rect<f64N>{
        let r=5.0f64.min(self.mass.sqrt()/10.0);
        let x1=self.pos[0]-r;
        let x2=self.pos[0]+r;
        let y1=self.pos[1]-r;
        let y2=self.pos[1]+r;
        let mut rect=axgeom::Rect::new(x1,x2,y1,y2);
        support::rectf64_to_notnan(rect)              
    }
}
impl GravityTrait for Bot{
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



use self::gravity::GravityTrait;
mod gravity{
    pub trait GravityTrait{
        fn pos(&self)->[f64;2];
        fn mass(&self)->f64;
        fn apply_force(&mut self,[f64;2]);
    }

    //Returns the force to be exerted to the first object.
    //The force to the second object can be retrieved simply by negating the first.
    pub fn gravitate<T:GravityTrait,T2:GravityTrait>(a:&mut T,b:&mut T2){
        let p1=a.pos();
        let p2=b.pos();
        let m1=a.mass();
        let m2=b.mass();

        let diffx=p2[0]-p1[0];
        let diffy=p2[1]-p1[1];
        let dis_sqr=diffx*diffx+diffy*diffy;


        if dis_sqr>0.0001{
            
            const GRAVITY_CONSTANT:f64=0.004;

            //newtons law of gravitation (modified for 2d??? divide by len instead of sqr)
            let force=GRAVITY_CONSTANT*(m1*m2)/dis_sqr;

            let dis=dis_sqr.sqrt();
            let finalx=diffx*(force/dis);
            let finaly=diffy*(force/dis);
            
            a.apply_force([finalx,finaly]);
            b.apply_force([-finalx,-finaly]);
        }else{
            //a.apply_force([1.0,0.0]);
            //b.apply_force([-1.0,0.0]);
        }
    }
}



pub struct DemoNbody{
    bots:Vec<Bot>,
    no_mass_bots:Vec<Bot>
}
impl DemoNbody{
    pub fn new(dim:[f64;2])->DemoNbody{

        let dim2=[f64n!(dim[0]),f64n!(dim[1])];
        let dim=&[0,dim[0] as isize,0,dim[1] as isize];
        let radius=[5,20];
        let velocity=[1,3];
        let mut bots:Vec<Bot>=create_world_generator(500,dim,radius,velocity).map(|ret|{
            Bot{pos:ret.pos,vel:ret.vel,force:[0.0;2],mass:100.0} //used to be 20
        }).collect();

        //Make one of the bots have a lot of mass.
        bots.last_mut().unwrap().mass=10000.0;

        let no_mass_bots:Vec<Bot>=Vec::new();

        DemoNbody{bots,no_mass_bots}
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
            /*
            let mut max_mag=0.0f64;
            let mag={
                let b=&bot;
                let x=b.force[0]-b.force_naive[0];
                let y=b.force[1]-b.force_naive[1];
                
                let dis=x*x+y*y;
                dis.sqrt()/b.mass //The more mass an object has, the less impact error has
            };
            max_mag=max_mag.max(mag);
            let mag=mag*100.0;
            */
            let ((x1,x2),(y1,y2))=bot.get().get();
            let arr=[x1.into_inner() as f64,y1.into_inner() as f64,x2.into_inner() as f64,y2.into_inner() as f64];
            let square = [arr[0],arr[1],arr[2]-arr[0],arr[3]-arr[1]];                    
            rectangle([0.0,0.5,0.0,1.0], square, c.transform, g);
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
