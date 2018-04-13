extern crate piston_window;

use piston_window::*;

mod support;

fn main() {
    

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
