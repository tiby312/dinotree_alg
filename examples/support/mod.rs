


use axgeom;
use std;
use rand;
use rand::{SeedableRng, StdRng};
use rand::distributions::{IndependentSample, Range};
use dinotree::*;
#[derive(Clone, Debug)]
pub struct Bot {
    pub id: usize,
    pub col: Vec<usize>,
}

impl Bot{
    pub fn new(id:usize)->Bot{
        Bot{id,col:Vec::new()}
    }
}

pub fn make_rect(a: (isize, isize), b: (isize, isize)) -> axgeom::Rect<isize> {
    axgeom::Rect::new(a.0, a.1, b.0, b.1)
}

pub fn create_rect_from_point(a: (isize, isize)) -> AABBox<isize> {
    let r: isize = 8;
    let x = a.0;
    let y = a.1;
    AABBox(make_rect((x , x + r), (y , y + r)))
}
pub fn create_unordered(a: &Bot, b: &Bot) -> (usize, usize) {
    if a.id < b.id {
        (a.id, b.id)
    } else {
        (b.id, a.id)
    }
}
pub fn compair_bot_pair(a: &(usize, usize), b: &(usize, usize)) -> std::cmp::Ordering {
    if a.0 < b.0 {
        std::cmp::Ordering::Less
    } else if a.0 > b.0 {
        std::cmp::Ordering::Greater
    } else {
        if a.1 < b.1 {
            std::cmp::Ordering::Less
        } else if a.1 > b.1 {
            std::cmp::Ordering::Greater
        } else {
            std::cmp::Ordering::Equal
        }
    }
}


pub struct PointGenerator {
    rng: StdRng,
    xdist: Range<isize>,
    ydist: Range<isize>,
}
impl PointGenerator {
    pub fn new(a: &axgeom::Rect<isize>, seed: &[usize]) -> PointGenerator {
        let rng: StdRng = SeedableRng::from_seed(seed);

        let rr = a.get_range2::<axgeom::XAXISS>();
        let xdist = rand::distributions::Range::new(rr.start, rr.end);

        let rr = a.get_range2::<axgeom::YAXISS>();
        let ydist = rand::distributions::Range::new(rr.start, rr.end);

        PointGenerator { rng, xdist, ydist }
    }
    pub fn random_point(&mut self) -> (isize, isize) {
        (
            self.xdist.ind_sample(&mut self.rng),
            self.ydist.ind_sample(&mut self.rng),
        )
    }
}
