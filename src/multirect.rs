
use inner_prelude::*;


use dinotree_inner::*;



pub fn rects_mut<A:AxisTrait,T:HasAabb>
(
	tree   : &mut DynTree<A,(),T>,
	rects1 : Rect<T::Num>,
	rects2 : Rect<T::Num>,
	func1  : impl FnMut(&mut T),
	func2  : impl FnMut(&mut T),
)
{

}