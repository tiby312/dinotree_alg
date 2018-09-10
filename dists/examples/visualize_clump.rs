extern crate compt;
extern crate piston_window;
extern crate axgeom;
extern crate rand;
extern crate dinotree_alg;
extern crate ordered_float;
extern crate dinotree_inner;
extern crate rayon;
extern crate dinotree_geom;
extern crate dists;


use dinotree_inner::*;
use axgeom::*;
use dinotree_alg::*;

#[derive(Copy,Clone)]
pub struct Bot{
    pos:[isize;2],
    num:usize
}
use piston_window::*;


use std::env;


pub fn aabb_from_point_isize(p:[isize;2],r:[isize;2])->Rect<isize>{
    Rect::new(p[0]-r[0],p[0]+r[0],p[1]-r[1],p[1]+r[1])
}

pub fn draw_rect_isize(col:[f32;4],r1:&Rect<isize>,c:&Context,g:&mut G2d){
    let ((x1,x2),(y1,y2))=r1.get();        
    {
        let ((x1,x2),(y1,y2))=((x1 as f64,x2 as f64),(y1 as f64,y2 as f64));
           
        let square = [x1,y1,x2-x1,y2-y1];
        rectangle(col, square, c.transform, g);
    }
}


fn main() {


	let args: Vec<String> = env::args().collect();
   
    let area=[1024u32,768];


    let mut window: PistonWindow = WindowSettings::new("dinotree test",area)
        .exit_on_esc(true)
        .build()
        .unwrap();

    let mut cursor=[0.0,0.0];





    let mut offset_rate=5;
    let mut num_clumped=5;
    while let Some(e) = window.next() {
        e.mouse_cursor(|x, y| {
            cursor = [x, y];
        });
        if let Some(Button::Keyboard(key)) = e.press_args() {
            if key == Key::Up {
                offset_rate+=1;
            }
            if key == Key::Down{
                offset_rate-=1;
            }
            if key == Key::Left{
                num_clumped-=1;
            }
            if key == Key::Right{
                num_clumped+=1;
            }
        };

        let s=dists::clump::Clump::new([400.0,400.0],num_clumped,offset_rate as f64);


        let mut bots:Vec<Bot>=s.take(100).map(|pos|{
            let pos=[pos[0] as isize,pos[1] as isize];
            Bot{num:0,pos}
        }).collect();


        let mut tree=DynTree::new_seq(axgeom::XAXISS,(),&bots,|b|{
            aabb_from_point_isize(b.pos,[5,5])
        });

        let mut val=false;
        window.draw_2d(&e, |c, mut g| {
            clear([1.0; 4], g);

            for bot in tree.iter_every_bot(){
                let a=bot.get();
                draw_rect_isize([0.0,0.0,0.0,0.3],a,&c,g);
            }


            let mut num_collide=0;
            colfind::query_seq_mut(&mut tree,|a, b| {
                
                num_collide+=1;
                draw_rect_isize([1.0,0.0,0.0,0.2],a.get(),&c,g);
                draw_rect_isize([1.0,0.0,0.0,0.2],b.get(),&c,g);
        
            });
            println!("num_clumped={} offset_rate={} num_collide={}",num_clumped,offset_rate,num_collide)

        });

    }
}
