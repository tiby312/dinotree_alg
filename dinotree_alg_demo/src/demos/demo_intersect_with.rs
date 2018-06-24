use support::prelude::*;
use dinotree::colfind;
use dinotree::rect;
use dinotree::intersect_with;
use dinotree_geom;
use dinotree;
pub struct Bot{
    pos:[f64;2],
    vel:[f64;2],
    force:[f64;2],
}
impl dinotree_geom::RepelTrait for Bot{
    type N=f64;
    fn pos(&self)->[f64;2]{
        self.pos
    }
    fn add_force(&mut self,force:[f64;2]){
        self.force[0]+=force[0];
        self.force[1]+=force[1];
    }
}
impl Bot{
    fn update(&mut self){
        self.vel[0]+=self.force[0];
        self.vel[1]+=self.force[1];

        //non linear drag
        self.vel[0]*=0.9;
        self.vel[1]*=0.9;

        self.pos[0]+=self.vel[0];
        self.pos[1]+=self.vel[1];
        self.force[0]=0.0;
        self.force[1]=0.0;
    }
}
pub struct IntersectWithDemo{
    radius:f64,
    bots:Vec<Bot>,
    walls:Vec<BBox<f64N,()>>,
    dim:[f64;2]
}
impl IntersectWithDemo{
    pub fn new(dim:[f64;2])->IntersectWithDemo{
        let dim2=&[0,dim[0] as isize,0,dim[1] as isize];
        let radius=[3,5];
        let velocity=[1,3];
        let bots=create_world_generator(4000,dim2,radius,velocity).map(|ret|{
            Bot{pos:ret.pos,vel:ret.vel,force:[0.0;2]}
        }).collect();

        let radius=[10,60];
        let walls=create_world_generator(40,dim2,radius,velocity).map(|ret|{
            let rect=aabb_from_pointf64(ret.pos,ret.radius);
            BBox::new(Conv::from_rect(rect),())//{pos:ret.pos,vel:ret.vel,force:[0.0;2]}
        }).collect();

        IntersectWithDemo{radius:10.0,bots,walls,dim}
    }
}

