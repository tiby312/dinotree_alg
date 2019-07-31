use crate::support::prelude::*;
use dinotree_alg::colfind;
use dinotree_alg::rect;
use duckduckgeo;
use dinotree_alg;


#[derive(Debug,Copy,Clone)]
struct Ray<N>{
    pub point:[N;2],
    pub dir:[N;2],
    pub tlen:N,
}

#[derive(Copy,Clone)]
pub struct Bot{
    id:usize, //id used to verify pairs against naive
    pos:Vector2<f64>,
    vel:Vector2<f64>,
    force:Vector2<f64>,
}

impl duckduckgeo::BorderCollideTrait for Bot{
    type N=f64;
    fn pos_vel_mut(&mut self)->(&mut Vector2<f64>,&mut Vector2<f64>){
        (&mut self.pos,&mut self.vel)
    }
}

impl duckduckgeo::RepelTrait for Bot{
    type N=f64;
    fn pos(&self)->[f64;2]{
        [self.pos.x,self.pos.y]
    }
    fn add_force(&mut self,force:[f64;2]){
        self.force.x+=force[0];
        self.force.y+=force[1];
    }
}


impl Bot{
    fn update(&mut self){
        self.vel+=self.force;


        //non linear drag
        self.vel*=0.9;


        self.pos+=self.vel;

        self.force=Vector2::zero();
    }
}


pub struct OrigOrderDemo{
    radius:f64,
    bots:Vec<BBoxMut<F64n,Bot>>,
    colors:Vec<[u8;3]>,
    dim:[f64;2]
}
impl OrigOrderDemo{
    pub fn new(dim:[f64;2])->OrigOrderDemo{
        let dim2=&[0,dim[0] as isize,0,dim[1] as isize];
        let radius=[3,5];
        let velocity=[1,3];
        let bots=create_world_generator(4000,dim2,radius,velocity).enumerate().map(|(id,ret)|{
            let bot=Bot{pos:vec2(ret.pos[0],ret.pos[1]),vel:vec2(ret.vel[0],ret.vel[1]),force:Vector2::zero(),id};
            let rect=axgeom::Rect::from_point(ret.pos,[5.0;2]).into_notnan().unwrap();
            BBoxMut::new(rect,bot)
        }).collect();
 
        let colors=ColorGenerator::new().take(4000).collect();
        OrigOrderDemo{radius:5.0,bots,colors,dim}
    }
}



impl DemoSys for OrigOrderDemo{
    fn step(&mut self,cursor:[f64;2],c:&piston_window::Context,g:&mut piston_window::G2d,check_naive:bool){
        let radius=self.radius;
        
        for b in self.bots.iter_mut(){
            b.inner.update();
            b.aabb=axgeom::Rect::from_point([b.inner.pos.x,b.inner.pos.y],[radius;2]).into_notnan().unwrap();
        }


        

        let mut tree=DinoTreeNoCopyBuilder::new(axgeom::XAXISS,&mut self.bots).build_par(); 


        let rect=axgeom::Rect::new(0.0,self.dim[0],0.0,self.dim[1]).into_notnan().unwrap();
            

        {
            let rect2=rect.into_inner();
            dinotree_alg::rect::for_all_not_in_rect_mut(&mut tree,&rect,|a|{
                duckduckgeo::collide_with_border(&mut a.inner,&rect2,0.5);
            });
        }

        rect::for_all_in_rect_mut(&mut tree,&axgeom::Rect::from_point(cursor,[100.0;2]).into_notnan().unwrap(),|b|{
            let _ =duckduckgeo::repel_one(&mut b.inner,cursor,0.001,20.0,|a|a.sqrt());
        });
        

        

        {
            struct Bla<'a,'b:'a>{
                c:&'a Context,
                g:&'a mut G2d<'b>
            }
            impl<'a,'b:'a> dinotree_alg::graphics::DividerDrawer for Bla<'a,'b>{
                type N=F64n;
                fn draw_divider<A:axgeom::AxisTrait>(&mut self,axis:A,div:F64n,cont:[F64n;2],length:[F64n;2],depth:usize){
                    let div=div.into_inner();
                    

                    let arr=if axis.is_xaxis(){
                        [div,length[0].into_inner(),div,length[1].into_inner()]
                    }else{
                        [length[0].into_inner(),div,length[1].into_inner(),div]
                    };


                    let radius=(1isize.max(5-depth as isize)) as f64;

                    line([0.0, 0.0, 0.0, 0.5], // black
                         radius, // radius of line
                         arr, // [x0, y0, x1,y1] coordinates of line
                         self.c.transform,
                         self.g);

                    let [x1,y1,w1,w2]=if axis.is_xaxis(){
                        [cont[0],length[0],cont[1]-cont[0],length[1]-length[0]]
                    }else{
                        [length[0],cont[0],length[1]-length[0],cont[1]-cont[0]]
                    };

                    let square = [x1.into_inner(),y1.into_inner(),w1.into_inner(),w2.into_inner()];
                    rectangle([0.0,1.0,1.0,0.2], square, self.c.transform, self.g);
                }
            }

