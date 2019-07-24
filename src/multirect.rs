//!
//!
//! If we have two non intersecting rectangles, it is safe to return to the user two sets of mutable references
//! of the bots strictly inside each rectangle since it is impossible for a bot to belong to both sets.
//!
//! # Safety
//!
//! Unsafe code is used.  We unsafely convert the references returned by the rect query
//! closure to have a longer lifetime.
//! This allows the user to store mutable references of non intersecting rectangles at the same time. 
//! If two requested rectangles intersect, an error is returned.
//!
use crate::inner_prelude::*;
use crate::colfind::oned;

///Indicates that the user supplied a rectangle
///that intersects with a another one previously queries
///in the session.
#[derive(Debug)]
pub struct RectIntersectErr;

///Handles a multi rect mut "sessions" within which
///the user can query multiple non intersecting rectangles.
pub struct MultiRectMut<'a,K:DinoTreeRefMutTrait> {
    //tree: DinoTreeRefMut<'a,A,T>,
    tree:&'a mut K,
    rects: SmallVec<[Rect<K::Num>; 16]>,
}

impl<'a,K:DinoTreeRefMutTrait> MultiRectMut<'a,K>{
	pub fn for_all_in_rect_mut(&mut self,rect:Rect<K::Num>,mut func:impl FnMut(&'a mut K::Item))->Result<(),RectIntersectErr>{
		for r in self.rects.iter(){
			if rect.get_intersect_rect(r).is_some(){
				return Err(RectIntersectErr);
			}
		}

		self.rects.push(rect);

		rect::for_all_in_rect_mut(&mut self.tree,&rect,|bbox:&mut K::Item|{
			//This is only safe to do because the user is unable to mutate the bounding box.
			let bbox:&'a mut K::Item=unsafe {core::mem::transmute(bbox)};
			func(bbox);
		});

		Ok(())
	}
}


///Starts a multi rect mut sessions.
pub fn multi_rect_mut<'a,K:DinoTreeRefMutTrait>(tree:&'a mut K)->MultiRectMut<'a,K>{
	MultiRectMut{tree,rects:SmallVec::new()}
}


///Sorts the bots.
fn sweeper_update<I:HasAabb,A:AxisTrait>(axis:A,collision_botids: &mut [I]) {

    let sclosure = |a: &I, b: &I| -> core::cmp::Ordering {
        let (p1,p2)=(a.get().get_range(axis).left,b.get().get_range(axis).left);
        if p1 > p2 {
            return core::cmp::Ordering::Greater;
        }
        core::cmp::Ordering::Less
    };

    collision_botids.sort_unstable_by(sclosure);
}

//use oned::Blee;
///Find all bots that collide along the specified axis only between two rectangles.
///So the bots may not actually collide in 2d space, but collide alone the x or y axis.
///This is useful when implementing "wrap around" behavior of bots that pass over a rectangular border.
pub fn collide_two_rect_parallel<
    'a,
    K:DinoTreeRefMutTrait,
    F: FnMut(&mut K::Item, &mut K::Item),
>(
    multi: &mut MultiRectMut<'a,K>,
    axis:impl AxisTrait, //axis to sort under. not neccesarily the same as DinoTree axis
    rect1: &Rect<K::Num>,
    rect2: &Rect<K::Num>,
    mut func: F,
)->Result<(),RectIntersectErr> {

	struct Wr<'a,T:HasAabb+'a>(&'a mut T);
	unsafe impl<'a,T:HasAabb+'a> HasAabb for Wr<'a,T>{
		type Num=T::Num;
		fn get(&self)->&Rect<Self::Num>{
			self.0.get()
		}
	}

	//let mut multi=multi_rect_mut(tree);

	let mut b1=Vec::new();
	multi.for_all_in_rect_mut(*rect1,|a|{
		b1.push(Wr(a));
	})?;

	let mut b2=Vec::new();
	multi.for_all_in_rect_mut(*rect2,|b|{
		b2.push(Wr(b));
	})?;

	sweeper_update(axis,&mut b1);
    sweeper_update(axis,&mut b2);
    

    let mut sweeper = oned::Sweeper::new();

    struct Bl<T:HasAabb,F:FnMut(&mut T,&mut T)> {
        a: F,
        _p:PhantomData<T>
    }

    impl<T:HasAabb,F:FnMut(&mut T,&mut T)> colfind::ColMulti for Bl<T,F> {
        type T = T;

        fn collide(&mut self, a: &mut Self::T, b: &mut Self::T) {
            (self.a)(a,b);
        }

    }

    let ff=|a:&mut Wr<K::Item>,b:&mut Wr<K::Item>|{
        func(a.0,b.0)
    };
    sweeper.find_parallel_2d_no_check(axis,&mut b1, &mut b2, &mut Bl{a:ff,_p:PhantomData});
    Ok(())
}

