use support::prelude::*;
use dinotree_alg::colfind;
use dinotree_alg::rect;
use dinotree_geom;
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


pub struct OrigOrderDemo{
    radius:f64,
    bots:Vec<Bot>,
    colors:Vec<[u8;3]>,
    dim:[f64;2]
}
impl OrigOrderDemo{
    pub fn new(dim:[f64;2])->OrigOrderDemo{
        let dim2=&[0,dim[0] as isize,0,dim[1] as isize];
        let radius=[3,5];
        let velocity=[1,3];
        let bots=create_world_generator(4000,dim2,radius,velocity).enumerate().map(|(id,ret)|{
            Bot{pos:ret.pos,vel:ret.vel,force:[0.0;2],id}
        }).collect();
 
        let colors=ColorGenerator::new().take(4000).collect();
        OrigOrderDemo{radius:5.0,bots,colors,dim}
    }
}

impl DemoSys for OrigOrderDemo{
    fn step(&mut self,cursor:[f64;2],c:&piston_window::Context,g:&mut piston_window::G2d,check_naive:bool){
        let radius=self.radius;
        let bots=&mut self.bots;

        for b in bots.iter_mut(){
            b.update();
            dinotree_geom::wrap_position(&mut b.pos,self.dim);
        }


        let mut tree=DinoTree::new(axgeom::XAXISS,(),&bots,|bot|{
           Conv::from_rect(aabb_from_pointf64(bot.pos,[radius;2]))
        }); 

        
        rect::for_all_in_rect_mut(&mut tree,&Conv::from_rect(aabb_from_pointf64(cursor,[100.0;2])),|b|{
            let _ =dinotree_geom::repel_one(&mut b.inner,cursor,0.001,20.0,|a|a.sqrt());
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

        if !check_naive{
            colfind::query_mut(&mut tree,|a, b| {
                let _ = dinotree_geom::repel(&mut a.inner,&mut b.inner,0.001,2.0,|a|a.sqrt());
            });
        }else{
            let mut res=Vec::new();
            colfind::query_seq_mut(&mut tree,|a, b| {
                let _ = dinotree_geom::repel(&mut a.inner,&mut b.inner,0.001,2.0,|a|a.sqrt());
                let (a,b)=if a.inner.id<b.inner.id{
                    (a,b)
                }else{
                    (b,a)
                };
                res.push((a.inner.id,b.inner.id));
            });


            let mut res2=Vec::new();
            let mut bots2:Vec<BBox<F64n,Bot>>=bots.iter().map(|bot|unsafe{BBox::new(Conv::from_rect(aabb_from_pointf64(bot.pos,[radius;2])),*bot)}).collect();
            colfind::query_naive_mut(&mut bots2,|a,b|{
                let (a,b)=if a.inner.id<b.inner.id{
                    (a,b)
                }else{
                    (b,a)
                };
                res2.push((a.inner.id,b.inner.id))
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
        
        tree.apply(bots,|b,t|*t=b.inner);

        /*
        //If you dont care about the order, you can do this instead.
        //But in this case, this will cause the colors to not be assigned to the correct bots.
        for (a,b) in tree.iter_every_bot().zip(bots.iter_mut()){
            *b=a.inner;
        }
        */
        
        fn conv(a:u8)->f32{
            a as f32/256.0
        }
        for (bot,cols) in bots.iter().zip(self.colors.iter()){
            let rect=&Conv::from_rect(aabb_from_pointf64(bot.pos,[radius;2]));
            draw_rect_f64n([conv(cols[0]),conv(cols[1]),conv(cols[2]),1.0],rect,c,g);
        }        
    }
}

