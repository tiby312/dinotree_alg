



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





fn get_center(a:&Rect<NotNaN<f64>>)->[NotNaN<f64>;2]{
    let x=a.get_range2::<axgeom::XAXISS>();
    let y=a.get_range2::<axgeom::YAXISS>();

    let dx=x.last-x.start;
    let dy=y.last-y.start;
    [dx,dy]
}


struct NodeMass{
    center:[f64;2],
    box:Rect<f64>,
    mass:f64,
    acc:[f64;2]
}


//struct NodeMass2<N:NumTrait>(NodeMass<N>);
impl GravityTrait for NodeMass{

    fn pos(&self)->[f64;2]{
        self.center
    }
    fn mass(&self)->f64{
        self.mass
    }
    fn apply_force(&mut self,a:[f64;2]){
        self.moved_amount[0]+=a[0];
        self.moved_amount[1]+=a[1];
    }   
}


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

use gravity::GravityTrait;
mod gravity{
    pub trait GravityTrait{
        fn pos(&self)->[f64;2];
        fn mass(&self)->f64;
        fn apply_force(&mut self,[f64;2]);
    }

    //Returns the force to be exerted to the first object.
    //The force to the second object can be retrieved simply by negating the first.
    pub fn gravitate<T:GravityTrait>(a:&mut T,b:&mut T){
        let p1=a.pos();
        let p2=b.pos();
        let m1=a.mass();
        let m2=b.mass();

        let diffx=p2[0]-p1[0];
        let diffy=p2[1]-p2[1];
        let dis_sqr=diffx*diffx+diffy*diffy;

        const GRAVITY_CONSTANT:f64=0.001;

        //newtons law of gravitation
        let force=GRAVITY_CONSTANT*(m1&m2)/dis_sqr;


        let dis=dis_sqr.sqrt();
        let finalx=diffx*(force/dis);
        let finaly=diffx*(force/dis);
        a.apply_force([finalx,finaly]);
        b.apply_force([-finalx,-finaly]);
    }
}



fn main() {

    let mut bots=create_bots_isize(|id|Bot{id,col:Vec::new()},&[0,800,0,800],500,[2,20]);

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
                let arr=[x1 as f64,y1 as f64,x2 as f64,y2 as f64];
                let square = rectangle::square(x1 as f64, y1 as f64, 8.0);
        
                rectangle([0.0,0.0,0.0,0.3], square, c.transform, g);
            }
            
            {
                let mut tree = DinoTree::new(&mut bots, StartAxis::Xaxis);


                let k={
                    
                    let v={
                        //Compute distance sqr
                        let min_rect=|point:[isize;2],aabb:&AABBox<isize>|{
                            {
                                let ((x1,x2),(y1,y2))=aabb.get();
                            
                                {
                                    let ((x1,x2),(y1,y2))=((x1 as f64,x2 as f64),(y1 as f64,y2 as f64));
                                    let square = [x1,y1,x2-x1,y2-y1];
                                    rectangle([0.0,0.0,0.0,0.5], square, c.transform, g);
                                }
                            }
                            let (px,py)=(point[0],point[1]);

                            let ((a,b),(c,d))=aabb.get();

                            let xx=num::clamp(px,a,b);
                            let yy=num::clamp(py,c,d);

                            (xx-px)*(xx-px) + (yy-py)*(yy-py)
                        };

                        //Compute distance sqr in 1d cases.
                        let min_oned=&|p1:isize,p2:isize|{
                            (p2-p1)*(p2-p1)
                        };


                        let mut v=Vec::new();
                        tree.k_nearest([cursor[0] as isize,cursor[1] as isize],3,|a,dis|{v.push((a,dis))},min_rect,min_oned);
                        v
                    };

                    let cols=[
                        [1.0,0.0,0.0,0.8], //red closest
                        [0.0,1.0,0.0,0.8], //green second closest
                        [0.0,0.0,1.0,0.8]  //blue third closets
                    
                    ];

                    for (i,a) in v.iter().enumerate(){
                        let ((x1,x2),(y1,y2))=a.0.rect.get();
                        
                        {
                            let ((x1,x2),(y1,y2))=((x1 as f64,x2 as f64),(y1 as f64,y2 as f64));
                            let square = [x1,y1,x2-x1,y2-y1];
                            rectangle(cols[i], square, c.transform, g);
                        }
                    }                    
                };
            }
        });
    }
}
