

use crate::inner_prelude::*;
use dinotree_alg::colfind;


	



#[derive(Copy,Clone)]
pub struct Bot{
    pos:[isize;2],
    num:usize
}

/*
#[derive(Copy,Clone)]
enum RebalOrQuery{
	Rebal,
	Query
}
*/

mod level_counter{
	use crate::datanum;
	use dinotree::advanced::Splitter;

	pub struct LevelCounter{
		counter:*mut datanum::Counter,
		cursor:Option<usize>,
	    levels:Vec<usize>,
	}

	impl LevelCounter{
	    pub fn new(counter:*mut datanum::Counter)->LevelCounter{
	        LevelCounter{counter,levels:Vec::new(),cursor:None}
	    }
	    pub fn into_inner(self)->Vec<usize>{
	        self.levels
	    }
	    fn node_end_common(&mut self){
	    	let counter=unsafe{&mut *self.counter};
	    	let nc=counter.into_inner();

	        let elapsed=nc-self.cursor.unwrap();
	        self.levels.push(elapsed);
	        self.cursor=None;
	    }
	}
	impl Splitter for LevelCounter{
	    fn div(&mut self)->Self{
	        self.node_end_common();

	        let length=self.levels.len();
	        let counter=self.counter;
	        LevelCounter{counter,levels:std::iter::repeat(0).take(length).collect(),cursor:None}
	    }
	    fn add(&mut self,a:Self){
	    	let len=self.levels.len();
	        for (a,b) in self.levels.iter_mut().zip(a.levels.iter()){
	            *a+=*b;
	        }
	        if len<a.levels.len(){
	            self.levels.extend_from_slice(&a.levels[len..]);
	        }
	    	/*
	        let (smaller,mut larger)=if self.levels.len()<a.levels.len(){
	            (self,a)
	        }else{
	            (a,self)
	        };


	        for (a,b) in larger.levels.iter_mut().zip(smaller.levels.iter()){
	            *a+=*b;
	        }
	        larger
	        */
	    }
	    fn node_start(&mut self){
	    	let counter=unsafe{&mut *self.counter};
	    	self.cursor=Some(counter.into_inner());
	    }
	    fn node_end(&mut self){
	        self.node_end_common();
	    } 
	}
}


struct TheoryRes{
	grow:f64,
	rebal:Vec<usize>,
	query:Vec<usize>
}

fn handle_inner_theory(num_bots:usize,grow_iter:impl Iterator<Item=f64>)->Vec<TheoryRes>{
	let mut rects=Vec::new();
    for grow in grow_iter{
               
	    let s=dists::spiral::Spiral::new([400.0,400.0],12.0,grow);

	    //let num_bots=10_000;

	    let mut bots:Vec<Bot>=s.take(num_bots).enumerate().map(|(_e,pos)|{
	        let pos=[pos[0] as isize,pos[1] as isize];
	        Bot{num:0,pos}
	    }).collect();
	

	    {
	    	let mut counter=datanum::Counter::new();

		    let mut levelc=level_counter::LevelCounter::new(&mut counter);


			let mut tree=DinoTreeBuilder::new(axgeom::XAXISS,(),&bots,|b|{
        		datanum::from_rect(&mut counter,aabb_from_point_isize(b.pos,[5,5]))  
			}).build_with_splitter_seq(&mut levelc);
	

			counter.reset();
			let mut levelc2=level_counter::LevelCounter::new(&mut counter);
			colfind::QueryBuilder::new(tree.as_ref_mut()).query_with_splitter_seq(|a,b|{
				a.inner.num+=1;
				b.inner.num+=1;
			},&mut levelc2);


		    counter.into_inner();


		    tree.apply(&mut bots,|a,b|{
		        *b=a.inner;
		    });


		    let mut t=TheoryRes{grow,rebal:levelc.into_inner(),query:levelc2.into_inner()};
		    let height=tree.as_ref().height();

		    grow_to_fit(&mut t.rebal,height);
			grow_to_fit(&mut t.query,height);

		    assert_eq!(t.rebal.len(),t.query.len());
		    rects.push(t)
	    }
	}
	rects
}
struct BenchRes{
	grow:f64,
	rebal:Vec<f64>,
	query:Vec<f64>
}

fn handle_inner_bench(num_bots:usize,grow_iter:impl Iterator<Item=f64>)->Vec<BenchRes>{
	let mut rects=Vec::new();
    for grow in grow_iter{
               
	    let s=dists::spiral::Spiral::new([400.0,400.0],12.0,grow);

	    //let num_bots=10_000;

	    let mut bots:Vec<Bot>=s.take(num_bots).enumerate().map(|(_e,pos)|{
	        let pos=[pos[0] as isize,pos[1] as isize];
	        Bot{num:0,pos}
	    }).collect();
	    
	    let height=compute_tree_height_heuristic(num_bots);
		let mut times1=LevelTimer::new();

		

		let mut tree=DinoTreeBuilder::new(axgeom::XAXISS,(),&bots,|b|{
		        aabb_from_point_isize(b.pos,[5,5])  
		}).build_with_splitter_seq(&mut times1);

		
		/*
		let mut tree=dinotree::advanced::NotSorted::new_adv_seq(axgeom::XAXISS,(),&bots,|b|{
			        aabb_from_point_isize(b.pos,[5,5])  
			    },height,&mut times1);
		*/


		let mut times2=LevelTimer::new();
		colfind::QueryBuilder::new(tree.as_ref_mut()).query_with_splitter_seq(|a,b|{a.inner.num+=1;b.inner.num+=1},&mut times2);
		//colfind::query_nosort_seq_adv_mut(&mut tree,|a,b|{a.inner.num+=1;b.inner.num+=1},&mut times2);

	    tree.apply(&mut bots,|a,b|{
	        *b=a.inner;
	    });


	    let mut t=BenchRes{grow,rebal:times1.into_inner(),query:times2.into_inner()};
	    let height=tree.as_ref().height();
	    
	    grow_to_fit(&mut t.rebal,height);
		grow_to_fit(&mut t.query,height);

	    assert_eq!(t.rebal.len(),t.query.len());
	    rects.push(t)
    }	
    rects
}

