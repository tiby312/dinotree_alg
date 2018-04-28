



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
    acc:[f64;2],
    rect:Rect<f64>
}

impl GravityTrait for NodeMass{
    fn pos(&self)->[f64;2]{
        self.center
    }
    fn mass(&self)->f64{
        BOT_MASS
    }
    fn apply_force(&mut self,a:[f64;2]){
        self.acc[0]+=a[0];
        self.acc[1]+=a[1];
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
    fn new(rect:Rect<NotNaN<f64>>,b:&[Self::T])->Self{
        fn get_center(a:&Rect<f64>)->[f64;2]{
            let x=a.get_range2::<axgeom::XAXISS>();
            let y=a.get_range2::<axgeom::YAXISS>();

            let dx=(x.end-x.start)/2.0;
            let dy=(y.end-y.start)/2.0;
            [x.start+dx,y.start+dy]
        }
        let rect=rectnotnan_to_f64(rect);

        NodeMass{center:get_center(&rect),numbots:b.len(),mass:b.iter().fold(0.0,|a,b|a+b.val.mass()),acc:[0.0;2],rect:rect}
    }
    fn increase_mass(&mut self,b:&[Self::T]){
        for i in b.iter(){
            self.mass+=i.val.mass();
        }
        self.numbots+=b.len();
    }
    fn apply(&mut self,b:&mut Self::T){
        gravity::gravitate(self,&mut b.val);
    }
    fn is_far_enough(&self,b:&Rect<<Self::T as SweepTrait>::Num>)->bool{
        distance_from(&self.rect,&rectnotnan_to_f64(*b))>100.0
    }
    fn get_box(&self)->Rect<<Self::T as SweepTrait>::Num>{
        rectf64_to_notnan(self.rect)
    }
    fn undo(&self,b:&mut [Self::T]){
        let mass_per_bot=self.mass/(self.numbots as f64);

        //TODO or something to this effect???
        //can be optimized

        let mag=(mass_per_bot/self.numbots as f64);
        let forcex=self.acc[0]*mag;
        let forcey=self.acc[1]*mag;

        for i in b.iter_mut(){
            i.val.apply_force([forcex,forcey]);
        }
    }
}




const BOT_MASS:f64=0.005;
struct Bot{
    pos:[f64;2],
    vel:[f64;2],
    acc:[f64;2]
}

impl GravityTrait for Bot{
    fn pos(&self)->[f64;2]{
        self.pos
    }
    fn mass(&self)->f64{
        BOT_MASS
    }
    fn apply_force(&mut self,a:[f64;2]){
        self.acc[0]+=a[0];
        self.acc[1]+=a[1];
    }
}



/*
impl NodeMassTrait for NodeMass{
    type T=BBox<NotNaN<f64>,Bot>;

    fn handle_with(&self,b:&mut Self){
        gravity::gravitate(a,b);
    }
    fn handle_bot(a:&mut Self::T,b:&mut Self::T){
        gravity::gravitate(a,b);
    }
    fn new(rect:&Rect<NotNaN<f64>>,b:&[Self::T])->Self{
        let mass=b.iter().fold(0,|a,b|a+b.mass);
        NodeMass{aabb:rect,center:get_center(rect),mass,moved_amount:[0.0;2]}
    }
    fn apply(&mut self,b:&mut [Self::T]){
        let diffx=a.moved_amount[0];
        let diffy=a.moved_amount[1];

        let dis_sqr=(diffx*diffx)+(diffy*diffy);
        let dis=dis_sqr.sqrt();

        //The magnitude of the vector for each individual bot.
        let bot_dis=dis/b.len();

        //let diffx=diffx*(bot_dis/dis);
        //let diffy=diffy*(bot_dis/dis);
        let ll=b.len() as f64;
        let diffx=diffx/ll;
        let diffy=diffy/ll;


        for i in b{
            b.apply_force([diffx,diffy]);
        }
    }
    fn is_far_enough(&self,b:&NodeMass<Self::N>)->bool{
        //TODO check rect distance, not point.
        let p1=a.point;
        let p2=b.point;

        let mut dis=0;
        for i in 0..2{
            dis+=(p2[i]-p1[i])
        }
        if dis>1000.0{
            true
        }else{
            false
        }
    }
}
*/

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
        let diffy=p2[1]-p2[1];
        let dis_sqr=diffx*diffx+diffy*diffy;

        const GRAVITY_CONSTANT:f64=0.001;

        //newtons law of gravitation
        let force=GRAVITY_CONSTANT*(m1*m2)/dis_sqr;


        let dis=dis_sqr.sqrt();
        let finalx=diffx*(force/dis);
        let finaly=diffx*(force/dis);
        a.apply_force([finalx,finaly]);
        b.apply_force([-finalx,-finaly]);
    }
}


fn distance_from(recta:&Rect<f64>,rectb:&Rect<f64>)->f64{
    unimplemented!();
}
fn rectf64_to_notnan(rect:Rect<f64>)->Rect<NotNaN<f64>>{
    unimplemented!();
}
fn rectnotnan_to_f64(rect:Rect<NotNaN<f64>>)->Rect<f64>{
    unimplemented!();
}


fn main() {

    let mut bots=create_bots_f64(|id|Bot{pos:[0.0;2],vel:[0.0;2],acc:[0.0;2]},&[0,800,0,800],500,[2,20]);

    let mut window: PistonWindow = WindowSettings::new("dinotree test", [800, 800])
        .exit_on_esc(true)
        .build()
        .unwrap();

    let mut cursor=[0.0,0.0];
    while let Some(e) = window.next() {
        e.mouse_cursor(|x, y| {
            cursor = [x, y];
        });

        window.draw_2d(&e, |c, g| {
            clear([1.0; 4], g);

            for bot in bots.iter(){
                let ((x1,x2),(y1,y2))=bot.rect.get();
                let arr=[x1.into_inner() as f64,y1.into_inner() as f64,x2.into_inner() as f64,y2.into_inner() as f64];
                let square = rectangle::square(x1.into_inner() as f64, y1.into_inner() as f64, 8.0);
        
                rectangle([0.0,0.0,0.0,0.3], square, c.transform, g);
            }
            
            {
                let mut tree = DinoTree::new(&mut bots, StartAxis::Xaxis);


                let k={
                    
                    
                    let rect=rectf64_to_notnan(Rect::new(0.0,800.0,0.0,800.0));
                    tree.n_body::<NodeMass>(AABBox(rect));
                    
       
                };
            }
        });
    }
}
