use axgeom::rect;
use dinotree_alg::prelude::*;

fn main(){
	let mut aabbs=[
		BBox::new(rect(0isize,10,0,10),0),    
		BBox::new(rect(15,20,15,20),0), 
		BBox::new(rect(5,15,5,15),0)
	];
	
	//Create a layer of direction.
	let mut ref_aabbs  = aabbs.iter_mut().collect::<Vec<_>>();

	//Create a DinoTree by picking a starting axis (x or y).
	//This will change the order of the elements in bboxes,
	//but this is okay since we populated it with mutable references.	
	let mut tree=DinoTree::new(&mut ref_aabbs);

	//Find all colliding aabbs.
	tree.find_collisions_mut(|mut a,mut b|{
		*a.inner_mut()+=1;
		*b.inner_mut()+=1;
	});

	assert_eq!(aabbs[0].inner, 1);
	assert_eq!(aabbs[1].inner, 0);
	assert_eq!(aabbs[2].inner, 1);
}
