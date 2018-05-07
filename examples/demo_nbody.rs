extern crate piston_window;
extern crate axgeom;
extern crate num;
extern crate rand;
extern crate dinotree;
extern crate ordered_float;

use piston_window::*;

mod support;
use ordered_float::NotNaN;
use axgeom::Rect;
use dinotree::*;
use dinotree::support::*;
use support::*;


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
impl NodeMassTrait for Bla{
    type T=BBox<NotNaN<f64>,Bot>;
    type No=NodeMass;


    //gravitate this nodemass with another node mass
    fn handle_node_with_node(&self,a:&mut Self::No,b:&mut Self::No){
        gravity::gravitate(a,b);
    }

    //gravitate a bot with a bot
    fn handle_bot_with_bot(&self,a:&mut Self::T,b:&mut Self::T){
        gravity::gravitate(&mut a.val,&mut b.val);
    }

    //gravitate a nodemass with a bot
    fn handle_node_with_bot(&self,a:&mut Self::No,b:&mut Self::T){
        gravity::gravitate(a,&mut b.val);
    }
    fn div(self)->(Self,Self){
        (Bla,Bla)
    }


    fn new<'a,I:Iterator<Item=&'a Self::T>> (&'a self,it:I,len:usize)->Self::No{
        let mut total_x=0.0;
        let mut total_y=0.0;
        let mut total_mass=0.0;
        for i in it{
            total_mass+=i.val.mass();
            total_x+=i.val.pos[0];
            total_y+=i.val.pos[1];
        }

        let center=if len!=0{
            [total_x/len as f64,
            total_y/len as f64]
        }else{
            [0.0;2]
        };
        NodeMass{center,mass:total_mass,force:[0.0;2]}
    }


    fn undo<'a,I:Iterator<Item=&'a mut Self::T>> (&'a self,a:&'a Self::No,it:I,len:usize){

        let len_sqr=a.force[0]*a.force[0]+a.force[1]+a.force[1];

        if len_sqr>0.01{

            let total_forcex=a.force[0];
            let total_forcey=a.force[1];

            let forcex=total_forcex/len as f64;
            let forcey=total_forcey/len as f64;

            for i in it{
                i.val.apply_force([forcex,forcey]);
            }
        }else{
            //No acceleration was applied to this node mass.
        }
    }


    fn is_far_enough(&self,a:<Self::T as SweepTrait>::Num,b:<Self::T as SweepTrait>::Num)->bool{
        (a-b).abs()>200.0
    }

    fn is_far_enough_half(&self,a:<Self::T as SweepTrait>::Num,b:<Self::T as SweepTrait>::Num)->bool{
        (a-b).abs()>100.0
    }

}




struct Bot{
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
    fn handle(bots_pruned:&mut [BBox<NotNaN<f64>,Bot>]){
        for bot in bots_pruned.iter_mut(){
            let b=&mut bot.val;

            b.pos[0]+=b.vel[0];
            b.pos[1]+=b.vel[1];
        
            Self::wrap_position(&mut b.pos);

            //F=MA
            //A=F/M
            let accx=b.force[0]/b.mass;
            let accy=b.force[1]/b.mass;

            b.vel[0]+=accx;
            b.vel[1]+=accy;            

            let r=20.0f64.min(b.mass.sqrt()/10.0);
            let x1=b.pos[0]-r;
            let x2=b.pos[0]+r;
            let y1=b.pos[1]-r;
            let y2=b.pos[1]+r;
            let mut rect=Rect::new(x1,x2,y1,y2);
            bot.rect.0=support::rectf64_to_notnan(rect);                

            b.force=[0.0;2];
        }        
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



use gravity::GravityTrait;
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
            a.apply_force([1.0,00.0]);
            b.apply_force([-1.0,0.0]);
        }
    }
}


