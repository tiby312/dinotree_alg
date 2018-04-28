



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
        self.acc[0]+=a[0]/self.mass;
        self.acc[1]+=a[1]/self.mass;
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
        //false
        distance_sqr_from(&self.rect,&rectnotnan_to_f64(*b))>100.0*100.0
    }
    fn get_box(&self)->Rect<<Self::T as SweepTrait>::Num>{
        rectf64_to_notnan(self.rect)
    }
    fn undo(&self,b:&mut [Self::T]){
        let mass_per_bot=self.mass/(self.numbots as f64);


        let len_sqr=self.acc[0]*self.acc[0]+self.acc[1]+self.acc[1];

        if len_sqr>0.01{
            let len=len_sqr.sqrt();
            //TODO or something to this effect???
            //can be optimized

            let mag=mass_per_bot/len;
            let forcex=self.acc[0]*mag;
            let forcey=self.acc[1]*mag;

            for i in b.iter_mut(){
                i.val.apply_force([forcex,forcey]);
            }
        }else{
            //No acceleration was applied to this node mass.
        }
    }
}




const BOT_MASS:f64=1.0;
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
        self.acc[0]+=a[0]/BOT_MASS;
        self.acc[1]+=a[1]/BOT_MASS;
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
        let diffy=p2[1]-p1[1];
        let dis_sqr=diffx*diffx+diffy*diffy;

        if dis_sqr>0.01{
            const GRAVITY_CONSTANT:f64=5.0;

            //newtons law of gravitation
            let force=GRAVITY_CONSTANT*(m1*m2)/dis_sqr;

            //clamp the gravity to not be too extreme if two bots are extremly close together
            let force=force.min(1.0);

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


    //Closest point in rectb to the top left of recta. 
    //let xx=num::clamp(ax1,bx1,bx2);
    //let yy=num::clamp(ay1,by1,by2);

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


fn main() {

    let mut bots=create_bots_f64(|id,pos|Bot{pos,vel:[0.0;2],acc:[0.0;2]},&[0,800,0,800],500,[2,20]);

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

            for bot in bots.iter_mut(){
                let b=&mut bot.val;

                b.pos[0]+=b.vel[0];
                b.pos[1]+=b.vel[1];
                b.vel[0]+=b.acc[0];
                b.vel[1]+=b.acc[1];


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
                bot.rect.0=rectf64_to_notnan(rect);

                b.acc=[0.0;2];
            }
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


            for bot in bots.iter(){
                let p1x=bot.val.pos[0];
                let p1y=bot.val.pos[1];
                let p2x=p1x+bot.val.acc[0]*200.0;
                let p2y=p1y+bot.val.acc[1]*200.0;

                //println!("acc={:?}",bot.val.acc);
                let arr=[p1x,p1y,p2x,p2y];
                line([0.0, 0.0, 0.0, 0.4], // black
                     1.0, // radius of line
                     arr, // [x0, y0, x1,y1] coordinates of line
                     c.transform,
                     g);
            }


        });
    }
}