            let mut dd=Bla{c:&c,g};
            dinotree_alg::graphics::draw(&tree,&mut dd,&axgeom::Rect::new(f64n!(0.0),f64n!(self.dim[0]),f64n!(0.0),f64n!(self.dim[1])));
        }


        //draw lines to the bots.
        {
            fn draw_bot_lines<A:axgeom::AxisTrait>
                (axis:A,stuff:Vistr<BBox<F64n,Bot>>,rect:&axgeom::Rect<F64n>,c:&Context,g:&mut G2d){
                use compt::Visitor;
                let (nn,rest)=stuff.next();

                let mid=match rest{

                    Some([left,right]) =>{
               
                        match nn.div{
                            Some(div)=>{

                                let (a,b)=rect.subdivide(axis,*div);

                                draw_bot_lines(axis.next(),left,&a,c,g);
                                draw_bot_lines(axis.next(),right,&b,c,g);

                                let ((x1,x2),(y1,y2))=rect.into_inner().get();
                                let midx = if !axis.is_xaxis(){
                                    x1 + (x2-x1)/2.0
                                }else{
                                    div.into_inner()
                                };

                                let midy = if axis.is_xaxis(){
                                    y1 + (y2-y1)/2.0
                                }else{
                                    div.into_inner()
                                };


                                Some((midx,midy))
                        
                            },
                            None=>{
                               None
                            }
                        }
                    },
                    None=>{
                        let ((x1,x2),(y1,y2))=rect.into_inner().get();
                        let midx = x1 + (x2-x1)/2.0;

                        let midy = y1 + (y2-y1)/2.0;

                        Some((midx,midy))
                    }
                };


                if let Some((midx,midy)) = mid{
                    let color_delta=1.0/nn.bots.len() as f32;
                    let mut counter=0.0;
                    for b in nn.bots.iter(){
                        let bx=b.inner.pos[0];
                        let by=b.inner.pos[1];

                        line([counter, 0.2, 0.0, 0.3], // black
                             2.0, // radius of line
                             [midx,midy,bx,by], // [x0, y0, x1,y1] coordinates of line
                             c.transform,
                             g);

                        counter+=color_delta;
                    }
                }
            }

            draw_bot_lines(tree.axis(),tree.vistr(),&rect,c,g);

        }


        if !check_naive{
            colfind::QueryBuilder::new(&mut tree).query_par(|a, b| {
                let _ = duckduckgeo::repel(&mut a.inner,&mut b.inner,0.001,2.0,|a|a.sqrt());
            });
        }else{
            let mut res=Vec::new();
            colfind::QueryBuilder::new(&mut tree).query_seq(|a, b| {
                let a=&mut a.inner;
                let b=&mut b.inner;
                let _ = duckduckgeo::repel(a,b,0.001,2.0,|a|a.sqrt());
                let (a,b)=if a.id<b.id{
                    (a,b)
                }else{
                    (b,a)
                };
                res.push((a.id,b.id));
            });



            let mut res2=Vec::new();
            
            colfind::query_naive_mut(tree.get_bots_mut(),|a,b|{
                let a=&mut a.inner;
                let b=&mut b.inner;
                let (a,b)=if a.id<b.id{
                    (a,b)
                }else{
                    (b,a)
                };
                res2.push((a.id,b.id))
            });

            let cmp=|a:&(usize,usize),b:&(usize,usize)|{
                use std::cmp::Ordering;
           
                match a.0.cmp(&b.0){
                    Ordering::Less=>{
                        Ordering::Less
                    },
                    Ordering::Greater=>{
                        Ordering::Greater
                    },
                    Ordering::Equal=>{
                        match a.1.cmp(&b.1){
                            Ordering::Less=>{
                                Ordering::Less
                            },
                            Ordering::Greater=>{
                                Ordering::Greater
                            },
                            Ordering::Equal=>{
                                Ordering::Equal
                            }
                        }
                    }
                }
            };

            res.sort_by(cmp);
            res2.sort_by(cmp);
            println!("lens={:?}",(res.len(),res2.len()));
            assert_eq!(res.len(),res2.len());
            for (a,b) in res.iter().zip(res2.iter()){
                assert_eq!(a,b)
            }

        }
        
        tree.into_original();
        

        
        fn conv(a:u8)->f32{
            let a:f32=a.as_();
            a/256.0
        }
        
        for (bot,cols) in self.bots.iter().zip(self.colors.iter()){
            let rect=&axgeom::Rect::from_point([bot.inner.pos.x,bot.inner.pos.y],[radius;2]).into_notnan().unwrap();
            draw_rect_f64n([conv(cols[0]),conv(cols[1]),conv(cols[2]),0.6],rect,c,g);
        } 
        

    }
}