fn main() {

    let mut bots=create_bots_f64(|id,pos|{
        let velx=((id as isize%3)-1) as f64;
        let vely=(((id+1) as isize % 3)-1) as f64;
        Bot{pos,vel:[velx,vely],force:[0.0;2],mass:20.0}
    },&[0,800,0,800],5000,[1,2]);
    let mut last_bot_with_mass=bots.len();
   
    let mut window: PistonWindow = WindowSettings::new("dinotree test", [800, 800])
        .exit_on_esc(true)
        .build()
        .unwrap();

    let mut cursor=[0.0,0.0];
    while let Some(e) = window.next() {
        e.mouse_cursor(|x, y| {
            cursor = [x, y];
        });
        if let Some(Button::Mouse(_button)) = e.press_args() {

            if last_bot_with_mass<bots.len(){
                let b=&mut bots[last_bot_with_mass];
                b.val.mass=80.0;
                b.val.pos[0]=cursor[0];
                b.val.pos[1]=cursor[1];
                b.val.force=[0.0;2];
                b.val.vel=[0.0;2];

                last_bot_with_mass+=1;
                println!("added bot");
            }else{
                println!("already maxxed");
            }
        }
        window.draw_2d(&e, |c, g| {
            clear([1.0; 4], g);

            {
                let bots_pruned=&mut bots[0..last_bot_with_mass];
                Bot::handle(bots_pruned);


                for bot in bots_pruned.iter(){
                    let ((x1,x2),(y1,y2))=bot.rect.get();
                    let arr=[x1.into_inner() as f64,y1.into_inner() as f64,x2.into_inner() as f64,y2.into_inner() as f64];
                    let square = [arr[0],arr[1],arr[2]-arr[0],arr[3]-arr[1]];                    
                    rectangle([0.0,0.0,0.0,1.0], square, c.transform, g);
                }
                
                
                /*
                for i in 0..bots_pruned.len(){
                    let b1=&mut bots_pruned[i] as *mut BBox<NotNaN<f64>,Bot>;
                    for j in i+1..bots_pruned.len(){
                        let b1=unsafe{&mut *b1};
                        let b2=&mut bots_pruned[j];
                        NodeMass::handle_bot(b1,b2);
                    }
                }
                
                
                let forces_control:Vec<[f64;2]>=bots_pruned.iter().map(|b|{b.val.force}).collect();


                for b in bots_pruned.iter_mut(){
                    b.val.force=[0.0;2];
                }*/
                
                
                
                {
                    let mut tree = DinoTree::new(bots_pruned, StartAxis::Xaxis);
        
                    tree.n_body(Bla);
                                
                    tree.intersect_every_pair_seq(|a, b| {
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
                        }
                        //a.inner.vel[0]=(a.inner.vel[0]+b.inner.vel[0])/2.0;
                        //a.inner.vel[1]=(a.inner.vel[0]+b.inner.vel[0])/2.0;
                    });
                    
                }
            }
            
            last_bot_with_mass={
                let bots_pruned=&mut bots[0..last_bot_with_mass];
                let mut last=bots_pruned.len();
                let mut counter=0;
                for _ in 0..bots_pruned.len(){
                    
                    if bots_pruned[counter].val.mass==0.0{
                        last-=1;
                        bots_pruned.swap(counter,last);
                    }else{
                        counter+=1;
                    }
                }
                assert!(counter==last);

                for (ii,i) in bots_pruned[0..last].iter().enumerate(){
                    assert!(i.val.mass!=0.0,"i:{:?}  val={:?}",ii,i.val.mass);
                }

                for (ii,i) in bots_pruned[last..].iter().enumerate(){
                    assert!(i.val.mass==0.0,"i:{:?}  val={:?}",ii,i.val.mass);
                }
                
                last
            };

            {
                let seed:&[usize]=&[40,20];
                let mut rng:rand::StdRng =  rand::SeedableRng::from_seed(seed);
                let xdist = rand::distributions::Range::new(0,800);
                let vdist = rand::distributions::Range::new(-1,1);
        
                if last_bot_with_mass<bots.len(){
                    //for i in 0..(bots.len()-last_bot_with_mass){
                        let b=&mut bots[last_bot_with_mass];
                        b.val.mass=10.0;

                        use rand::distributions::IndependentSample;
                        let x1=xdist.ind_sample(&mut rng);
                        let y1=xdist.ind_sample(&mut rng);
                        b.val.pos[0]=x1 as f64;
                        b.val.pos[1]=y1 as f64;
                        b.val.force=[0.0;2];
                        let v1=vdist.ind_sample(&mut rng);
                        let v2=vdist.ind_sample(&mut rng);
                        b.val.vel=[v1 as f64,v2 as f64];
                        last_bot_with_mass+=1;
                    //}
                }
            }


            
            /*
            let forces:Vec<[f64;2]>=bots_pruned.iter().map(|b|{b.val.force}).collect();
            
            
            let mut max_err=[0.0f64;2];
            for (i,(a,b)) in forces.iter().zip(forces_control.iter()).enumerate(){
                let diffx=(a[0]-b[0]).abs();
                let diffy=(a[1]-b[1]).abs();
                max_err[0]=max_err[0].max(diffx);
                max_err[1]=max_err[1].max(diffy);
                //assert!(diffx+diffy<0.1,"mismatch:diff{:?}",(i,(diffx,diffy)));
            }
            println!("max err sum={:?}",max_err[0]+max_err[1]);
            */
            
            
            
            

        });
    }
}
