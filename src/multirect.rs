
use inner_prelude::*;


#[derive(Debug)]
pub struct RectIntersectErr;


pub struct MultiRectMut<'a,A: AxisTrait+'a,T:HasAabb+'a> {
    tree: &'a mut DynTree<A,(),T>,
    rects: SmallVec<[Rect<T::Num>; 16]>,
}

impl<'a,A: AxisTrait+'a,T:HasAabb+'a> MultiRectMut<'a,A,T>{
	pub fn for_all_in_rect_mut(&mut self,rect:Rect<T::Num>,mut func:impl FnMut(&'a mut T))->Result<(),RectIntersectErr>{
		for r in self.rects.iter(){
			if rect.get_intersect_rect(r).is_some(){
				return Err(RectIntersectErr);
			}
		}

		self.rects.push(rect);

		rect::for_all_in_rect_mut(self.tree,&rect,|bbox:&mut T|{
			//This is only safe to do because the user is unable to mutate the bounding box.
			let bbox:&'a mut T=unsafe {std::mem::transmute(bbox)};
			func(bbox);
		});

		Ok(())
	}
}

pub fn multi_rect_mut<'a,A:AxisTrait,T:HasAabb>(tree:&'a mut DynTree<A,(),T>)->MultiRectMut<'a,A,T>{
	MultiRectMut{tree,rects:SmallVec::new()}
}
