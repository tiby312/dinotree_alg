use inner_prelude::*;



#[derive(Copy,Clone)]
pub struct Bot{
    num:usize,
    pos:[isize;2]
}




fn test1(bots:&mut [Bot])->f64{
    
    let mut counter=datanum::Counter::new();

    let mut tree=DynTree::new_seq(axgeom::XAXISS,(),bots,|b|{
        datanum::from_rect(&mut counter,aabb_from_point_isize(b.pos,[5,5]))  
    });

    let a=counter.into_inner();

    colfind::query_seq_mut(&mut tree,|a, b| {
        a.inner.num+=2;
        b.inner.num+=2;
    });

    tree.apply_orig_order(bots,|a,b|{
        b.num=a.inner.num;
    });

    let b=counter.into_inner();

    return (b as f64-a as f64)/b as f64;
}



fn test2(bots:&mut [Bot])->f64{
    
    let instant=Instant::now();

    let mut tree=DynTree::new_seq(axgeom::XAXISS,(),bots,|b|{
        aabb_from_point_isize(b.pos,[5,5])  
    });


    let a=instant_to_sec(instant.elapsed());
    

    colfind::query_seq_mut(&mut tree,|a, b| {
        a.inner.num+=2;
        b.inner.num+=2;
    });

    tree.apply_orig_order(bots,|a,b|{
        b.num=a.inner.num;
    });

    let b=instant_to_sec(instant.elapsed());

    return (b-a)/b;
}

fn test3(bots:&mut [Bot])->f64{
    
    let instant=Instant::now();

    let mut tree=DynTree::new(axgeom::XAXISS,(),bots,|b|{
        aabb_from_point_isize(b.pos,[5,5])  
    });


    let a=instant_to_sec(instant.elapsed());
    

    colfind::query_mut(&mut tree,|a, b| {
        a.inner.num+=2;
        b.inner.num+=2;
    });

    tree.apply_orig_order(bots,|a,b|{
        b.num=a.inner.num;
    });

    let b=instant_to_sec(instant.elapsed());

    return (b-a)/b;
}

fn handle1(){


    #[derive(Debug)]
    struct Record {
        num_bots: usize,
        grow:f64,
        z: f64,
        z2:f64,
        z3:f64        
    }
    let mut rects=Vec::new();
    
    for grow in (0..50).map(|a|0.2+(a as f64)*0.2){

        for num_bots in (0..20000usize).step_by(1000){
            let s=dists::spiral::Spiral::new([400.0,400.0],17.0,grow);


            //let mut bots:Vec<Bot>=(0..num_bots).map(|a|Bot{num:0}).collect();
            let mut bots:Vec<Bot>=s.take(num_bots).map(|pos|{
                let pos=[pos[0] as isize,pos[1] as isize];
                Bot{num:0,pos}
            }).collect();

            let z=test1(&mut bots);
            let z2=test2(&mut bots);
            let z3=test3(&mut bots);

            let r=Record{num_bots,grow,z,z2,z3};
            rects.push(r);   
        }
    }



    let x=rects.iter().map(|a|a.num_bots as f64);
    let y=rects.iter().map(|a|a.grow as f64);
    let z=rects.iter().map(|a|a.z);
    let z2=rects.iter().map(|a|a.z2);
    let z3=rects.iter().map(|a|a.z3);
    
    

    let mut fg = Figure::new();

    fg.axes3d().set_view(110.0,30.0)
        .set_title("Comparisons of Querying Over Total Comparisons", &[])
        .set_x_label("Number of Objects", &[])
        .set_y_label("Spareness of Objects", &[])
        .set_z_label("Query/Total Comparisons", &[Rotate(90.0),TextOffset(-3.0,0.0)])
        .points(x.clone(), y.clone(), z.clone(), &[PointSymbol('O'), Color("red"), PointSize(1.0)]);
                    

    fg.show();


 

    let mut fg = Figure::new();

    fg.axes3d().set_view(110.0,30.0)
        .set_title("Querying Bench Over Total Bench", &[])
        .set_x_label("Number of Objects", &[])
        .set_y_label("Spareness of Objects", &[])
        .set_z_label("Query/Total Time", &[Rotate(90.0),TextOffset(-3.0,0.0)])
        .points(x.clone(), y.clone(), z3.clone(), &[PointSymbol('O'), Color("red"), PointSize(0.5)])
        .points(x.clone(), y.clone(), z2.clone(), &[PointSymbol('O'), Color("violet"), PointSize(0.5)]);
                

    fg.show();



    
}

pub fn handle(){
    //handle1();
    handle2();
}
fn handle2(){

    let mut rects=Vec::new();
    
    struct Record{
        grow:f64,
        bench1:f64,
        bench2:f64,
        comparison:f64
    }
    let num_bots=20000;

    for grow in (0..50).map(|a|0.2+(a as f64)*0.2){
        let s=dists::spiral::Spiral::new([400.0,400.0],17.0,grow);



        //let mut bots:Vec<Bot>=(0..num_bots).map(|a|Bot{num:0}).collect();
        let mut bots:Vec<Bot>=s.take(num_bots).map(|pos|{
            let pos=[pos[0] as isize,pos[1] as isize];
            Bot{num:0,pos}
        }).collect();

        let comparison=test1(&mut bots);
        let bench1=test2(&mut bots);
        let bench2=test3(&mut bots);

        let r=Record{grow,bench1,bench2,comparison};
        rects.push(r);   
    }



    let x=rects.iter().map(|a|a.grow as f64);
    let y1=rects.iter().map(|a|a.bench1);
    let y2=rects.iter().map(|a|a.bench2);
    let y3=rects.iter().map(|a|a.comparison);


    let mut fg = Figure::new();

    fg.axes2d()
        .set_pos_grid(2,1,0)
        .set_title("Querying Bench Over Total Bench with a 20000 objects", &[])
        .lines(x.clone(), y1,  &[Caption("Sequential"), Color("blue"), LineWidth(2.0)])
        .lines(x.clone(), y2,  &[Caption("Parallel"), Color("green"), LineWidth(2.0)])
        .set_x_label("Grow", &[])
        .set_y_label("Query/Total Time", &[]);

    fg.axes2d()
        .set_pos_grid(2,1,1)
        .set_title("Querying Bench Over Total Bench with a 20000 objects", &[])
        .lines(x.clone(), y3,  &[Caption("Sequential"), Color("blue"), LineWidth(2.0)])
        .set_x_label("Grow", &[])
        .set_y_label("Query/Total Time", &[]);

    fg.show();

}