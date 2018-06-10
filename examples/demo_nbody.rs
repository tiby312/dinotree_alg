extern crate piston_window;
extern crate axgeom;
extern crate num;
extern crate rand;
extern crate dinotree;
extern crate ordered_float;
extern crate dinotree_inner;
extern crate rayon;
use piston_window::*;

mod support;
use ordered_float::NotNaN;
use axgeom::Rect;
use dinotree::*;
use dinotree::support::*;
use dinotree_inner::*;
use support::*;

use dinotree::nbody;


mod knearf64{
    use super::*;
    #[derive(Copy,Clone,Ord,Eq,PartialEq,PartialOrd,Debug)]
    pub struct DisSqr(NotNaN<f64>);
    pub struct Kn;

    impl k_nearest::Knearest for Kn{
        type T=BBox<NotNaN<f64>,Bot>;
        type N=NotNaN<f64>;
        type D=DisSqr;
        fn twod_check(&mut self, point:[Self::N;2],bot:&Self::T)->Self::D{
            
            let (px,py)=(point[0],point[1]);

            let ((a,b),(c,d))=bot.get().get();

            let xx=num::clamp(px,a,b);
            let yy=num::clamp(py,c,d);

            DisSqr((xx-px)*(xx-px) + (yy-py)*(yy-py))
        }

        fn oned_check(&mut self,p1:Self::N,p2:Self::N)->Self::D{
            DisSqr((p2-p1)*(p2-p1))
        }

        //create a range around n.
        fn create_range(&mut self,b:Self::N,d:Self::D)->[Self::N;2]{
            let dis=d.0.sqrt();
            [b-dis,b+dis]
        }
    }
}
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

    //fn create_empty(&self)->Self::No{
    //}

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
    id:usize,
    pos:[f64;2],
    vel:[f64;2],
    force:[f64;2],
    force_naive:[f64;2],
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
    fn handle(bot:&mut BBoxVisible<NotNaN<f64>,Bot>){
        
        let b=&mut bot.inner;

        b.pos[0]+=b.vel[0];
        b.pos[1]+=b.vel[1];
    
        Self::wrap_position(&mut b.pos);

        //F=MA
        //A=F/M
        let accx=b.force[0]/b.mass;
        let accy=b.force[1]/b.mass;

        b.vel[0]+=accx;
        b.vel[1]+=accy;            

        let r=10.0f64.min(b.mass.sqrt()/10.0);
        let x1=b.pos[0]-r;
        let x2=b.pos[0]+r;
        let y1=b.pos[1]-r;
        let y2=b.pos[1]+r;
        let mut rect=Rect::new(x1,x2,y1,y2);
        bot.rect=support::rectf64_to_notnan(rect);                


        b.force=[0.0;2];
        b.force_naive=[0.0;2];
           
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
            //a.apply_force([1.0,0.0]);
            //b.apply_force([-1.0,0.0]);
        }
    }
}


