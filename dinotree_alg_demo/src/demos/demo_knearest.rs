use support::prelude::*;
use dinotree::k_nearest;
use dinotree_geom;
pub struct KnearestDemo{
    tree:DynTree<axgeom::XAXISS,(),BBox<f64N,()>>
}
impl KnearestDemo{
    pub fn new(dim:[f64;2])->KnearestDemo{
        
        let dim2=[f64n!(dim[0]),f64n!(dim[1])];
        let dim=&[0,dim[0] as isize,0,dim[1] as isize];
        let radius=[5,20];
        let velocity=[1,3];
        let bots=create_world_generator(500,dim,radius,velocity).map(|ret|{
            let p=ret.pos;
            let r=ret.radius;
            BBox::new(Conv::from_rect(aabb_from_pointf64(p,r)),())
        });

        let tree = DynTree::new(axgeom::XAXISS,(),bots);
        KnearestDemo{tree}
    }
}

impl DemoSys for KnearestDemo{
    fn step(&mut self,cursor:[f64;2],c:&piston_window::Context,g:&mut piston_window::G2d){
        let tree=&self.tree;

        for bot in tree.iter(){
            let ((x1,x2),(y1,y2))=bot.get().get();
            let ((x1,x2),(y1,y2))=((x1.into_inner(),x2.into_inner()),(y1.into_inner(),y2.into_inner()));
            let square = [x1,y1,x2-x1,y2-y1];
                                    
            rectangle([0.0,0.0,0.0,0.3], square, c.transform, g);
        }

        #[derive(Copy,Clone,Ord,Eq,PartialEq,PartialOrd,Debug)]
        struct DisSqr(f64N);
        struct Kn<'a,'c:'a>{
            c:&'a Context,
            g:&'a mut G2d<'c>,
        };

        impl<'a,'c:'a> k_nearest::Knearest for Kn<'a,'c>{
            type T=BBox<f64N,()>;
            type N=f64N;
            type D=DisSqr;
            fn twod_check(&mut self, point:[Self::N;2],bot:&Self::T)->Self::D{
                {
                    let ((x1,x2),(y1,y2))=bot.get().get();
                    
                    {
                        let ((x1,x2),(y1,y2))=((x1.into_inner(),x2.into_inner()),(y1.into_inner(),y2.into_inner()));
                        let square = [x1,y1,x2-x1,y2-y1];
                        rectangle([0.0,0.0,0.0,0.5], square, self.c.transform, self.g);
                    }   
                }

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

        let mut vv:Vec<(&BBox<f64N,()>,DisSqr)>=Vec::new();
        {
            let mut kn=Kn{c:&c,g};
            let point=[f64n!(cursor[0]),f64n!(cursor[1])];
            k_nearest::k_nearest(&tree,point,3,kn,|a,b|{vv.push((a,b))});
        }

        let cols=[
            [1.0,0.0,0.0,0.8], //red closest
            [0.0,1.0,0.0,0.8], //green second closest
            [0.0,0.0,1.0,0.8]  //blue third closets
        ];
        
        for ((a,dis),color) in vv.iter().zip(cols.iter()){
            let ((x1,x2),(y1,y2))=a.get().get();
            
            {
                let ((x1,x2),(y1,y2))=((x1.into_inner(),x2.into_inner()),(y1.into_inner(),y2.into_inner()));
                let square = [x1,y1,x2-x1,y2-y1];
                            
                rectangle(*color, square, c.transform, g);
            }
        } 
    }   
}

