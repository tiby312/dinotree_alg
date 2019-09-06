use crate::inner_prelude::*;



const ARR_SIZE:usize=4;

#[derive(Copy,Clone)]
pub struct Bot{
    num:usize,
    pos:Vec2<isize>,
    _val:[isize;ARR_SIZE]
}




fn test1(bots:&mut [Bot])->f64{
    
    let instant=Instant::now();

   
    let mut tree=DinoTreeBuilder::new(axgeom::XAXISS,bots,|b|axgeom::Rect::from_point(b.pos,vec2same(5))).build_seq();

    
    colfind::QueryBuilder::new(&mut tree).query_seq(|mut a,mut b| {
        a.inner.num+=1;
        b.inner.num+=1;
    });

    black_box(tree);

    instant_to_sec(instant.elapsed())
}

fn test2(bots:&mut [Bot])->f64{
    
    let instant=Instant::now();

   
    let mut tree=DinoTreeBuilder::new(axgeom::XAXISS,bots,|b|axgeom::Rect::from_point(b.pos,vec2same(5)) ).build_par();

    
    colfind::QueryBuilder::new(&mut tree).query_par(|mut a,mut b| {
        a.inner.num+=1;
        b.inner.num+=1;
    });
    
    black_box(tree);

    instant_to_sec(instant.elapsed())
}


fn test3(bots:&mut Vec<Bot>)->f64{
    
    let instant=Instant::now();

   
    let mut tree=DinoTreeDirectBuilder::new(axgeom::XAXISS,bots,|b|axgeom::Rect::from_point(b.pos,vec2same(5))).build_seq();

    
    colfind::QueryBuilder::new(&mut tree).query_seq(|mut a,mut b| {
        a.inner.num+=1;
        b.inner.num+=1;
    });

    tree.into_inner(bots);

    instant_to_sec(instant.elapsed())
}


fn test4(bots:&mut Vec<Bot>)->f64{
    
    let instant=Instant::now();

   
    let mut tree=DinoTreeDirectBuilder::new(axgeom::XAXISS,bots,|b|axgeom::Rect::from_point(b.pos,vec2same(5))).build_par();

    
    colfind::QueryBuilder::new(&mut tree).query_par(|mut a,mut b| {
        a.inner.num+=1;
        b.inner.num+=1;
    });

    tree.into_inner(bots);

    instant_to_sec(instant.elapsed())
}


fn test5(bots:&mut [BBox<isize,Bot>])->f64{
    
    let instant=Instant::now();

   
    let mut tree=DinoTreeIndirectBuilder::new(axgeom::XAXISS,bots).build_seq();

    
    colfind::QueryBuilder::new(&mut tree).query_seq(|mut a,mut b| {
        a.inner.num+=1;
        b.inner.num+=1;
    });

    instant_to_sec(instant.elapsed())
}


fn test6(bots:&mut [BBox<isize,Bot>])->f64{
    
    let instant=Instant::now();

   
    let mut tree=DinoTreeIndirectBuilder::new(axgeom::XAXISS,bots).build_seq();

    
    colfind::QueryBuilder::new(&mut tree).query_par(|mut a,mut b| {
        a.inner.num+=1;
        b.inner.num+=1;
    });

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
        const NAMES:&[&str]=&["Dinotree Seq","Dinotree Par","Direct Seq","Direct Par","Indirect Seq","Indirect Par"];
        {
            let k=fg.axes2d()
                .set_title(&"Dinotree vs Direct vs Indirect".to_string(), &[])
                .set_legend(Graph(1.0),Graph(1.0),&[LegendOption::Horizontal],&[])
                .set_x_label("Number of Elements", &[])
                .set_y_label("Number of Comparisons", &[]);

            let x=records.iter().map(|a|a.num_bots);
            for index in 0..6{
                let y=records.iter().map(|a|a.arr[index]);
                k.lines(x.clone(),y,&[Caption(NAMES[index]),Color(COLS[index]),LineWidth(1.0)]);
            }
        }
    }
}



fn handle_num_bots(fb:&mut FigureBuilder,grow:f32){
    
    let s=dists::spiral::Spiral::new([400.0,400.0],17.0,grow);
    let mut rects=Vec::new();

    for num_bots in (0..150_000).rev().step_by(200){
        
        let mut bots2:Vec<BBox<isize,Bot>>=s.clone().take(num_bots).map(|pos|{
            let inner=Bot{num:0,pos:pos.inner_as(),_val:[0;ARR_SIZE]};
            let rect=axgeom::Rect::from_point(inner.pos,vec2same(5));
            BBox{rect,inner}
        }).collect();
        

        let mut bots:Vec<Bot>=s.clone().take(num_bots).map(|pos|{
            Bot{num:0,pos:pos.inner_as(),_val:[0;ARR_SIZE]}
        }).collect();

        let arr=[
            test1(&mut bots),
            test2(&mut bots),
            test3(&mut bots),
            test4(&mut bots),
            test5(&mut bots2),
            test6(&mut bots2)];

        let r=Record{num_bots,arr};
        rects.push(r);      
    }

    let mut fg= fb.build(&format!("dinotree_direct_indirect{}",grow));
    
    Record::draw(&rects,&mut fg);
    
    fb.finish(fg);
}
