





pub fn handle(fb:&FigureBuilder){
 
	let mut tree=DynTree::new_seq(axgeom::XAXISS,(),&bots,|b|{
        datanum::from_rect(&mut counter,aabb_from_point_isize(b.pos,[5,5]))  
    });

	struct Bo{};
	impl ColMulti for Bo{
		type T=BBox<Box,isize>;
		fn collide(&mut self,a:&mut Self::T,b:&mut Self::T){

		}
	}

	struct Splitter{
		height:usize
	}
	
    colfind::query_seq_adv_mut(&mut tree,Bo{},splitter);
    
    tree.apply_orig_order(&mut bots,|a,b|{
        *b=a.inner;
    });




    let s=dists::spiral::Spiral::new([400.0,400.0],12.0,1.5);

    let mut fg=fb.new("colfind_theory");
    handle_theory(&s,&mut fg);
    handle_bench(&s,&mut fg);

    fg.show();
}


