use crate::inner_prelude::*;


#[derive(Copy,Clone)]
pub struct Bot{
    num:usize,
    pos:Vec2<isize>
}

struct Res{
    num_pairs:usize,
    num_comparison:usize
}

fn test1(bots:&mut [Bot])->Res{
    let mut counter=datanum::Counter::new();

    let mut tree=DinoTreeBuilder::new(axgeom::XAXISS,bots,|b|{
        datanum::from_rect(&mut counter,axgeom::Rect::from_point(b.pos,vec2same(5)))  
    }).build_seq();

    let mut num_pairs=0;

    colfind::QueryBuilder::new(&mut tree).query_seq(|_a, _b| {
        num_pairs+=1;
    });

    Res{num_pairs,num_comparison:counter.into_inner()}
}

fn test2(bots:&mut [Bot])->Res{
    let mut counter=datanum::Counter::new();

    let mut bb:Vec<BBox<datanum::DataNum<_>,Bot>>=bots.iter().map(|b|{   
        let rect=datanum::from_rect(&mut counter,axgeom::Rect::from_point(b.pos,vec2same(5)));  
        BBox::new(rect,*b)
    }).collect();
    

    let mut num_pairs=0;
    colfind::query_sweep_mut(axgeom::XAXISS,&mut bb,|_a, _b| {
        num_pairs+=1;
    });
    
    for (i,j) in bb.into_iter().zip(bots.iter_mut()){
        *j=i.inner;
    }

    Res{num_pairs,num_comparison:counter.into_inner()}
}

fn test3(bots:&mut [Bot])->Res{
    let mut counter=datanum::Counter::new();

    let mut bb:Vec<BBox<datanum::DataNum<_>,Bot>>=bots.iter().map(|b|{   
        let rect=datanum::from_rect(&mut counter,axgeom::Rect::from_point(b.pos,vec2same(5)));  
        BBox::new(rect,*b)
    }).collect();
    

    let mut num_pairs=0;
    colfind::query_naive_mut(ElemSliceMut::new(ElemSlice::from_slice_mut(&mut bb)),|_a, _b| {
        num_pairs+=1;
    });
    
    for (i,j) in bb.into_iter().zip(bots.iter_mut()){
        *j=i.inner;
    }

    Res{num_pairs,num_comparison:counter.into_inner()}
}

#[derive(Debug)]
struct Record {
    num_bots: usize,
    grow: f32,
    num_pairs:usize,
    z1: usize,
    z2: usize,
    z3: Option<usize>
}








fn handle_spiral(fb:&mut FigureBuilder){
    let mut rects=Vec::new();

    for num_bots in (0..10000).step_by(1000){

        for grow in (0usize..100).map(|a|{let a:f32=a as f32;0.0005+a*0.0001}){//0.001 to 0.002
            let s=dists::spiral::Spiral::new([400.0,400.0],17.0,grow);

            let mut bots:Vec<Bot>=s.take(num_bots).map(|pos|{
                Bot{num:0,pos:pos.inner_as()}
            }).collect();

            let z1=test1(&mut bots);
            let z2=test2(&mut bots);
            let z3=if num_bots<8000{
                Some(test3(&mut bots))
            }else{
                None
            };

            black_box(bots.drain(..).map(|a|a.num).count());

            let num_pairs={
                assert_eq!(z1.num_pairs,z2.num_pairs);
                if let Some(z3)=&z3{
                    assert_eq!(z2.num_pairs,z3.num_pairs);    
                }
                z1.num_pairs
            };
            
            
            let z1=z1.num_comparison;
            let z2=z2.num_comparison;
            let z3=z3.map(|a|a.num_comparison);
            let r=Record{num_bots,grow,num_pairs,z1,z2,z3};
            rects.push(r);      
            
        }
    }
    draw_rects(&mut rects,fb,"3d_colfind_num_pairs");       
}
/*
fn handle_spiral_two(fb:&mut FigureBuilder){
    let mut rects=Vec::new();

    for num_bots in (0..10000).step_by(1000){
        for grow in (0..100).map(|a|{let a:f32=a.as_();0.2+a*0.1}){
            let s=dists::spiral::Spiral::new([400.0,400.0],17.0,grow);

            
            let mut bots:Vec<Bot>=s.take(num_bots).map(|pos|{
                let pos=[pos[0] as isize,pos[1] as isize];
                Bot{num:0,pos}
            }).collect();

            let z1=test1(&mut bots);
            let z2=test2(&mut bots);
            let z3=if num_bots<3000{
                Some(test3(&mut bots))
            }else{
                None
            };

            let num_pairs={
                assert_eq!(z1.num_pairs,z2.num_pairs);
                if let Some(z3)=&z3{
                    assert_eq!(z2.num_pairs,z3.num_pairs);    
                }
                z1.num_pairs
            };
            
            
            let z1=z1.num_comparison;
            let z2=z2.num_comparison;
            let z3=z3.map(|a|a.num_comparison);
            let r=Record{num_bots,grow,num_pairs,z1,z2,z3};
            rects.push(r);      
            
        }
    }
    draw_rects(&mut rects,fb,"3d_colfind_num_pairs","3d_colfind_num_comparisons");    
}
*/


fn draw_rects(rects:&mut [Record],fb:&mut FigureBuilder,name1:&str){
    {
        let x=rects.iter().map(|a|a.num_bots as f32);
        let y=rects.iter().map(|a|a.grow);
        let z1=rects.iter().map(|a|a.z1 as f32);
        let z2=rects.iter().map(|a|a.z2 as f32);

        
        let (x2,y2,z3)={

            let ii=rects.iter().filter(|a|a.z3.is_some());
            let x=ii.clone().map(|a|a.num_bots as f32);
            let y=ii.clone().map(|a|a.grow as f32);
            let z3=ii.clone().map(|a|a.z3.unwrap());

            (x,y,z3)
        };
        

        let mut fg=fb.build(name1);

        fg.axes3d().set_view(110.0,30.0)
            .set_title("Comparison of Sweep and Prune versus Dinotree", &[])
            .set_x_label("Number of Objects", &[])
            .set_y_label("Spareness of Objects", &[])
            .set_z_label("Number of Comparisons", &[Rotate(90.0),TextOffset(-3.0,0.0)])
            .points(x.clone(), y.clone(), z1.clone(), &[Caption("Dinotree"),PointSymbol('O'), Color("violet"), PointSize(1.0)])
            .points(x.clone(), y.clone(), z2.clone(), &[Caption("Sweep and Prune"),PointSymbol('o'), Color("red"), PointSize(1.0)])
            .points(x2.clone(), y2.clone(), z3.clone(), &[Caption("Naive"),PointSymbol('o'), Color("green"), PointSize(0.5)]);

        fb.finish(fg);
    }
}

pub fn handle(fb:&mut FigureBuilder){
    handle_spiral(fb); 
}
