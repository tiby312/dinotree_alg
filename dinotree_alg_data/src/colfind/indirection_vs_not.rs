use crate::inner_prelude::*;



const ARR_SIZE:usize=20;

#[derive(Copy,Clone)]
pub struct Bot{
    pos:Vec2<isize>,
    num:usize,
    arr:[usize;ARR_SIZE]
}

struct BBox2<A:NumTrait,B>(BBox<A,B>);

unsafe impl HasAabb for BBox2<isize,Bot>{
    type Num=isize;
    type Inner=Bot;
    fn get(&self)->BBoxRef<isize,Bot>{
        BBoxRef::new(&self.0.rect,&self.0.inner)
    }
}

unsafe impl HasAabb for &mut BBox2<isize,Bot>{
    type Num=isize;
    type Inner=Bot;
    fn get(&self)->BBoxRef<isize,Bot>{
        BBoxRef::new(&self.0.rect,&self.0.inner)
    }
}

unsafe impl HasAabbMut for BBox2<isize,Bot>{
    fn get_mut(&mut self)->BBoxRefMut<isize,Bot>{
        BBoxRefMut::new(&self.0.rect,&mut self.0.inner)
    }
}

unsafe impl HasAabbMut for &mut BBox2<isize,Bot>{
    fn get_mut(&mut self)->BBoxRefMut<isize,Bot>{
        BBoxRefMut::new(&self.0.rect,&mut self.0.inner)
    }
}




