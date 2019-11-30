Provides various query aabb broad phase algorithms such as collision pair finding, raycast, or k_nearest. 

### Inner projects

The dinotree_alg_demo inner project is meant to show case the use of these algorithms. It depends on the piston 2d engine to draw to the screen. 

The dinotree_alg_data project generates some graphs using RustGnuPlot. These graphs are used to create the reports in the dinotree_report project that is a seperate project.

### Analysis

Please see the [dinotree_report](https://github.com/tiby312/dinotree_report) github project for a writeup of the design and analysis of the algorithms in this project.

### Example

```rust
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

	//Create a DinoTree by picking a starting axis.
	//This will change the order of the elements in bboxes,
	//but this is okay since we populated it with mutable references.	
	let mut tree=DinoTree::new(axgeom::XAXISS,&mut ref_aabbs);

	//Find all colliding aabbs.
	tree.find_collisions_mut(|mut a,mut b|{
		*a.inner_mut()+=1;
		*b.inner_mut()+=1;
	});

	assert_eq!(aabbs[0].inner, 1);
	assert_eq!(aabbs[1].inner, 0);
	assert_eq!(aabbs[2].inner, 1);
}
```