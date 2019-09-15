use crate::inner_prelude::*;



pub trait TestTrait:Copy+Send+Sync{}
impl<T:Copy+Send+Sync> TestTrait for T{}


#[derive(Copy,Clone)]
pub struct Bot<T>{
    num:usize,
    pos:Vec2<isize>,
    _val:T
}


#[derive(Copy,Clone,Debug)]
pub struct TestResult{
    rebal:f64,
    query:f64
}


fn test_seq<T:HasAabb>(bots:&mut [T],func:impl Fn(ProtectedBBox<T>,ProtectedBBox<T>))->TestResult{
    let instant=Instant::now();

    let mut tree=DinoTreeBuilder::new(axgeom::XAXISS,bots).build_seq();
    
    let rebal=instant_to_sec(instant.elapsed());

    colfind::QueryBuilder::new(&mut tree).query_seq(|mut a,mut b| {
        func(a,b);
    });

    black_box(tree);

    let total = instant_to_sec(instant.elapsed());

    TestResult{rebal,query:total-rebal}
}
fn test_par<T:HasAabb+Send+Sync>(bots:&mut [T],func:impl Fn(ProtectedBBox<T>,ProtectedBBox<T>)+Send+Sync)->TestResult{
    let instant=Instant::now();

    let mut tree=DinoTreeBuilder::new(axgeom::XAXISS,bots).build_par();
    
    let rebal=instant_to_sec(instant.elapsed());

    colfind::QueryBuilder::new(&mut tree).query_par(|mut a,mut b| {
        func(a,b);
    });

    black_box(tree);

    let total = instant_to_sec(instant.elapsed());

    TestResult{rebal,query:total-rebal}
}



#[derive(Copy,Clone,Debug)]
pub struct CompleteTestResult{
    direct_seq:TestResult,
    direct_par:TestResult,
    
    indirect_seq:TestResult,
    indirect_par:TestResult,
    
    default_seq:TestResult,
    default_par:TestResult
}
impl CompleteTestResult{
    fn into_arr(self)->[TestResult;6]{
        [self.direct_seq,self.direct_par,self.indirect_seq,self.indirect_par,self.default_seq,self.default_par]
    }
}

fn complete_test<T:TestTrait>(bots:&mut [Bot<T>])->CompleteTestResult{
    let aabb_make=|b:&Bot<T>|axgeom::Rect::from_point(b.pos,vec2same(5));
    
    
    let (direct_seq,direct_par) = {
        let mut direct:Vec<_>=bots.iter().map(|a|BBox::new(aabb_make(a),*a)).collect();
        
        let collide=|mut b:ProtectedBBox<BBox<isize,Bot<T>>>,mut c:ProtectedBBox<BBox<isize,Bot<T>>>|{
            b.inner_mut().num+=1;
            c.inner_mut().num+=1;
        };

    
        (
            test_seq(&mut direct,collide),
            test_par(&mut direct,collide)
        )
    };

    let (indirect_seq,indirect_par) = {
        let mut direct:Vec<_>=bots.iter().map(|a|BBox::new(aabb_make(a),*a)).collect();
        let mut indirect:Vec<_>=direct.iter_mut().map(|a|BBoxIndirect::new(a)).collect();

    
        let collide=|mut b:ProtectedBBox<BBoxIndirect<BBox<isize,Bot<T>>>>,mut c:ProtectedBBox<BBoxIndirect<BBox<isize,Bot<T>>>>|{
            b.inner_mut().num+=1;
            c.inner_mut().num+=1;
        };

        (
            test_seq(&mut indirect,collide),
            test_par(&mut indirect,collide)
        )
    };
    let (default_seq,default_par) = {
        let mut default=create_bbox_mut(bots,aabb_make);


        let collide=|mut b:ProtectedBBox<BBoxMut<isize,Bot<T>>>,mut c:ProtectedBBox<BBoxMut<isize,Bot<T>>>|{
            b.inner_mut().num+=1;
            c.inner_mut().num+=1;
        };

    
        (
            test_seq(&mut default,collide),
            test_par(&mut default,collide)
        )

    };

    CompleteTestResult{
        direct_seq,
        direct_par,
        indirect_seq,
        indirect_par,
        default_seq,
        default_par
    }
}



pub fn handle(fb:&mut FigureBuilder){ 
    handle_num_bots(fb,0.1,[0u8;8],"8 bytes");
    handle_num_bots(fb,0.1,[0u8;32],"32 bytes");
    handle_num_bots(fb,0.1,[0u8;128],"128 bytes");
    handle_num_bots(fb,0.1,[0u8;256],"256 bytes");


    handle_num_bots(fb,0.01,[0u8;128],"128 bytes");
    handle_num_bots(fb,1.0,[0u8;128],"128 bytes");
    
    
}



#[derive(Debug)]
struct Record {
    num_bots:usize,
    arr:CompleteTestResult    
}
impl Record{
    fn draw(records:&[Record],fg:&mut Figure,grow:f32,name:&str,func:impl Fn(TestResult)->f64){
        const NAMES:&[&str]=&["direct seq","direct par","indirect seq","indirect par","default seq","default par"];
        {
            let k=fg.axes2d()
                .set_title(&format!("Dinotree vs Direct vs Indirect with grow {} and {}",grow,name), &[])
                .set_legend(Graph(1.0),Graph(1.0),&[LegendOption::Horizontal],&[])
                .set_x_label("Number of Elements", &[])
                .set_y_label("Number of Comparisons", &[]);

            let x=records.iter().map(|a|a.num_bots);
            for index in 0..6{
                let y=records.iter().map(|a|func(a.arr.into_arr()[index]));
                k.lines(x.clone(),y,&[Caption(NAMES[index]),Color(COLS[index]),LineWidth(1.0)]);
            }
        }
    }
}



fn handle_num_bots<T:TestTrait>(fb:&mut FigureBuilder,grow:f32,val:T,name:&str){
    
    let s=dists::spiral::Spiral::new([400.0,400.0],17.0,grow);
    let mut rects=Vec::new();

    for num_bots in (0..40_000).rev().step_by(500){
        
        let mut bots:Vec<Bot<T>>=s.clone().take(num_bots).map(|pos|{
            Bot{num:0,pos:pos.inner_as(),_val:val.clone()}
        }).collect();


        let r=Record{num_bots,arr:complete_test(&mut bots)};
        rects.push(r);      
    }


    let mut fg= fb.build(&format!("dinotree_direct_indirect_rebal_{}_{}",grow,name));
    Record::draw(&rects,&mut fg,grow,name,|a|a.rebal);
    fb.finish(fg);


    let mut fg= fb.build(&format!("dinotree_direct_indirect_query_{}_{}",grow,name));
    Record::draw(&rects,&mut fg,grow,name,|a|a.query);
    fb.finish(fg);


}

