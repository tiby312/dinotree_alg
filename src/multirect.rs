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


///Sorts the bots.
fn sweeper_update<I:HasAabb,A:AxisTrait>(axis:A,collision_botids: &mut [I]) {

    let sclosure = |a: &I, b: &I| -> std::cmp::Ordering {
        let (p1,p2)=(a.get().get_range(axis).left,b.get().get_range(axis).left);
        if p1 > p2 {
            return std::cmp::Ordering::Greater;
        }
        std::cmp::Ordering::Less
    };

    collision_botids.sort_unstable_by(sclosure);
}

//use oned::Blee;
///Find all bots that collide along the specified axis only between two rectangles.
///So the bots may not actually collide in 2d space, but collide alone the x or y axis.
///This is useful when implementing "wrap around" behavior of bots that pass over a rectangular border.
pub fn collide_two_rect_parallel<
    'a,A: AxisTrait,
    Num: NumTrait,
    T: HasAabb<Num = Num>,
    F: FnMut(&mut T, &mut T),
>(
    multi: &mut MultiRectMut<'a,A,T>,
    axis:impl AxisTrait, //axis to sort under. not neccesarily the same as dyntree axis
    rect1: &Rect<T::Num>,
    rect2: &Rect<T::Num>,
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
        fn div(self)->(Self,Self){
            unreachable!();
        }
        fn add(self,_:Self)->Self{
            unreachable!();
        }
    }

    let ff=|a:&mut Wr<T>,b:&mut Wr<T>|{
        func(a.0,b.0)
    };
    sweeper.find_parallel_2d_no_check(axis,&mut b1, &mut b2, Bl{a:ff,_p:PhantomData});
    Ok(())
}

