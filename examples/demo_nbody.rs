



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
    numbots:usize,
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
        //println!("mass={:?}",total_mass);
        //let total_mass*==3.0;
        let center=if len!=0{
            [total_x/len as f64,
            total_y/len as f64]
        }else{
            [0.0;2]
        };
        NodeMass{center,numbots:len,mass:total_mass,force:[0.0;2]}
    }


    fn undo<'a,I:Iterator<Item=&'a mut Self::T>> (&self,it:I,len:usize) where Self::T:'a{
        //F=ma

        assert_eq!(len,self.numbots);
        //let mass_per_bot=self.mass/(self.numbots as f64);


        let len_sqr=self.force[0]*self.force[0]+self.force[1]+self.force[1];

        if len_sqr>0.01{

            let dis=len_sqr.sqrt();
            let total_forcex=self.force[0];
            let total_forcey=self.force[1];


            //TODO or something to this effect???
            //can be optimized

            //let mag=mass_per_bot/dis;
            //let forcex=self.acc[0]*mag;
            //let forcey=self.acc[1]*mag;
            let forcex=total_forcex/self.numbots as f64;
            let forcey=total_forcey/self.numbots as f64;

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

    //depth is the depth of the anchor.
    //its possible the bots belong to the same node?
    fn is_far_enough(depth:usize,a:<Self::T as SweepTrait>::Num,b:<Self::T as SweepTrait>::Num)->bool{
        //false
        //(1+depth) as f64*(a-b).abs()>2000.0
        (a-b).abs()>200.0
    }

    fn is_far_enough_half(depth:usize,a:<Self::T as SweepTrait>::Num,b:<Self::T as SweepTrait>::Num)->bool{
        //false
        //(1+depth) as f64*(a-b).abs()>1000.0
        (a-b).abs()>100.0
    }

}




const BOT_MASS:f64=10.0;
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
            let dis=dis_sqr.sqrt();

            const GRAVITY_CONSTANT:f64=0.002;

            //newtons law of gravitation (modified for 2d??? divide by len instead of sqr)
            let force=GRAVITY_CONSTANT*(m1*m2)/dis_sqr;

            //clamp the gravity to not be too extreme if two bots are extremly close together
            //let force=force.min(1000.0);

            let dis=dis_sqr.sqrt();
            let finalx=diffx*(force/dis);
            let finaly=diffy*(force/dis);
            
            a.apply_force([finalx,finaly]);
            b.apply_force([-finalx,-finaly]);
        }else{
            //TODO handle this case
        }
    }
}


fn distance_sqr_from(recta:&Rect<f64>,rectb:&Rect<f64>)->f64{
    let ((ax1,ax2),(ay1,ay2))=recta.get();
    let ((bx1,bx2),(by1,by2))=rectb.get();

    //This describes the outer rectangle.
    //https://gamedev.stackexchange.com/questions/154036/efficient-minimum-distance-between-two-axis-aligned-squares
    let rx1=ax1.min(bx1);
    let rx2=ax2.max(bx2);
    let ry1=ay1.min(by1);
    let ry2=ay2.max(by2);

    //inner_width = max(0, rect_outer.width - square_a.width - square_b.width)
    //inner_height = max(0, rect_outer.height - square_a.height - square_b.height)

    //negative if rectangles are touching, in which case, we want
    //the distance to just be zero.
    let inner_width= 0.0f64.max( (rx2-rx1)-(ax2-ax1)-(bx2-bx1));
    let inner_height=0.0f64.max( (ry2-ry1)-(ay2-ay1)-(by2-by1));

    //return the squre
    return inner_width*inner_width+inner_height*inner_height;
}
fn rectf64_to_notnan(rect:Rect<f64>)->Rect<NotNaN<f64>>{
    let ((a,b),(c,d))=rect.get();

    Rect::new(NotNaN::new(a).unwrap(),NotNaN::new(b).unwrap(),NotNaN::new(c).unwrap(),NotNaN::new(d).unwrap())
}
fn rectnotnan_to_f64(rect:Rect<NotNaN<f64>>)->Rect<f64>{
    let ((a,b),(c,d))=rect.get();
    Rect::new(a.into_inner(),b.into_inner(),c.into_inner(),d.into_inner())
}


