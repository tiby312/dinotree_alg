use axgeom::{Rect,rect,vec2};
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

		let border=rect(0,100,0,100);
		let mut kk=KnearestSimple::new(|p,r|distance(p,r) );
		let res = tree.k_nearest_mut(vec2(30,30),2,&mut kk,border);
		assert_eq!(res[0].bot.get(),&rect(15,20,15,20)  );
		assert_eq!(res[1].bot.get(),&rect(5,15,5,15)  );


		let mut kk=RaycastSimple::new(|r,rr|raycast(r,rr));
		let ray=Ray{point:vec2(-10,1),dir:vec2(1,0)};
		let res = tree.raycast_mut(border,ray,&mut kk);
		assert_eq!(res.unwrap().0[0].get(),&rect(0,10,0,10)  );
		
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

fn distance(p:axgeom::Vec2<isize>,r:&Rect<isize>)->isize{
	//This is not a full implementation of the contract of Knearest trait.
	//In the same of simplicity of just showing the api, it is hardcoded to give the answer we would expect.
	(r.x.start-p.x).abs()
}

fn raycast(ray:&Ray<isize>,r:&Rect<isize>)->RayIntersectResult<isize>{
	//This is not a full implementation of the contract of RayCastTrait.
	//In the same of simplicity of just showing the api, it is hardcoded to give the answer we would expect.
	RayIntersectResult::Hit(r.x.start-ray.point.x)
}

