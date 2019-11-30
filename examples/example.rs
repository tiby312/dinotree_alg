

use axgeom::rect;
use axgeom::vec2;
use dinotree_alg::prelude::*;




fn main(){

	let mut aabbs=[
		BBox::new(rect(0isize,10,0,10),0),    
		BBox::new(rect(15,20,15,20),0), 
		BBox::new(rect(5,15,5,15),0)
	];


	{
		//Create a layer of direction.
		let mut ref_aabbs  = aabbs.iter_mut().collect::<Vec<_>>();

		//Create a DinoTree by picking a starting axis (x or y).
		//This will change the order of the elements in bboxes,
		//but this is okay since we populated it with mutable references.	
		let mut tree=DinoTree::new(axgeom::XAXISS,&mut ref_aabbs);

		//Find all colliding aabbs.
		tree.find_collisions_mut(|mut a,mut b|{
			*a.inner_mut()+=1;
			*b.inner_mut()+=1;
		});

		//TODO work on this.
		//let res = tree.k_nearest_mut(vec2(0,0),2,&mut KnearestSimple::new(|p:axgeom::Vec2<isize>,r:&axgeom::Rect<isize>|(r.x.start -p.x).abs() ),rect(0,100,0,100));


		//Here we query for read-only references so we can pull
		//them out of the closure.
		let mut rect_collisions = Vec::new();
		tree.for_all_intersect_rect(&rect(-5,1,-5,1),|a|{
			rect_collisions.push(a);
		});

		assert_eq!(rect_collisions.len(),1);
		assert_eq!(*rect_collisions[0].get(),rect(0,10,0,10));
	}

	assert_eq!(aabbs[0].inner, 1);
	assert_eq!(aabbs[1].inner, 0);
	assert_eq!(aabbs[2].inner, 1);
}

