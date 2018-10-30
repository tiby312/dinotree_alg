extern crate compt;
extern crate piston_window;
extern crate axgeom;
extern crate rand;
extern crate dinotree_alg;
extern crate ordered_float;
extern crate dinotree;
extern crate rayon;
extern crate dinotree_geom;
extern crate dists;
extern crate find_folder;

use dinotree::*;
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

    let assets = find_folder::Search::ParentsThenKids(3, 3)
       .for_folder("assets").unwrap();
    println!("{:?}", assets);
    let ref font = assets.join("font.ttf");
    let factory = window.factory.clone();
    let mut glyphs = Glyphs::new(font, factory,TextureSettings::new()).unwrap();

    let mut cursor=[0.0,0.0];





    let mut grow=0.0;
    let mut num_bots=300;    

    while let Some(e) = window.next() {
        e.mouse_cursor(|x, y| {
            cursor = [x, y];
        });
        if let Some(Button::Keyboard(key)) = e.press_args() {
            if key == Key::Up {
                grow=(grow+0.01f64).min(50000.0);
            }
            if key == Key::Down {
                grow=(grow-0.01f64).max(0.0);
            }
            
            if key == Key::Left{
                if num_bots>100{
                    num_bots-=100;
                }
            }
            if key == Key::Right{
                if num_bots<50000{
                    num_bots+=100;
                }
            }
        };

        let s=dists::spiral::Spiral::new([400.0,400.0],17.0,grow);
        


        let mut bots:Vec<Bot>=s.take(num_bots).map(|pos|{
            let pos=[pos[0] as isize,pos[1] as isize];
            Bot{num:0,pos}
        }).collect();


        let mut tree=DinoTree::new_seq(axgeom::XAXISS,(),&bots,|b|{
            aabb_from_point_isize(b.pos,[5,5])
        });

        window.draw_2d(&e, |c, mut g| {
            clear([1.0; 4], g);

            
            for bot in tree.iter(){
                let a=bot.get();
                draw_rect_isize([0.0,0.0,0.0,0.3],a,&c,g);
            }


            let mut num_collide=0;
            if grow>1.0{
                colfind::query_seq_mut(&mut tree,|a, b| {
                    
                    num_collide+=1;
                    draw_rect_isize([1.0,0.0,0.0,0.2],a.get(),&c,g);
                    draw_rect_isize([1.0,0.0,0.0,0.2],b.get(),&c,g);
            
                });
            }

            let fla=format!("grow={:.02}  num objects={}",grow,num_bots);

            let transform=c.transform.trans(300.0, 700.0);
            text::Text::new_color([0.0, 0.0, 0.0, 1.0], 32).draw(
                &fla,
                &mut glyphs,
                &c.draw_state,
                transform, g
            ).unwrap();
            //println!("grow={} num_collide={}",grow,num_collide);

        });


        /*
        if let Some(Button::Keyboard(key)) = e.press_args() {
            if key == Key::Up {
                circular_grow+=1;
            }
            if key == Key::Down{
                circular_grow-=1;
            }
            if key == Key::Right {
                left_grow+=0.05;
            }
            if key == Key::Left{
                left_grow-=0.05;
            }
        };

        let s=dists::spiral::Spiral::new([400.0,400.0],circular_grow as f64,left_grow);


        let mut bots:Vec<Bot>=s.take(500).map(|pos|{
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
            println!("num_collide={}",num_collide)

        });
        */

    }
}
