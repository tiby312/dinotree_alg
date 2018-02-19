//! Module with code to support test code.

use axgeom;
use rand;
use std;


#[derive(Copy,Clone,Debug)]
pub struct Bot{
    pub id:usize,
    pub col:usize
}

impl Bot{
    pub fn new(a:usize)->Bot{
        Bot{id:a,col:0}
    }
}

pub fn create_unordered(a:&Bot,b:&Bot)->(usize,usize){
    if a.id<b.id{
        (a.id,b.id)
    }else{
        (b.id,a.id)
    }
}

pub fn compair_bot_pair(a:&(usize,usize),b:&(usize,usize))->std::cmp::Ordering{
    if a.0<b.0{
        std::cmp::Ordering::Less
    }else if a.0>b.0{
        std::cmp::Ordering::Greater
    }else{
        if a.1<b.1{
            std::cmp::Ordering::Less
        }else if a.1>b.1{
            std::cmp::Ordering::Greater
        }else{
            std::cmp::Ordering::Equal
        }
    }
}


use ordered_float::NotNaN;
use support::BBox;
use rand::{ SeedableRng, StdRng};
use support::Numf32;


pub fn create_bots(world:&axgeom::Rect<Numf32>,num_bots:usize,seed:&[usize])->Vec<BBox<Numf32,Bot>>{
    use rand::distributions::IndependentSample;
    
    let mut rng: StdRng = SeedableRng::from_seed(seed);

    let vect:Vec<BBox<Numf32,Bot>>=(0..num_bots).map(|a|
    {
        let d=Numf32(std::default::Default::default());
        let mut new_rect=axgeom::Rect::new(d,d,d,d);
        for axis in axgeom::AxisIter::new(){
            let rr=world.get_range(axis);

            let aa=rand::distributions::Range::new(rr.start.0.into_inner(),rr.end.0.into_inner()); 
        
            let g1=Numf32(NotNaN::new(aa.ind_sample(&mut rng)).unwrap());
            let g2=Numf32(NotNaN::new(aa.ind_sample(&mut rng)).unwrap());
            if g1<g2{
                let j=new_rect.get_range_mut(axis);
                j.start=g1;
                j.end=g2;
            }else{
                let j=new_rect.get_range_mut(axis);
                j.start=g2;
                j.end=g1;
            }
        }

        let bot=Bot::new(a);
        BBox::new(bot,new_rect)
    }
        ).collect();

    vect
    
}

pub fn create_word()->axgeom::Rect<Numf32>{
    axgeom::Rect::new(
        Numf32(NotNaN::new(-1000.0).unwrap()),
        Numf32(NotNaN::new(1000.0).unwrap()),
        Numf32(NotNaN::new(-1000.0).unwrap()),
        Numf32(NotNaN::new(1000.0).unwrap()),
        )
}
pub fn get_random_rect(world:&axgeom::Rect<NotNaN<f32>>)->axgeom::Rect<NotNaN<f32>>{
    use rand::distributions::IndependentSample;
    let mut rng=rand::thread_rng();

    //let mut new_rect=axgeom::Rect::new(0.0,0.0,0.0,0.0);
    let d=std::default::Default::default();
    let mut new_rect=axgeom::Rect::new(d,d,d,d);
        
    for axis in axgeom::AxisIter::new() {
        let rr=world.get_range(axis);

        let a=rand::distributions::Range::new(rr.start.into_inner(),rr.end.into_inner()); 
        
        let g1=NotNaN::new(a.ind_sample(&mut rng)).unwrap();
        let g2=NotNaN::new(a.ind_sample(&mut rng)).unwrap();
        if g1<g2{
            let j=new_rect.get_range_mut(axis);
            j.start=g1;
            j.end=g2;
        }else{
            let j=new_rect.get_range_mut(axis);
            j.start=g2;
            j.end=g1;
        }
        
    }
    new_rect
}