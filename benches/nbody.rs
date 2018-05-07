#![feature(test)]

extern crate piston_window;
extern crate axgeom;
extern crate num;
extern crate rand;
extern crate dinotree;
extern crate ordered_float;
extern crate test;

use piston_window::*;

mod support;
use ordered_float::NotNaN;
use axgeom::Rect;
use dinotree::*;
use dinotree::support::*;
use support::*;
use test::*;

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

impl NodeMassTrait for NodeMass{
    type T=BBox<NotNaN<f64>,Bot>;
    fn handle_with(&mut self,b:&mut Self){
        gravity::gravitate(self,b);
    }
    fn handle_bot(a:&mut Self::T,b:&mut Self::T){
        gravity::gravitate(&mut a.val,&mut b.val);
    }
    fn new<'a,I:Iterator<Item=&'a Self::T>> (it:I,len:usize)->Self where Self::T:'a{
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


    fn undo<'a,I:Iterator<Item=&'a mut Self::T>> (&self,it:I,len:usize) where Self::T:'a{

        let len_sqr=self.force[0]*self.force[0]+self.force[1]+self.force[1];

        if len_sqr>0.01{

            let total_forcex=self.force[0];
            let total_forcey=self.force[1];

            let forcex=total_forcex/len as f64;
            let forcey=total_forcey/len as f64;

            for i in it{
                i.val.apply_force([forcex,forcey]);
            }
        }else{
            //No acceleration was applied to this node mass.
        }
    }

    fn apply(&mut self,b:&mut Self::T){
        gravity::gravitate(self,&mut b.val);
    }

    fn is_far_enough(a:<Self::T as SweepTrait>::Num,b:<Self::T as SweepTrait>::Num)->bool{
        (a-b).abs()>200.0
    }

    fn is_far_enough_half(a:<Self::T as SweepTrait>::Num,b:<Self::T as SweepTrait>::Num)->bool{
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


//TODO talk about how these benches dont really mean much since they do not test test different distributions of bots.

#[bench]
fn nbody_naive(bench:&mut Bencher){
    let mut bots=create_bots_f64(|id,pos|{
        let velx=((id as isize%3)-1) as f64;
        let vely=(((id+1) as isize % 3)-1) as f64;
        Bot{pos,vel:[velx,vely],force:[0.0;2],mass:20.0}
    },&[0,800,0,800],5000,[1,2]);
   
    bench.iter(||{
        for i in 0..bots.len(){
            let b1=&mut bots[i] as *mut BBox<NotNaN<f64>,Bot>;
            for j in i+1..bots.len(){
                let b1=unsafe{&mut *b1};
                let b2=&mut bots[j];
                NodeMass::handle_bot(b1,b2);
            }
        }    
    });
    
}

#[bench]
fn nbody_seq(bench:&mut Bencher) {

    let mut bots=create_bots_f64(|id,pos|{
        let velx=((id as isize%3)-1) as f64;
        let vely=(((id+1) as isize % 3)-1) as f64;
        Bot{pos,vel:[velx,vely],force:[0.0;2],mass:20.0}
    },&[0,800,0,800],5000,[1,2]);
    

    let mut tree = DinoTree::new(&mut bots, StartAxis::Xaxis);
                
    bench.iter(||{
        tree.n_body_seq::<NodeMass>();
    }); 

    black_box(tree);      
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
            
}
#[bench]
fn nbody_par(bench:&mut Bencher) {

    let mut bots=create_bots_f64(|id,pos|{
        let velx=((id as isize%3)-1) as f64;
        let vely=(((id+1) as isize % 3)-1) as f64;
        Bot{pos,vel:[velx,vely],force:[0.0;2],mass:20.0}
    },&[0,800,0,800],5000,[1,2]);
    

    let mut tree = DinoTree::new(&mut bots, StartAxis::Xaxis);
                
    bench.iter(||{
        tree.n_body::<NodeMass>();

    }); 

    black_box(tree);      
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
            
}
