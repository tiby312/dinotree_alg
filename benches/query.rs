
#![feature(test)]
extern crate test;
extern crate dinotree;





#[bench]
fn query(b:&mut test::Bencher){
	let radius=5;
	let grow=1.0;
    let s=dists::spiral::Spiral::new([400.0,400.0],17.0,grow);
	
	#[derive(Copy,Clone)]
	struct Bot{
		pos:[isize;2],
		num:usize
	}

	fn aabb_create_isize(pos:[isize;2],radius:isize)->axgeom::Rect<isize>{
        axgeom::Rect::new(pos[0]-radius,pos[0]+radius,pos[1]-radius,pos[1]+radius)
    }
    //unsafe{BBox::new(aabb_create_isize(pos,5),())}
    let mut bots:Vec<_>=s.as_isize().take(200_000).map(|pos|Bot{pos,num:0}).collect();
    
    b.iter(||{
    	let mut tree=dinotree::DinoTreeBuilder::new(axgeom::XAXISS,&mut bots,|b|aabb_create_isize(b.pos,radius)).build_seq();

        dinotree_alg::colfind::QueryBuilder::new(tree.as_ref_mut()).query_seq(|a,b|{
            a.inner.num+=1;
            b.inner.num+=1;
        });

        test::black_box(tree);
	});
}