fn handle_bench_inner(s:&dists::spiral::Spiral,fg:&mut Figure,title:&str){

    #[derive(Debug)]
    struct Record {
        num_bots: usize,
        rect_direct:f64,
        rect_indirect:f64,
        bb:f64,
        direct: f64,
        indirect:f64
    }

    let mut records=Vec::new();

    for num_bots in (0..100_000).rev().step_by(1000){
        let s2=s.clone();


        let mut bots:Vec<Bot>=s2.as_isize().take(num_bots).enumerate().map(|(_e,pos)|{
            Bot{num:0,pos,arr:[0;ARR_SIZE]}
        }).collect();
        



        let a0={
            
            let mut bots2:Vec<Bot>=s.clone().take(num_bots).map(|pos|{
                Bot{pos:pos.inner_as(),num:0,arr:[0;ARR_SIZE]}
            }).collect();
            


            let instant=Instant::now();
            

            let mut tree=DinoTreeBuilder::new(axgeom::XAXISS,&mut bots2,|bot|{
                axgeom::Rect::from_point(bot.pos,vec2same(5))
            }).build_seq();

    
            colfind::QueryBuilder::new(&mut tree).query_seq(|mut a,mut b| {
                a.inner.num+=1;
                b.inner.num+=1;
        
            });

            instant_to_sec(instant.elapsed())
        };

        let a1={
            
            let mut bots2:Vec<Bot>=s.clone().take(num_bots).map(|pos|{
                Bot{pos:pos.inner_as(),num:0,arr:[0;ARR_SIZE]}
            }).collect();
            
            let mut bots2:Vec<*mut Bot>=bots2.iter_mut().map(|a|a as *mut _).collect();


            let instant=Instant::now();
            

            let mut tree=DinoTreeBuilder::new(axgeom::XAXISS,&mut bots2,|bot|{
                let bot:&mut Bot=unsafe{&mut *(*bot)};
                axgeom::Rect::from_point(bot.pos,vec2same(5))
            }).build_seq();

    
            colfind::QueryBuilder::new(&mut tree).query_seq(|mut a,mut b| {
                let a:&mut Bot=unsafe{&mut *(*a.inner)};
                let b:&mut Bot=unsafe{&mut *(*b.inner)};
                a.num+=1;
                b.num+=1;
        
            });
            

            instant_to_sec(instant.elapsed())
        };


        let b0={
            
            let mut bots2:Vec<BBox2<isize,Bot>>=s.clone().take(num_bots).map(|pos|{
                let inner=Bot{pos:pos.inner_as(),num:0,arr:[0;ARR_SIZE]};
                let rect=axgeom::Rect::from_point(inner.pos,vec2same(5));
                BBox2(BBox{rect,inner})
            }).collect();
            

            let mut bots2:Vec<*mut BBox2<isize,Bot>>=bots2.iter_mut().map(|a|a as *mut _).collect();


            let instant=Instant::now();
            

            let mut tree=DinoTreeBuilder::new(axgeom::XAXISS,&mut bots2,|bot|{
                let bot:&mut BBox2<_,_>=unsafe{&mut *(*bot)};
                bot.0.rect
            }).build_seq();

    
            colfind::QueryBuilder::new(&mut tree).query_seq(|mut a,mut b| {
                let a:&mut BBox2<_,_>=unsafe{&mut *(*a.inner)};
                let b:&mut BBox2<_,_>=unsafe{&mut *(*b.inner)};
                a.0.inner.num+=1;
                b.0.inner.num+=1;
        
            });
            

            instant_to_sec(instant.elapsed())
        };

        let c0={
            
            let mut bots2:Vec<BBox2<isize,Bot>>=s.clone().take(num_bots).map(|pos|{
                let inner=Bot{pos:pos.inner_as(),num:0,arr:[0;ARR_SIZE]};
                let rect=axgeom::Rect::from_point(inner.pos,vec2same(5));
                BBox2(BBox{rect,inner})
            }).collect();
            


            let instant=Instant::now();
            

            let mut tree=DinoTreeGenericBuilder::new(axgeom::XAXISS,&mut bots2).build_seq();

    
            colfind::QueryBuilder::new(&mut tree).query_seq(|mut a,mut b| {
                a.inner.num+=1;
                b.inner.num+=1;
        
            });
            

            tree.into_original();

            instant_to_sec(instant.elapsed())
        };

        let c1={


            let mut bots2:Vec<BBox2<isize,Bot>>=s.clone().take(num_bots).map(|pos|{
                let inner=Bot{pos:pos.inner_as(),num:0,arr:[0;ARR_SIZE]};
                let rect=axgeom::Rect::from_point(inner.pos,vec2same(5));
                BBox2(BBox{rect,inner})
            }).collect();
            
            let mut bots3:Vec<&mut BBox2<isize,Bot>>=bots2.iter_mut().collect();


            let instant=Instant::now();
            

            let mut tree=DinoTreeGenericBuilder::new(axgeom::XAXISS,&mut bots3).build_seq();

            
            colfind::QueryBuilder::new(&mut tree).query_seq(|mut a,mut b| {
                a.inner.num+=1;
                b.inner.num+=1;
            });


            instant_to_sec(instant.elapsed())
        };



        records.push(Record{num_bots,rect_direct:a0,rect_indirect:a1,bb:b0,direct:c0,indirect:c1});
    }

    records.reverse();

    let rects=&mut records;
    use gnuplot::*;
    let x=rects.iter().map(|a|a.num_bots);
    let y1=rects.iter().map(|a|a.rect_direct);
    let y2=rects.iter().map(|a|a.rect_indirect);
    let y3=rects.iter().map(|a|a.bb);
    let y4=rects.iter().map(|a|a.direct);
    let y5=rects.iter().map(|a|a.indirect);
        

    fg.axes2d()
        .set_title(title, &[])
        .set_legend(Graph(1.0),Graph(1.0),&[LegendOption::Horizontal],&[])
        .lines(x.clone(), y1,  &[Caption("rect direct"), Color("blue"), LineWidth(2.0)])
        .lines(x.clone(), y2,  &[Caption("rect indirect"), Color("green"), LineWidth(2.0)])
        .lines(x.clone(), y3,  &[Caption("bb"), Color("brown"), LineWidth(2.0)])
        .lines(x.clone(), y4,  &[Caption("direct"), Color("red"), LineWidth(2.0)])
        .lines(x.clone(), y5,  &[Caption("indirect"), Color("yellow"), LineWidth(2.0)])
        .set_x_label("Number of Objects", &[])
        .set_y_label("Time taken in seconds", &[]);

}





pub fn handle_bench(fb:&mut FigureBuilder){
           
    let s1=dists::spiral::Spiral::new([400.0,400.0],12.0,0.05);
    //let s2=dists::spiral::Spiral::new([400.0,400.0],12.0,0.05);

    let mut fg=fb.build("direct_vs_indirect");
    handle_bench_inner(&s1.clone(),&mut fg,"Direct vs Indirect");
    //handle_bench_inner(&s2.clone(),&mut fg,"Comparison of space partitioning algs with abspiral(x,0.05)",1);
    
    fb.finish(fg);
}