fn clamp_position(a:&mut [f64;2]){
    a[0]=num::clamp(a[0],0.0,800.0);
    a[1]=num::clamp(a[1],0.0,800.0);
}
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

fn test_nodemass(){
    let mut b1=create_bots_f64(|id,pos|Bot{pos,vel:[0.0;2],force:[0.0;2],mass:BOT_MASS},&[0,100,0,100],50,[2,20]);
    let mut b2=create_bots_f64(|id,pos|Bot{pos,vel:[0.0;2],force:[0.0;2],mass:BOT_MASS},&[800,900,800,900],50,[2,20]);

    let control={
        for i in b1.iter_mut(){
            for j in b2.iter_mut(){
                NodeMass::handle_bot(i,j);
            }
        }

        let control:Vec<[f64;2]> =b1.iter().chain(b2.iter()).map(|a|a.val.force).collect();
        for b in b1.iter_mut().chain(b2.iter_mut()){
            b.val.force=[0.0;2]
        }
        control
    };


    let test={
        let mut n1=NodeMass::new(b1.iter(),b1.len());
        let mut n2=NodeMass::new(b2.iter(),b2.len());

        n1.handle_with(&mut n2);
        

        let b1len=b1.len();
        let b2len=b2.len();
        n1.undo(b1.iter_mut(),b1len);
        n2.undo(b2.iter_mut(),b2len);

        let test:Vec<[f64;2]>=b1.iter().chain(b2.iter()).map(|a|a.val.force).collect();
        for b in b1.iter_mut().chain(b2.iter_mut()){
            b.val.force=[0.0;2]
        }
        test
    };

    for (a,b) in control.iter().zip(test.iter()){
        let diffx=(a[0]-b[0]).abs();
        let diffy=(a[1]-b[1]).abs();
        println!("diff={:?}",(diffx,diffy));
    }

    //one list of bots.
    //second list of bots.

    //handle as node masses



}

