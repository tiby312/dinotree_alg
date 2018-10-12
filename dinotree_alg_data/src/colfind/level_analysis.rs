

use inner_prelude::*;
use std::time::Instant;
use dinotree_alg::colfind;

#[derive(Copy,Clone)]
pub struct Bot{
    pos:[isize;2],
    num:usize
}


mod timer{
	use super::*;

	fn into_secs(elapsed:std::time::Duration)->f64{
		let sec = (elapsed.as_secs() as f64) + (elapsed.subsec_nanos() as f64 / 1000_000_000.0);
    	sec
	}
	pub struct LevelTimer{
		levels:Vec<f64>,
		time:Option<Instant>,
		height:usize
	}

	impl LevelTimer{
		pub fn new(height:usize)->LevelTimer{
			
			LevelTimer{levels:Vec::new(),time:None,height}
		}
		pub fn into_inner(self)->Vec<f64>{
			self.levels
		}
		fn node_end_common(&mut self){

    		let time=self.time.unwrap();

    		let elapsed=time.elapsed();
			self.levels.push(into_secs(elapsed));
			self.time=None;
		}
	}
	impl Splitter for LevelTimer{
		fn div(mut self)->(Self,Self){
			self.node_end_common();

			let height=self.height;
			let length=self.levels.len();

			(self,LevelTimer{levels:std::iter::repeat(0.0).take(length).collect(),time:None,height})
		}
		fn add(mut self,a:Self)->Self{
			assert_eq!(self.levels.len(),a.levels.len());
			for (a,b) in self.levels.iter_mut().zip(a.levels.iter()){
				*a+=*b;
			}
			self
		}
		fn node_start(&mut self){
			assert!(self.time.is_none());
			self.time=Some(Instant::now());
		}
    	fn node_end(&mut self){
    		self.node_end_common();
		} 
	}
}


pub fn handle(fb:&FigureBuilder){
    let mut fg=fb.new("colfind_theory");

    let s=dists::spiral::Spiral::new([400.0,400.0],12.0,1.5);

    let num_bots=100_000;

    let mut bots:Vec<Bot>=s.take(num_bots).enumerate().map(|(_e,pos)|{
        let pos=[pos[0] as isize,pos[1] as isize];
        Bot{num:0,pos}
    }).collect();
    

	let mut tree=DynTree::new_seq(axgeom::XAXISS,(),&bots,|b|{
        aabb_from_point_isize(b.pos,[5,5])  
    });

	struct Bo{};
	impl colfind::ColMulti for Bo{
		type T=BBox<isize,Bot>;
		fn collide(&mut self,a:&mut Self::T,b:&mut Self::T){

		}
	}

	let height=tree.get_height();
	let leveltimer=timer::LevelTimer::new(height);
    let times=colfind::query_seq_adv_mut(&mut tree,Bo{},leveltimer);
    
    let times=times.1.into_inner();
    println!("times={:?}",times);

    tree.apply_orig_order(&mut bots,|a,b|{
        *b=a.inner;
    });





    //fg.show();
}


