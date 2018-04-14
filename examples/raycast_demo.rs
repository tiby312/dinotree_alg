extern crate piston_window;
extern crate axgeom;
extern crate num;
extern crate rand;
extern crate dinotree;

use piston_window::*;

mod support;
use dinotree::*;
use dinotree::support::*;
use support::*;


fn main() {
    let mut p = PointGenerator::new(
        &support::make_rect((0, 1000), (0, 1000)),
        &[100, 42, 6],
    );

    let mut bots = Vec::new();
    for id in 0..10000 {
        let ppp = p.random_point();
        let k = support::create_rect_from_point(ppp);
        bots.push(BBox::new(
            Bot {
                id,
                col: Vec::new(),
            },
            k,
        ));
    }

    //let height = compute_tree_height(bots.len());

    let mut tree = DinoTree::new(&mut bots, StartAxis::Xaxis);


    let mut window: PistonWindow = WindowSettings::new("raycast test", [640, 480])
        .exit_on_esc(true)
        .build()
        .unwrap();
    while let Some(e) = window.next() {
        window.draw_2d(&e, |c, g| {
            clear([1.0; 4], g);

            line([0.0, 0.0, 0.0, 1.0], // black
                 2.0, // radius of line
                 [10.0,10.0,40.0,40.0], // [x0, y0, x1,y1] coordinates of line
                 c.transform,
                 g);
        
        });
    }

}