use std::time::Instant;
fn main() {


    let mut bots=create_bots_f64(|id,pos|{
        let velx=((id as isize%3)-1) as f64;
        let vely=(((id+1) as isize % 3)-1) as f64;
        Bot{pos,vel:[velx,vely],force:[0.0;2],mass:BOT_MASS}
    },&[0,800,0,800],5000,[1,2]);
    let mut bots_pruned:&mut [BBox<NotNaN<f64>,Bot>]=&mut bots;



    let mut window: PistonWindow = WindowSettings::new("dinotree test", [800, 800])
        .exit_on_esc(true)
        .build()
        .unwrap();

    let mut cursor=[0.0,0.0];
    while let Some(e) = window.next() {
        e.mouse_cursor(|x, y| {
            cursor = [x, y];
        });
        if let Some(Button::Mouse(button)) = e.press_args() {
            println!("Pressed mouse button '{:?}'", button);
        }
        window.draw_2d(&e, |c, g| {
            clear([1.0; 4], g);

            for bot in bots_pruned.iter_mut(){
                let b=&mut bot.val;

                b.pos[0]+=b.vel[0];
                b.pos[1]+=b.vel[1];
            

                wrap_position(&mut b.pos);

                //b.vel[0]*=0.99;
                //b.vel[1]*=0.99;
                
                assert!(!b.mass.is_nan(),"mass nan!");
                
                
                {
                    //F=MA
                    //A=F/M

                    let accx=b.force[0]/b.mass;
                    let accy=b.force[1]/b.mass;

                    b.vel[0]+=accx;
                    b.vel[1]+=accy;
                
                }

                assert!(!b.pos[0].is_nan(),"xpos nan!");
                assert!(!b.pos[1].is_nan(),"ypos nan!");
                
                /*
                let mut rect=rectnotnan_to_f64(bot.rect.0);

                {
                    let r1=rect.get_range2_mut::<axgeom::XAXISS>();
                    let width=r1.end-r1.start;

                    r1.start=b.pos[0]-width/2.0;
                    r1.end=b.pos[0]+width/2.0;                
                }
                {
                    let r2=rect.get_range2_mut::<axgeom::YAXISS>();
                    let height=r2.end-r2.start;

                    r2.start=b.pos[1]-height/2.0;
                    r2.end=b.pos[1]+height/2.0;
                }
                */
                let r=b.mass.sqrt()/10.0;
                //println!("radius={:?}",r);
                let x1=b.pos[0]-r;
                let x2=b.pos[0]+r;
                let y1=b.pos[1]-r;
                let y2=b.pos[1]+r;
                let mut rect=Rect::new(x1,x2,y1,y2);
                bot.rect.0=rectf64_to_notnan(rect);
                

                b.force=[0.0;2];
            }
            for bot in bots_pruned.iter(){
                let ((x1,x2),(y1,y2))=bot.rect.get();
                /*
                let pos=bot.val.pos;//get();

                let r=bot.val.mass.sqrt()/20.0;
                println!("radius={:?}",r);
                let x1=pos[0]-r;
                let x2=pos[0]+r;
                let y1=pos[1]-r;
                let y2=pos[1]+r;
               */

                let arr=[x1.into_inner() as f64,y1.into_inner() as f64,x2.into_inner() as f64,y2.into_inner() as f64];
                //let square = rectangle::square(x1.into_inner() as f64, y1.into_inner() as f64, bot.val.mass.sqrt()*2.0);
                let square = [arr[0],arr[1],arr[2]-arr[0],arr[3]-arr[1]];
                  
                //let square = [x1,y1,x2-x1,y2-y1];
                        
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
            */
            /*
            let forces_control:Vec<[f64;2]>=bots.iter().map(|b|{b.val.force}).collect();


            for b in bots.iter_mut(){
                b.val.force=[0.0;2];
            }
            */
            
            
            {
                let mut tree = DinoTree::new(bots_pruned, StartAxis::Xaxis);

                
                let k={
                    let rect=rectf64_to_notnan(Rect::new(0.0,800.0,0.0,800.0));
                    tree.n_body::<NodeMass>();
                };
                

                
                tree.intersect_every_pair_seq(|a, b| {
                    let (a,b)=if a.inner.mass>b.inner.mass{
                        (a,b)
                    }else{
                        (b,a)
                    };

                    a.inner.mass+=b.inner.mass;
                    a.inner.force[0]+=b.inner.force[0];
                    a.inner.force[1]+=b.inner.force[1];
                    b.inner.mass=0.0;
                    b.inner.force[0]=0.0;
                    b.inner.force[1]=0.0;
                
                });
            }

            let last_bot_with_mass={
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
            let bb=std::mem::replace(&mut bots_pruned,&mut []);
            std::mem::replace(&mut bots_pruned,&mut bb[0..last_bot_with_mass]);
            //println!("len={:?}",bots_pruned.len());
            
            /*
            let forces:Vec<[f64;2]>=bots.iter().map(|b|{b.val.force}).collect();
            
            
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
            
            
            
            /*  
            for bot in bots.iter(){
                let p1x=bot.val.pos[0];
                let p1y=bot.val.pos[1];
                let p2x=p1x+bot.val.acc[0]*2000.0;
                let p2y=p1y+bot.val.acc[1]*2000.0;

                //println!("acc={:?}",bot.val.acc);
                let arr=[p1x,p1y,p2x,p2y];
                line([0.0, 0.0, 0.0, 0.4], // black
                     1.0, // radius of line
                     arr, // [x0, y0, x1,y1] coordinates of line
                     c.transform,
                     g);
            }
            */
            

        });
    }
}