impl DemoSys for IntersectWithDemo{
    fn step(&mut self,cursor:[f64;2],c:&piston_window::Context,g:&mut piston_window::G2d){
        let radius=5.0;
        let bots=&mut self.bots;
        let walls=&mut self.walls;

        for b in bots.iter_mut(){
            b.update();
            dinotree_geom::wrap_position(&mut b.pos,self.dim);
        }
        bots[0].pos=cursor;


        let mut tree=DynTree::new(axgeom::XAXISS,(),bots.drain(..).map(|b|{
            let p=b.pos;
            let rect=aabb_from_pointf64(p,[radius;2]);
            BBox::new(Conv::from_rect(rect),b)
        }));

        use axgeom::*;
        

        intersect_with::intersect_with_mut(&mut tree,walls,|bot,wall|{
            let fric=0.8;


            let wallx=wall.get().as_axis().get(axgeom::XAXISS);
            let wally=wall.get().as_axis().get(axgeom::YAXISS);
            let vel=bot.inner.vel;

            let ret=match dinotree_geom::collide_with_rect(bot.get(),wall.get()){
                dinotree_geom::WallSide::Above=>{
                    [None,Some((wally.left-radius,-vel[1]*fric))]
                },
                dinotree_geom::WallSide::Below=>{
                    [None,Some((wally.right+radius,-vel[1]*fric))]
                },
                dinotree_geom::WallSide::LeftOf=>{
                    [Some((wallx.left-radius,-vel[0]*fric)),None]
                },
                dinotree_geom::WallSide::RightOf=>{
                    [Some((wallx.right+radius,-vel[0]*fric)),None]
                }
            };

            match ret[0]{
                Some((pos,vel))=>{
                    bot.inner.pos[0]=pos.into_inner();
                    bot.inner.vel[0]=vel;
                },
                None=>{}
            }
            match ret[1]{
                Some((pos,vel))=>{
                    bot.inner.pos[1]=pos.into_inner();
                    bot.inner.vel[1]=vel;
                },
                None=>{}
            }
        });

        
        for b in tree.into_iter_orig_order(){
            bots.push(b.inner);
        }

        //Update the aabbs to match the new positions.
        let mut tree=DynTree::new(axgeom::XAXISS,(),bots.drain(..).map(|b|{
            let p=b.pos;
            let rect=aabb_from_pointf64(p,[radius;2]);
            BBox::new(Conv::from_rect(rect),b)
        }));
    


        
        rect::for_all_in_rect_mut(&mut tree,&Conv::from_rect(aabb_from_pointf64(cursor,[100.0;2])),|b|{
            //b.inner.repel_mouse(cursor);
            let _ =dinotree_geom::repel_one(&mut b.inner,cursor,0.001,20.0,|a|a.sqrt());
        });
        

        for wall in walls.iter(){
            let ((x1,x2),(y1,y2))=wall.get().get();
            //let ((x1,x2),(y1,y2))=((x1 as f64,x2 as f64),(y1 as f64,y2 as f64));
            let ((x1,x2),(y1,y2))=((x1.into_inner(),x2.into_inner()),(y1.into_inner(),y2.into_inner()));
              
            let square = [x1,y1,x2-x1,y2-y1];
            rectangle([0.0,0.0,1.0,0.3], square, c.transform, g);
        }
        for bot in tree.iter(){
            let ((x1,x2),(y1,y2))=bot.get().get();
            //let ((x1,x2),(y1,y2))=((x1 as f64,x2 as f64),(y1 as f64,y2 as f64));
            let ((x1,x2),(y1,y2))=((x1.into_inner(),x2.into_inner()),(y1.into_inner(),y2.into_inner()));
              
            let square = [x1,y1,x2-x1,y2-y1];
            rectangle([0.0,0.0,0.0,0.3], square, c.transform, g);
        }

        {
         
            colfind::query_mut(&mut tree,|a, b| {
                let _ = dinotree_geom::repel(&mut a.inner,&mut b.inner,0.001,2.0,|a|a.sqrt());
            });
        
        }

        struct Bla<'a,'b:'a>{
            c:&'a Context,
            g:&'a mut G2d<'b>
        }
        impl<'a,'b:'a> dinotree::graphics::DividerDrawer for Bla<'a,'b>{
            type N=f64N;
            fn draw_divider<A:axgeom::AxisTrait>(&mut self,axis:A,div:f64N,cont:Option<[f64N;2]>,length:[f64N;2],depth:usize){
                let div=div.into_inner();
                

                let arr=if axis.is_xaxis(){
                    [div,length[0].into_inner(),div,length[1].into_inner()]
                }else{
                    [length[0].into_inner(),div,length[1].into_inner(),div]
                };


                let radius=(5-depth) as f64;

                line([0.0, 0.0, 0.0, 0.5], // black
                     radius, // radius of line
                     arr, // [x0, y0, x1,y1] coordinates of line
                     self.c.transform,
                     self.g);

                match cont{
                    Some(cont)=>{
                        let [x1,y1,w1,w2]=if axis.is_xaxis(){
                            [cont[0],length[0],cont[1]-cont[0],length[1]-length[0]]
                        }else{
                            [length[0],cont[0],length[1]-length[0],cont[1]-cont[0]]
                        };
                        //let ((x1,x2),(w1,w2))=((x1 as f64,x2 as f64),(w1 as f64,w2 as f64));
        
                        let square = [x1.into_inner(),y1.into_inner(),w1.into_inner(),w2.into_inner()];
                        rectangle([0.0,1.0,1.0,0.2], square, self.c.transform, self.g);
                    
                    },
                    None=>{

                    }
                }
                
            }
        }
        let mut dd=Bla{c:&c,g};
        dinotree::graphics::draw(&tree,&mut dd,&axgeom::Rect::new(f64n!(0.0),f64n!(self.dim[0]),f64n!(0.0),f64n!(self.dim[1])));

        for b in tree.into_iter_orig_order(){
            bots.push(b.inner);
        }
     }
}

