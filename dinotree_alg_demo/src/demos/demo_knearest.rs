use support::prelude::*;
use dinotree::k_nearest;
use dinotree_geom;
pub struct KnearestDemo{
    tree:DynTree<axgeom::XAXISS,(),BBox<F64n,([f64;2],[f64;2])>>
}
impl KnearestDemo{
    pub fn new(dim:[f64;2])->KnearestDemo{
        let dim=&[0,dim[0] as isize,0,dim[1] as isize];
        let radius=[5,20];
        let velocity=[1,3];
        let bots:Vec<([f64;2],[f64;2])>=create_world_generator(500,dim,radius,velocity).map(|ret|{
            let p=ret.pos;
            let r=ret.radius;
            (p,r)
            //BBox::new(Conv::from_rect(aabb_from_pointf64(p,r)),())
        }).collect();

        let tree = DynTree::new(axgeom::XAXISS,(),&bots,|(p,r)|{Conv::from_rect(aabb_from_pointf64(*p,*r))});
        KnearestDemo{tree}
    }
}

impl DemoSys for KnearestDemo{
    fn step(&mut self,cursor:[f64;2],c:&piston_window::Context,g:&mut piston_window::G2d){
        let tree=&self.tree;

        for bot in tree.iter_every_bot(){
            draw_rect_f64n([0.0,0.0,0.0,0.3],bot.get(),c,g);
        }

        #[derive(Copy,Clone,Ord,Eq,PartialEq,PartialOrd,Debug)]
        struct DisSqr(F64n);
        struct Kn<'a,'c:'a>{
            c:&'a Context,
            g:&'a mut G2d<'c>,
        };

        impl<'a,'c:'a> k_nearest::Knearest for Kn<'a,'c>{
            type T=BBox<F64n,([f64;2],[f64;2])>;
            type N=F64n;
            type D=DisSqr;
            fn twod_check(&mut self, point:[Self::N;2],bot:&Self::T)->Self::D{
                
                draw_rect_f64n([0.0,0.0,1.0,0.5],bot.get(),self.c,self.g);
                
                DisSqr(f64n!(dinotree_geom::distance_squared_point_to_rect(Conv::point_to_inner(point),&Conv::rect_to_inner(*bot.get()))))
            }

            fn oned_check(&mut self,p1:Self::N,p2:Self::N)->Self::D{
                let p1=p1.into_inner();
                let p2=p2.into_inner();
                DisSqr(f64n!((p2-p1)*(p2-p1)))
            }

            //create a range around n.
            fn create_range(&mut self,b:Self::N,d:Self::D)->[Self::N;2]{
                let b=b.into_inner();
                let dis=d.0.into_inner().sqrt();
                [f64n!(b-dis),f64n!(b+dis)]
            }
        }

        let mut vv:Vec<(&BBox<F64n,([f64;2],[f64;2])>,DisSqr)>=Vec::new();
        {
            let kn=Kn{c:&c,g};
            let point=[f64n!(cursor[0]),f64n!(cursor[1])];
            k_nearest::k_nearest(&tree,point,3,kn,|a,b|{vv.push((a,b))});
        }

        let cols=[
            [1.0,0.0,0.0,0.8], //red closest
            [0.0,1.0,0.0,0.8], //green second closest
            [0.0,0.0,1.0,0.8]  //blue third closets
        ];
        
        for ((a,_dis),color) in vv.iter().zip(cols.iter()){
            draw_rect_f64n(*color,a.get(),c,g);
        } 
    }   
}

