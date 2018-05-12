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
        println!("AAAA={:?}",a);
        self.force[0]+=a[0];
        self.force[1]+=a[1];
    }
}

#[derive(Clone,Copy)]
struct Bla;
impl NodeMassTrait for Bla{
    type T=BBox<NotNaN<f64>,Bot>;
    type No=NodeMass;

    fn create_empty(&self)->Self::No{
        NodeMass{center:[0.0;2],mass:0.0,force:[0.0;2]}
    }

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
            let m=i.val.mass();
            total_mass+=m;
            total_x+=m*i.val.pos[0];
            total_y+=m*i.val.pos[1];
        }

        let center=if len!=0{
            [total_x/total_mass,
            total_y/total_mass]
        }else{
            [0.0;2]
        };
        NodeMass{center,mass:total_mass,force:[0.0;2]}
    }


    fn apply_to_bots<'a,I:Iterator<Item=&'a mut Self::T>> (&'a self,a:&'a Self::No,it:I,len:usize){

        let len_sqr=a.force[0]*a.force[0]+a.force[1]*a.force[1];

        if len_sqr>0.000001{
            println!("AAAAAAAAAAAAAAAAAAAAAAA");
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

    //TODO improve accuracy by relying on depth???
    fn is_far_enough<A:axgeom::AxisTrait>(&self,depth:usize,a:&Self::No,b:[<Self::T as SweepTrait>::Num;2])->bool{
        
        //false
        
        let a=if A::new().is_xaxis(){
            a.center[0]
        }else{
            a.center[1]
        };
        
        
        //let a=b[0];
        let x=(depth+1) as f64;
        (a-b[1].into_inner()).abs()*x>800.0
        
        //false
    }

    fn is_far_enough_half<A:axgeom::AxisTrait>(&self,depth:usize,a:&Self::No,b:[<Self::T as SweepTrait>::Num;2])->bool{
        //false
        //(a-b).abs()>100.0
        
        let a=if A::new().is_xaxis(){
            a.center[0]
        }else{
            a.center[1]
        };
        
        
        //let a=b[0];
        let x=(depth+1) as f64;
        //let a=b[0];
        (a-b[1].into_inner()).abs()*x>400.0
        
        //false
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


        if dis_sqr>0.0000001{
            
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


//mod support;
//use std;

struct Bot{
    pos:[f64;2],
    vel:[f64;2],
    force:[f64;2],
    mass:f64
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

#[test]
fn test_nodemass(){
    let mut b1=support::create_bots_f64(|_id,pos|Bot{pos,vel:[0.0;2],force:[0.0;2],mass:1.0},&[0,1000,0,1000],10,[2,20]);
    let mut b2=support::create_bots_f64(|_id,pos|Bot{pos,vel:[0.0;2],force:[0.0;2],mass:1.0},&[3000,4000,0,1000],10,[2,20]);

    b1.last_mut().unwrap().val.mass=10000000.0;

    let control={
        for i in b1.iter_mut(){
            for j in b2.iter_mut(){
                Bla.handle_bot_with_bot(i,j);
            }
        }

        let control:Vec<[f64;2]> =b1.iter().chain(b2.iter()).map(|a|a.val.force).collect();
        for b in b1.iter_mut().chain(b2.iter_mut()){
            b.val.force=[0.0;2]
        }
        control
    };

    //b1.last_mut().unwrap().val.pos=[5000.0,-5000.0];


    let test={
        let mut n1=Bla.new(b1.iter(),b1.len());
        let mut n2=Bla.new(b2.iter(),b2.len());

        Bla.handle_node_with_node(&mut n1,&mut n2);
        

        let b1len=b1.len();
        let b2len=b2.len();
        Bla.apply_to_bots(&n1,b1.iter_mut(),b1len);
        Bla.apply_to_bots(&n2,b2.iter_mut(),b2len);

        let test:Vec<[f64;2]>=b1.iter().chain(b2.iter()).map(|a|a.val.force).collect();
        for b in b1.iter_mut().chain(b2.iter_mut()){
            b.val.force=[0.0;2]
        }
        test
    };

    for (a,b) in control.iter().zip(test.iter()){
        //println!("control={:?}\t\t\t\ttest={:?}",a,b);
        let diffx=(a[0]-b[0]).abs();
        let diffy=(a[1]-b[1]).abs();
        println!("diff={:?}",(diffx,diffy));
    }
    panic!();
    //one list of bots.
    //second list of bots.

    //handle as node masses



}
