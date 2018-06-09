
use inner_prelude::*;


use dinotree_inner::*;
use rect;

pub struct RectIntersectErr;

/*
pub struct MultiRect<'a,A: AxisTrait+'a, N:NumTrait+'a,T:'a> {
    tree: &'a mut DynTree<A,(),BBox<N,T>>,
    rects: SmallVec<[Rect<N>; 16]>,
}

impl<'a,A: AxisTrait, N:NumTrait,T> MultiRect<'a,A,N,T>{
	pub fn for_all_in_rect_mut(&mut self,rect:Rect<N>,mut func:impl FnMut(BBoxDet<'a,N,T>))->Result<(),RectIntersectErr>{
		for r in self.rects.iter(){
			if rect.intersects_rect(r){
				return Err(RectIntersectErr);
			}
		}

		self.rects.push(rect);

		rect::for_all_in_rect_mut(self.tree,&rect,|bbox:BBoxDet<N,T>|{
			//This is only safe to do because the user is unable to mutate the bounding box.
			let bbox:BBoxDet<'a,N,T>=unsafe {std::mem::transmute(bbox)};
			func(bbox);
		});

		Ok(())
	}
}

pub fn multi_rect<'a,A:AxisTrait,N:NumTrait,T>(tree:&'a mut DynTree<A,(),BBox<N,T>>)->MultiRect<'a,A,N,T>{
	MultiRect{tree,rects:SmallVec::new()}
}
*/