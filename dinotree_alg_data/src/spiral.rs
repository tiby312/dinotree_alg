
use crate::inner_prelude::*;



pub fn handle(fb:&mut FigureBuilder){
	handle1(fb);
	handle2(fb);
}

fn handle1(fb:&mut FigureBuilder){


	let mut fg=fb.new("spiral_data");	

	let num_bots=10000;
	let mut rects=Vec::new();
	for grow in (0..100).map(|a|0.2+(a as f64)*0.02){
		let s=dists::spiral::Spiral::new([0.0,0.0],17.0,grow);

	    let mut bots:Vec<[f64;2]>=s.take(num_bots).collect();
    	

        let mut tree=DinoTree::new(axgeom::XAXISS,(),&bots,|b|{   
            ConvF64::from_rect(aabb_from_pointf64(*b,[5.0,5.0]))
        });

        let mut num_intersection=0;
        colfind::QueryBuilder::new().query_seq(tree.as_ref_mut(),|_a, _b| {
           num_intersection+=1;
        });

        /*
        tree.apply_orig_order(&mut bots,|a,b|{
            b.num=a.inner.num;
        });
        */

        rects.push((grow,num_intersection));
	}


	let x=rects.iter().map(|a|a.0);
	let y=rects.iter().map(|a|a.1);
    fg.axes2d()
    	.set_title("Number of Intersections with 10000 objects with a AABB for size 10 and a spiral separation of 17.0", &[])
        .lines(x, y,  &[Caption("Naive"), Color("red"), LineWidth(4.0)])
        .set_x_label("Spiral Grow", &[])
        .set_y_label("Number of Intersections", &[]);
    fg.show();
}

fn handle2(fb:&mut FigureBuilder){



    fn make(grow:f64)->Vec<[f64;2]>{
  	    let num_bots=1000;

    	let s=dists::spiral::Spiral::new([0.0,0.0],17.0,grow);

	    let bots:Vec<[f64;2]>=s.take(num_bots).collect();
    	bots
    };



	let mut fg=fb.new("spiral_visualize");	


    let a=make(0.1);
    let ax=a.iter().map(|a|a[0]);
    let ay=a.iter().map(|a|a[1]);
    

	            
    fg.axes2d()
    	.set_pos_grid(2,2,0)
    	.set_x_range(Fix(-500.0),Fix(500.0))
    	.set_y_range(Fix(-500.0),Fix(500.0))
        .set_title("Grow of 0.1 of size 10000", &[])
        .points(ax, ay,  &[Caption("Naive"), Color("red"), LineWidth(4.0)])
        .set_x_label("x", &[])
        .set_y_label("y", &[]);
    fg.show();


    let a=make(0.5);
    let ax=a.iter().map(|a|a[0]);
    let ay=a.iter().map(|a|a[1]);


    fg.axes2d()
    	.set_pos_grid(2,2,1)
    	.set_x_range(Fix(-500.0),Fix(500.0))
    	.set_y_range(Fix(-500.0),Fix(500.0))
        .set_title("Grow of 0.3 of size 10000", &[])
        .points(ax, ay,  &[Caption("Naive"), Color("red"), LineWidth(4.0)])
        .set_x_label("x", &[])
        .set_y_label("y", &[]) ;
    fg.show();


    let a=make(3.0);
    let ax=a.iter().map(|a|a[0]);
    let ay=a.iter().map(|a|a[1]);


    fg.axes2d()
    	.set_pos_grid(2,2,2)
    	.set_x_range(Fix(-500.0),Fix(500.0))
    	.set_y_range(Fix(-500.0),Fix(500.0))
        .set_title("Grow of 3.0 of size 10000", &[])
        .points(ax, ay,  &[Caption("Naive"), Color("red"), LineWidth(4.0)])
        .set_x_label("x", &[])
        .set_y_label("y", &[]);
    fg.show();


    let a=make(6.0);
    let ax=a.iter().map(|a|a[0]);
    let ay=a.iter().map(|a|a[1]);


    fg.axes2d()
    	.set_pos_grid(2,2,3)
    	.set_x_range(Fix(-500.0),Fix(500.0))
    	.set_y_range(Fix(-500.0),Fix(500.0))
        .set_title("Grow of 6.0 of size 10000", &[])
        .points(ax, ay,  &[Caption("Naive"), Color("red"), LineWidth(4.0)])
        .set_x_label("x", &[])
        .set_y_label("y", &[]);
    fg.show();
    
}