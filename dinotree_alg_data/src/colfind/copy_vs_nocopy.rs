use crate::inner_prelude::*;



const ARR_SIZE:usize=4;

#[derive(Copy,Clone)]
pub struct Bot{
    num:usize,
    pos:Vec2<isize>,
    _val:[isize;ARR_SIZE]
}


//TODO use this!!!
trait TestInt{
    type T;
    type Result;
    fn test(bots:&mut [Self::T])->Self::Result;
    fn name(&self)->&'static str;
}




fn test1(bots:&mut [Bot])->f64{
    
    let instant=Instant::now();

   
    let mut tree=DinoTreeBuilder::new(axgeom::XAXISS,bots,|b|axgeom::Rect::from_point(b.pos,vec2same(5))).build_seq();

    
    colfind::QueryBuilder::new(&mut tree).query_seq(|mut a,mut b| {
        a.inner_mut().num+=1;
        b.inner_mut().num+=1;
    });

    black_box(tree);

    instant_to_sec(instant.elapsed())
}


fn test2(bots:&mut [BBox<isize,Bot>])->f64{
    
    let instant=Instant::now();

   
    let mut tree=DinoTreeNoCopyBuilder::new(axgeom::XAXISS,bots).build_seq();

    
    colfind::QueryBuilder::new(&mut tree).query_seq(|mut a,mut b| {
        a.inner.num+=1;
        b.inner.num+=1;
    });
    
    let bots=tree.into_original();

    black_box(bots);

    instant_to_sec(instant.elapsed())
}



fn test3(bots:&mut [Bot])->f64{
    
    let instant=Instant::now();

   
    let mut tree=DinoTreeBuilder::new(axgeom::XAXISS,bots,|b|axgeom::Rect::from_point(b.pos,vec2same(5)) ).build_par();

    
    colfind::QueryBuilder::new(&mut tree).query_par(|mut a,mut b| {
        a.inner_mut().num+=1;
        b.inner_mut().num+=1;
    });
    
    black_box(tree);

    instant_to_sec(instant.elapsed())
}


fn test4(bots:&mut [BBox<isize,Bot>])->f64{
    
    let instant=Instant::now();

   
    let mut tree=DinoTreeNoCopyBuilder::new(axgeom::XAXISS,bots).build_par();

    
    colfind::QueryBuilder::new(&mut tree).query_par(|mut a,mut b| {
        a.inner.num+=1;
        b.inner.num+=1;
    });
    

    let bots = tree.into_original();

    black_box(bots);

    instant_to_sec(instant.elapsed())
}



fn test5(bots:&mut [BBox<isize,Bot>])->f64{
    
    let instant=Instant::now();

   
    let mut tree=DinoTreeNoCopyBuilder::new(axgeom::XAXISS,bots).build_seq_aux();

    
    colfind::QueryBuilder::new(&mut tree).query_seq(|mut a,mut b| {
        a.inner.num+=1;
        b.inner.num+=1;
    });
    

    let bots = tree.into_original();

    black_box(bots);

    instant_to_sec(instant.elapsed())
}




fn test6(bots:&mut [BBox<isize,Bot>])->f64{
    
    let instant=Instant::now();

   
    let mut tree=DinoTreeNoCopyBuilder::new(axgeom::XAXISS,bots).build_par_aux();

    
    colfind::QueryBuilder::new(&mut tree).query_par(|mut a,mut b| {
        a.inner.num+=1;
        b.inner.num+=1;
    });
    

    let bots = tree.into_original();

    black_box(bots);

    instant_to_sec(instant.elapsed())
}





pub fn handle(fb:&mut FigureBuilder){ 
    handle_num_bots(fb,1.0);
}


#[derive(Debug)]
struct Record {
    num_bots:usize,
    arr:[f64;6]    
}
impl Record{
    fn draw(records:&[Record],fg:&mut Figure){
        const NAMES:&[&str]=&["Copy Seq","No Copy Seq","Copy Par","No Copy Par","NoCopy Seq Aux","NoCopy Par Aux"];
        {
            let k=fg.axes2d()
                .set_title(&"Rebal vs Query Comparisons with a spiral grow of 1".to_string(), &[])
                .set_legend(Graph(1.0),Graph(1.0),&[LegendOption::Horizontal],&[])
                .set_x_label("Number of Elements", &[])
                .set_y_label("Number of Comparisons", &[]);

            let x=records.iter().map(|a|a.num_bots);
            for index in 0..6{
                let y=records.iter().map(|a|a.arr[index]);
                k.lines(x.clone(),y,&[Caption(NAMES[index]),Color(COLS[index]),LineWidth(2.0)]);
            }
        }
    }
}



fn handle_num_bots(fb:&mut FigureBuilder,grow:f32){
    
    let s=dists::spiral::Spiral::new([400.0,400.0],17.0,grow);
    let mut rects=Vec::new();

    for num_bots in (0..150_000).rev().step_by(2000){
        
        let mut bots2:Vec<BBoxMut<isize,Bot>>=s.clone().take(num_bots).map(|pos|{
            let inner=Bot{num:0,pos:pos.inner_as(),_val:[0;ARR_SIZE]};
            let aabb=axgeom::Rect::from_point(inner.pos,vec2same(5));
            BBoxMut{aabb,inner}
        }).collect();

        let bots2=into_bbox_slice(&mut bots2);

        

        let mut bots:Vec<Bot>=s.clone().take(num_bots).map(|pos|{
            Bot{num:0,pos:pos.inner_as(),_val:[0;ARR_SIZE]}
        }).collect();

        let arr=[
            test1(&mut bots),
            test2(bots2),
            test3(&mut bots),
            test4(bots2),
            test5(bots2),
            test6(bots2)];

        let r=Record{num_bots,arr};
        rects.push(r);      
    }

    let mut fg= fb.build(&format!("copy_vs_no_copy{}",grow));
    
    Record::draw(&rects,&mut fg);
    
    fb.finish(fg);
}
