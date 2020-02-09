use crate::support::prelude::*;



///With true rigid body, we only need to store velocity and pos!!!!!
#[derive(Copy, Clone)]
pub struct Bot {
    pos: Vec2<f32>,
    vel: Vec2<f32>, 
}

impl Bot{

    #[must_use]
    fn compute_tval_with_border(&self,border:&Rect<f32>,radius:f32)->Option<f32>{
        let b=self;

        let tvalx1=(border.x.start-b.pos.x-radius)/b.vel.x;
        let tvaly1=(border.y.start-b.pos.y-radius)/b.vel.y;

        let tvalx2=(border.x.end-b.pos.x+radius)/b.vel.x;
        let tvaly2=(border.y.end-b.pos.y+radius)/b.vel.y;

        let arr=[tvalx1,tvaly1,tvalx2,tvaly2];

        let k=arr.iter().filter(|a|**a>0.0).min_by(|a,b|a.partial_cmp(b).unwrap()).map(|a|*a);
        if let Some(k)=k{
            assert!(!k.is_nan())
        }
        k
    }
    #[must_use]
    fn compute_tval_with_bot(&self,bot:&Bot,radius:f32)->Option<f32>{
        let b1=self;
        let b2=bot;

        let pos=b2.pos-b1.pos;
        let vel=b2.vel-b1.vel;   
        //dbg!(pos,vel);
        if vel.magnitude().is_nan(){
            return None
        }

        //get the component of pos along vel
        let pp=pos.dot(vel)/vel.magnitude(); //TODO optimize

        let radius2=radius*2.0;

        let rr=(pos + vel.normalize_to(1.0)*pp).magnitude();

        if rr.is_nan(){
            return None
        }

        //bot passes right past this bot
        if radius2<rr{
            return None
        }
        let yy=(radius2*radius2-rr*rr).sqrt();

        let aa=(pp-yy)/vel.magnitude();

        assert!(!aa.is_nan(),"{:?}",(pp,yy,rr,radius2,aa));
        Some(aa )
    }

    fn collide_with_border(&mut self, rect2: &Rect<f32>) {
        let a = self;
        let xx = rect2.get_range(axgeom::XAXIS);
        let yy = rect2.get_range(axgeom::YAXIS);

        let pos=&mut a.pos;

        
        if pos.x < xx.start {
            pos.x = xx.start;
        }
        if pos.x > xx.end {
            pos.x = xx.end;
        }
        if pos.y < yy.start {
            pos.y = yy.start;
        }
        if pos.y > yy.end {
            pos.y = yy.end;
        }
    }
    fn collide_with_bot(&mut self, b: &mut Bot) {
        let a = self;

        let cc = 0.5;

        let pos_diff = b.pos - a.pos;

        let pos_diff_norm = pos_diff.normalize_to(1.0);

        let vel_diff = b.vel - a.vel;

        let im1 = 1.0;
        let im2 = 1.0;

        let vn = vel_diff.dot(pos_diff_norm);
        if vn > 0.0 {
            return;
        }

        let i = (-(1.0 + cc) * vn) / (im1 + im2);
        let impulse = pos_diff_norm * i;

        a.vel -= impulse * im1;
        b.vel += impulse * im2;
    }

}


struct BorderCollision{
    bot:*mut Bot,
    tval:Option<f32>
}
struct BotCollision{
    bota:*mut Bot,
    botb:*mut Bot,
    tval:Option<f32>
}