fn main() {

    let mut bots=create_bots_f64(|id,pos|{
        let velx=((id as isize%3)-1) as f64;
        let vely=(((id+1) as isize % 3)-1) as f64;
        Bot{id,pos,vel:[velx,vely],force:[0.0;2],force_naive:[0.0;2],mass:100.0} //used to be 20
    },&[0,800,0,800],5000,[1,2]);

    //Make one of the bots have a lot of mass.
    bots.last_mut().unwrap().inner.mass=10000.0;


    //let mut last_bot_with_mass=bots.len();
    let mut no_mass_bots:Vec<BBoxVisible<NotNaN<f64>,Bot>>=Vec::new();

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

            /*
            match no_mass_bots.pop(){
                Some(mut b)=>{
                    b.inner.mass=80.0;
                    b.inner.pos[0]=cursor[0];
                    b.inner.pos[1]=cursor[1];
                    b.inner.force=[0.0;2];
                    b.inner.vel=[0.0;2];
                    bots.push(b);
                },
                None=>{

                }
            }
            */
        }
        let no_mass_bots=&mut no_mass_bots;
        let bots=&mut bots;
        window.draw_2d(&e, |c, g| {
            clear([1.0; 4], g);

            {


                
                //Do naive solution so we can compare error 
                {
                    dinotree::tools::for_every_pair(bots,|b1,b2|{
                        struct Bo<'a>(&'a mut Bot);
                        impl<'a> GravityTrait for Bo<'a>{
                            fn pos(&self)->[f64;2]{
                                self.0.pos
                            }
                            fn mass(&self)->f64{
                                self.0.mass
                            }
                            fn apply_force(&mut self,a:[f64;2]){
                                self.0.force_naive[0]+=a[0];
                                self.0.force_naive[1]+=a[1];
                            }
                        }
                        gravity::gravitate(&mut Bo(&mut b1.inner),&mut Bo(&mut b2.inner)); 
                    });
                }


                //TODO store bots with no mass inteh front instead?
                let n=NodeMass{center:[0.0;2],mass:0.0,force:[0.0;2]};
    
                let mut tree = DynTree::new(axgeom::XAXISS,n,bots.drain(..).map(|a|a.into_bbox()));
                
                
                
                nbody::nbody_par(&mut tree,Bla);
                
                let mut tree=tree.with_extra(());                

                
                {//Bla
                    use rayon::prelude::*;
                    let bots:Vec<&BBox<NotNaN<f64>,Bot>>=tree.iter().collect();
                    


                    let lines=bots.par_iter().map(|b|{
                        let mut vv:Vec<(&BBox<NotNaN<f64>,Bot>,knearf64::DisSqr)>=Vec::new();
                        {
                            let mut kn=knearf64::Kn;
                            //let cursor=[NotNaN::new(cursor[0]).unwrap(),NotNaN::new(cursor[1]).unwrap()];
                            let pp=[NotNaN::new(b.inner.pos[0]).unwrap(),NotNaN::new(b.inner.pos[1]).unwrap()];
                            k_nearest::k_nearest(&tree,pp,2,kn,|a,b|{vv.push((a,b))});
                        }

                        let b1=&b.inner;
                        let b2=&vv[1].0.inner;
                        (b1,b2)
                    }).fold(||Vec::new(),|mut vec1,item|{vec1.push(item);vec1}).reduce(||Vec::new(),|mut a,mut b|{a.append(&mut b);a});

                    for (b,b2) in lines.iter(){
                        let arr=[b.pos[0] ,b.pos[1],b2.pos[0],b2.pos[1]];
                        line([1.0, 0.0, 1.0, 0.2], // black
                             1.0, // radius of line
                             arr, // [x0, y0, x1,y1] coordinates of line
                             c.transform,
                             g);
                    }

                    /*
                    for b in bots.par_iter(){
                        let mut vv:Vec<(&BBox<NotNaN<f64>,Bot>,knearf64::DisSqr)>=Vec::new();
                        {
                            let mut kn=knearf64::Kn{c:&c,g};
                            //let cursor=[NotNaN::new(cursor[0]).unwrap(),NotNaN::new(cursor[1]).unwrap()];
                            let pp=[NotNaN::new(b.inner.pos[0]).unwrap(),NotNaN::new(b.inner.pos[1]).unwrap()];
                            k_nearest::k_nearest(&tree,pp,2,kn,|a,b|{vv.push((a,b))});
                        }

                        let b=&b.inner;
                        let b2=&vv[1].0.inner;

                        let arr=[b.pos[0] ,b.pos[1],b2.pos[0],b2.pos[1]];
                        line([1.0, 0.0, 1.0, 0.2], // black
                             1.0, // radius of line
                             arr, // [x0, y0, x1,y1] coordinates of line
                             c.transform,
                             g);
                    }
                    */
                
                }

                /*
                iter_mut_special(&mut tree,|bot1,tree|{
                    let mut counter=0;
                    tree.k_nearest(bot.point,2,Knear,|bot2|{
                        if(counter==1){
                            //draw_line(bot,bot);
                        }
                        counter+=1;
                    })
                });
                */

              
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
                        //cap the mass!
                        //a.inner.mass=a.inner.mass.min(80000.0);
                        
                        a.inner.force[0]+=b.inner.force[0];
                        a.inner.force[1]+=b.inner.force[1];
                        a.inner.force_naive[0]+=b.inner.force_naive[0];
                        a.inner.force_naive[1]+=b.inner.force_naive[1];
                        a.inner.vel[0]=vx;
                        a.inner.vel[1]=vy;


                        b.inner.mass=0.0;
                        b.inner.force[0]=0.0;
                        b.inner.force[1]=0.0;
                        b.inner.force_naive[0]=0.0;
                        b.inner.force_naive[1]=0.0;
                        b.inner.vel[0]=0.0;
                        b.inner.vel[1]=0.0;
                        b.inner.pos[0]=0.0;
                        b.inner.pos[1]=0.0;
                    }
                });


                
                
                for b in tree.into_iter_orig_order(){
                    bots.push(b.into_visible());
                }
                

            }
            
            {
                let mut new_bots=Vec::new();
                for b in bots.drain(..){
                    if b.inner.mass==0.0{
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

            //Draw bots.
            for bot in bots.iter(){
                let mut max_mag=0.0f64;
                let mag={
                    let b=&bot.inner;
                    let x=b.force[0]-b.force_naive[0];
                    let y=b.force[1]-b.force_naive[1];
                    
                    let dis=x*x+y*y;
                    dis.sqrt()/b.mass //The more mass an object has, the less impact error has
                };
                max_mag=max_mag.max(mag);
                let mag=mag*100.0;
                let ((x1,x2),(y1,y2))=bot.rect.get();
                let arr=[x1.into_inner() as f64,y1.into_inner() as f64,x2.into_inner() as f64,y2.into_inner() as f64];
                let square = [arr[0],arr[1],arr[2]-arr[0],arr[3]-arr[1]];                    
                rectangle([mag as f32,0.0,0.0,1.0], square, c.transform, g);
            }

            //Add some bots
            {
                let seed:&[usize]=&[40,20];
                let mut rng:rand::StdRng =  rand::SeedableRng::from_seed(seed);
                let xdist = rand::distributions::Range::new(0,800);
                let vdist = rand::distributions::Range::new(-1,1);
        
                match no_mass_bots.pop(){
                    Some(mut b)=>{
                            //for _ in 0..bots.len()-last_bot_with_mass{
                        //for _ in 0..2{
                            b.inner.mass=10.0;

                            use rand::distributions::IndependentSample;
                            let x1=xdist.ind_sample(&mut rng);
                            let y1=xdist.ind_sample(&mut rng);
                            b.inner.pos[0]=x1 as f64;
                            b.inner.pos[1]=y1 as f64;
                            b.inner.force=[0.0;2];
                            {
                                let r=10.0f64.min(b.inner.mass.sqrt()/10.0);
                                let x1=b.inner.pos[0]-r;
                                let x2=b.inner.pos[0]+r;
                                let y1=b.inner.pos[1]-r;
                                let y2=b.inner.pos[1]+r;
                                let mut rect=Rect::new(x1,x2,y1,y2);
                                b.rect=support::rectf64_to_notnan(rect);                
                            }
                            let v1=vdist.ind_sample(&mut rng);

                            let v2=vdist.ind_sample(&mut rng);
                            b.inner.vel=[v1 as f64,v2 as f64];
                            bots.push(b);
                        //}
                    },
                    None=>{

                    }
                
                    
                }               
            }
        });
    }
}