fn grow_to_fit<T:Default>(a:&mut Vec<T>,b:usize){
	let diff=b-a.len();
	for _ in 0..diff{
	  	a.push(std::default::Default::default());
	}
}



pub fn handle(fb:&mut FigureBuilder){
	handle_bench(3000,fb);
	//handle_theory(3000,fb);
}


fn handle_bench(num_bots:usize,fb:&mut FigureBuilder){

    let res1=handle_inner_bench(num_bots,(0..1000).map(|a|0.0005+(a as f64)*0.00001));
	
	let res2=handle_inner_bench(num_bots,(0..1000).map(|a|0.01+(a as f64)*0.00002));


    fn draw_graph(title_name:&str,fg:&mut Figure,res:&Vec<BenchRes>,rebal:bool,pos:usize){
    	//let cols=["blue","green","red","violet","red","orange","pink","gray","brown"];
	
    	let ax=fg.axes2d().set_pos_grid(2,1,pos as u32)
	        .set_title(title_name, &[])
	        .set_x_label("Spiral Grow", &[])
	        .set_y_label("Time taken in Seconds", &[]);
	  
	  	let num=res.first().unwrap().rebal.len();

	  	let x=res.iter().map(|a|a.grow);
    
    	if rebal{
	  		let cc=(0..num).map(|ii:usize|{res.iter().map(move |a|a.rebal[ii])});

		  	for (i,(col,y)) in COLS.iter().cycle().zip( cc   ).enumerate(){
		  		let s=format!("Level {}",i);
		  		ax.lines(x.clone(),y,&[Color(col),Caption(&s),LineWidth(1.0)]);
		  	}
		}else{
			let cc=(0..num).map(|ii:usize|{res.iter().map(move |a|a.query[ii])});
			
		  	for (i,(col,y)) in COLS.iter().cycle().zip( cc   ).enumerate(){
		  		let s=format!("Level {}",i);
		  		ax.lines(x.clone(),y,&[Color(col),Caption(&s),LineWidth(1.0)]);
		  	}
		}
	}
	let mut fg=fb.new("level_analysis_bench_rebal");
	draw_graph(&format!("Rebal Level Bench with {} objects",num_bots),&mut fg,&res1,true,0);
	draw_graph(&format!("Rebal Level Bench with {} objects",num_bots),&mut fg,&res2,true,1);
    fb.finish(fg);
    
	let mut fg=fb.new("level_analysis_bench_query");
	draw_graph(&format!("Query Level Bench with {} objects",num_bots),&mut fg,&res1,false,0);
	draw_graph(&format!("Query Level Bench with {} objects",num_bots),&mut fg,&res2,false,1);
    fb.finish(fg);
}

fn handle_theory(num_bots:usize,fb:&mut FigureBuilder){
	

    let res1=handle_inner_theory(num_bots,(0..100).map(|a|0.0005+(a as f64)*0.0001));
	
	let res2=handle_inner_theory(num_bots,(0..100).map(|a|0.01+(a as f64)*0.0002));


    use gnuplot::*;
    
    
    fn draw_graph(title_name:&str,fg:&mut Figure,res:&Vec<TheoryRes>,rebal:bool,pos:usize){
    	
    	let ax=fg.axes2d().set_pos_grid(2,1,pos as u32)
	        .set_title(title_name, &[])
	        .set_x_label("Spiral Grow", &[])
	        .set_y_label("Number of Comparisons", &[]);
	  
	  	let num=res.first().unwrap().rebal.len();


	  	let x=res.iter().map(|a|a.grow);
    
    	if rebal{
	  		let cc=(0..num).map(|ii:usize|{res.iter().map(move |a|a.rebal[ii])});

		  	for (i,(col,y)) in COLS.iter().cycle().zip( cc   ).enumerate(){
		  		let s=format!("Level {}",i);
		  		ax.lines(x.clone(),y,&[Color(col),Caption(&s),LineWidth(1.0)]);
		  	}
		}else{
			let cc=(0..num).map(|ii:usize|{res.iter().map(move |a|a.query[ii])});
			
		  	for (i,(col,y)) in COLS.iter().cycle().zip( cc   ).enumerate(){
		  		let s=format!("Level {}",i);
		  		ax.lines(x.clone(),y,&[Color(col),Caption(&s),LineWidth(1.0)]);
		  	}
		}
	}

	let mut fg=fb.new("level_analysis_theory_rebal");
	draw_graph(&format!("Rebal Level Comparisons with {} Objects",num_bots),&mut fg,&res1,true,0);
	draw_graph(&format!("Rebal Level Comparisons with {} Objects",num_bots),&mut fg,&res2,true,1);
    fb.finish(fg);

	let mut fg=fb.new("level_analysis_theory_query");
	draw_graph(&format!("Query Level Comparisons with {} Objects",num_bots),&mut fg,&res1,false,0);
	draw_graph(&format!("Query Level Comparisons with {} Objects",num_bots),&mut fg,&res2,false,1);
    fb.finish(fg);

}