pub fn make_demo(dim: Rect<F32n>) -> Demo {
    let num_bot = 4000;

    let radius = 5.0;

    let mut bots: Vec<_> = UniformRandGen::new(dim.inner_into())
        .take(num_bot)
        .map(|pos| Bot {
            pos,
            vel: vec2same(0.0),
        })
        .collect();

    Demo::new(move |cursor, canvas, _check_naive| {
        
        //The max amount a bot can move.
        let max_velocity=0.1;
        let time_step=30.0;
        let padding=max_velocity*time_step  ;
        let mut k = bbox_helper::create_bbox_mut(&mut bots, |b| {
            Rect::from_point(b.pos, vec2same(radius+padding))
                .inner_try_into()
                .unwrap()
        });
        let mut tree = DinoTree::new_par(&mut k);


        let mut border_collisions=Vec::new();
        {
            let dim2 = dim.inner_into();
            tree.for_all_not_in_rect_mut(&dim, |mut a| {
                let bot=a.inner_mut();
                let tval=bot.compute_tval_with_border(&dim2,radius);
                border_collisions.push(BorderCollision{bot,tval});
            });
        }
        
        
        
        let vv = vec2same(50.0).inner_try_into().unwrap();
        tree.for_all_in_rect_mut(&axgeom::Rect::from_point(cursor, vv), |mut b| {
            let b=b.inner_mut();
            
            let offset=b.pos-cursor.inner_into();
            b.vel+=offset*0.1;
        });

        
        

        let mut bot_collisions=Vec::new();
        tree.find_collisions_mut(|mut a, mut b| {
            let bota=a.inner_mut();
            let botb=b.inner_mut();

            let tval=bota.compute_tval_with_bot(botb,radius);

            bot_collisions.push(BotCollision{bota,botb,tval});
        });
        

        //Find all bot collisions
        //    (find all loose bounding box collisions and also compute their tval (might be zero. they might not collide))
        //Find all border collisions
        //Sort both
        //
        //while both lists are not empty take the lesser of the first of each list
        //   if bot collision
        //      move bots to collision point
        //      handle collision
        //      go through bot collision and find collisions that involve the bots that move and update their tval
        //      (also possible that they no longer collide at all)
        //   if wall collisions
        //
        
        //dbg!("iterating");
        
        loop{

            let next_border_col=border_collisions.iter_mut().filter(|a|a.tval.is_some()).min_by(|a,b|a.tval.partial_cmp(&b.tval).unwrap());
            let next_bot_col=bot_collisions.iter_mut().filter(|a|a.tval.is_some()).min_by(|a,b|a.tval.partial_cmp(&b.tval).unwrap());


            let (next_border_col,next_bot_col) = match (next_border_col,next_bot_col){
                (Some(next_border_col),Some(next_bot_col))=>{
                    if next_border_col.tval < next_bot_col.tval{
                        (Some(next_border_col),None)
                    }else{
                        (None,Some(next_bot_col))
                    }
                },
                (Some(next_border_col),None)=>{
                    (Some(next_border_col),None)
                },
                (None,Some(next_bot_col))=>{
                    (None,Some(next_bot_col))
                },
                (None,None)=>{
                    unreachable!()
                }
            };


            fn add(a:f32,b:Option<f32>)->Option<f32>{
                match b{
                    Some(b)=>{
                        Some(a+b)
                    },
                    None=>{
                        None
                    }
                }
            }

            if let Some(next_border_col)=next_border_col{
                if let Some(tval)=next_border_col.tval{
                    if tval>time_step{
                        break;
                    }

                    let bota=unsafe{&mut *next_border_col.bot};
                    
                    assert!(!bota.vel.magnitude().is_nan());

                    bota.pos+=bota.vel.normalize_to(1.0)*tval;
                    bota.collide_with_border(dim.as_ref());


                    for b in border_collisions.iter_mut(){
                        if b.bot==bota as *mut _{
                            b.tval=add(tval,bota.compute_tval_with_border(dim.as_ref(),radius));
                        }
                    }
                    for b in bot_collisions.iter_mut(){
                        if b.bota==bota as *mut _ {
                            b.tval=add(tval,bota.compute_tval_with_bot(unsafe{&mut *b.botb},radius));
                        }else if b.botb==bota as *mut _{
                            b.tval=add(tval,bota.compute_tval_with_bot(unsafe{&mut *b.bota},radius));
                        }
                    }
                }else{
                    unreachable!()
                }
               
            }else if let Some(next_bot_col)=next_bot_col{
                if let Some(tval)=next_bot_col.tval{
                    if tval>time_step{
                        break;
                    }

                    let bota=unsafe{&mut *next_bot_col.bota};
                    let botb=unsafe{&mut *next_bot_col.botb};
                    assert!(!bota.vel.magnitude().is_nan());
                    assert!(!botb.vel.magnitude().is_nan());

                    bota.pos+=bota.vel.normalize_to(1.0)*tval;
                    botb.pos+=botb.vel.normalize_to(1.0)*tval;
                    bota.collide_with_bot(botb);


                    for b in border_collisions.iter_mut(){
                        if b.bot==bota as *mut _{
                            b.tval=add(tval,bota.compute_tval_with_border(dim.as_ref(),radius));
                        }
                    }
                    for b in bot_collisions.iter_mut(){
                        if b.bota==bota as *mut _ {
                            b.tval=add(tval,bota.compute_tval_with_bot(unsafe{&mut *b.botb},radius));
                        }else if b.botb==bota as *mut _{
                            b.tval=add(tval,bota.compute_tval_with_bot(unsafe{&mut *b.bota},radius));
                        }else if b.bota==botb as *mut _{
                            b.tval=add(tval,botb.compute_tval_with_bot(unsafe{&mut *b.bota},radius));
                        }else if b.botb==botb as *mut _{
                            b.tval=add(tval,botb.compute_tval_with_bot(unsafe{&mut *b.botb},radius));
                        }
                    }
                }else{
                    unreachable!()
                }
                
            }
        }
        
       

        
        for b in bots.iter_mut(){
            b.pos += b.vel;
        }
        

        let mut circles = canvas.circles();
        for bot in bots.iter() {
            circles.add(bot.pos.into()); //TODO we're not testing that the bots were draw in the right order
        }
        circles.send_and_uniforms(canvas,radius).with_color([1.0, 1.0, 0.0, 0.6]).draw();
        
    })
}
