
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
        let (p1,p2)=(a.get().as_axis().get(axis).left,b.get().as_axis().get(axis).left);
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
    tree: &'a mut DynTree<A,(),T>,
    axis:impl AxisTrait, //axis to sort under. not neccesarily the same as dyntree axis
    rect1: &Rect<T::Num>,
    rect2: &Rect<T::Num>,
    mut func: F,
)->Result<(),RectIntersectErr> {

	struct Wr<'a,T:HasAabb+'a>(&'a mut T);
	impl<'a,T:HasAabb+'a> HasAabb for Wr<'a,T>{
		type Num=T::Num;
		fn get(&self)->&Rect<Self::Num>{
			self.0.get()
		}
	}

	let mut multi=multi_rect_mut(tree);

	let mut b1=Vec::new();
	multi.for_all_in_rect_mut(*rect1,|a|{
		b1.push(Wr(a));
	});

	let mut b2=Vec::new();
	multi.for_all_in_rect_mut(*rect2,|b|{
		b2.push(Wr(b));
	});

	sweeper_update(axis,&mut b1);
    sweeper_update(axis,&mut b2);
    

    //let mut sweeper = oned::mod_mut::Sweeper::new();
    unimplemented!();
    //sweeper.find_parallel_2d(axis,(&mut b1, &mut b2), func);


	/*
    struct Ba<'z, J: SweepTrait + 'z>(ColSingle<'z, J>);
    impl<'z, J: SweepTrait + Send + 'z> RebalTrait for Ba<'z, J> {
        type Num = J::Num;
        fn get(&self) -> &axgeom::Rect<J::Num> {
            &((self.0).rect).0
        }
    }

    impl<'z, J: SweepTrait + 'z> SweepTrait for Ba<'z, J> {
        type Inner = J::Inner;
        type Num = J::Num;

        ///Destructure into the bounding box and mutable parts.
        fn get_mut<'a>(&'a mut self) -> (&'a AABBox<J::Num>, &'a mut Self::Inner) {
            let r = &mut self.0;
            (r.rect, r.inner)
        }

        ///Destructue into the bounding box and inner part.
        fn get<'a>(&'a self) -> (&'a AABBox<J::Num>, &'a Self::Inner) {
            let r = &self.0;
            (r.rect, r.inner)
        }
    }

    let mut rects = tree.rects();

    let mut buffer1 = Vec::new();
    rects.for_all_in_rect(rect1, |a: ColSingle<T>| buffer1.push(Ba(a)))?;

    let mut buffer2 = Vec::new();
    rects.for_all_in_rect(rect2, |a: ColSingle<T>| buffer2.push(Ba(a)))?;

    {
        rayon::join(
            || sweeper_update::<_, A>(&mut buffer1),
            || sweeper_update::<_, A>(&mut buffer2),
        );
        use std::marker::PhantomData;
        use oned::Bleek;
        struct Bo<T: SweepTrait, F: FnMut(ColSingle<T>, ColSingle<T>)>(F, PhantomData<T>);

        impl<T: SweepTrait, F: FnMut(ColSingle<T>, ColSingle<T>)> Bleek for Bo<T, F> {
            type T = T;
            fn collide(&mut self, a: ColSingle<Self::T>, b: ColSingle<Self::T>) {
                self.0(a, b);
            }
        }

        let func2 = |aa: ColSingle<Ba<T>>, bb: ColSingle<Ba<T>>| {
            //let c=ColPair{a:(cc.a.0,cc.a.1),b:(cc.b.0,cc.b.1)};
            let a = ColSingle {
                rect: aa.rect,
                inner: aa.inner,
            };
            let b = ColSingle {
                rect: bb.rect,
                inner: bb.inner,
            };
            func(a, b);
        };

        let mut sweeper = Sweeper::new();

        let b = Bo(func2, PhantomData);
        sweeper.find_bijective_parallel::<A, _>((&mut buffer1, &mut buffer2), b);
    }
    Ok(())
    */
}

