use super::*;

///Provides a way to query multiple non-intersecting rectangles.
///References returned from the functions within this struct
///Can be held onto for the lifetime of this struct.
pub struct Rects<'a: 'b, 'b, T: SweepTrait + 'a>(RectsEnum<'a, 'b, T>);

impl<'a: 'b, 'b, T: SweepTrait + 'a> Rects<'a, 'b, T> {
    pub(crate) fn new(tree: &'b mut DinoTree<'a, T>) -> Rects<'a, 'b, T> {
        Rects(match &mut tree.0 {
            &mut DynTreeEnum::Xa(ref mut a) => RectsEnum::Xa(RectsInner {
                tree: a,
                rects: SmallVec::new(),
            }),
            &mut DynTreeEnum::Ya(ref mut a) => RectsEnum::Ya(RectsInner {
                tree: a,
                rects: SmallVec::new(),
            }),
        })
    }

    ///Returns an error if user supplies a rectangle that intersects with another one used to call this same
    ///function.
    pub fn for_all_in_rect<F: FnMut(ColSingle<'b, T>)>(&mut self, rect: &AABBox<T::Num>, func: F)->Result<(),RectIntersectErr>  {
        match &mut self.0 {
            &mut RectsEnum::Xa(ref mut a) => {
                a.for_all_in_rect(rect, func)
            }
            &mut RectsEnum::Ya(ref mut a) => {
                a.for_all_in_rect(rect, func)
            }
        }
    }
}

pub struct RectIntersectErr;

enum RectsEnum<'a: 'b, 'b, T: SweepTrait + 'a> {
    Xa(RectsInner<'a, 'b, XAXISS, T>),
    Ya(RectsInner<'a, 'b, YAXISS, T>),
}

struct RectsInner<'a: 'b, 'b, A: AxisTrait + 'a, T: SweepTrait + 'a> {
    tree: &'b mut DynTree<'a, A, T>,
    rects: SmallVec<[AABBox<T::Num>; 16]>,
}
use axgeom::AxisTrait;

impl<'a: 'b, 'b, A: AxisTrait + 'a, T: SweepTrait + 'a> RectsInner<'a, 'b, A, T> {
    ///Iterate over all bots in a rectangle.
    ///It is safe to call this function multiple times with rectangles that
    ///do not intersect. Because the rectangles do not intersect, all bots retrieved
    ///from inside either rectangle are guarenteed to be disjoint.
    ///If a rectangle is passed that does intersect one from a previous call,
    ///this function will panic.
    ///
    ///Note the lifetime of the mutable reference in the passed function.
    ///The user is allowed to move this reference out and hold on to it for
    ///the lifetime of this struct.
    pub fn for_all_in_rect<F: FnMut(ColSingle<'b, T>)>(
        &mut self,
        rect: &AABBox<T::Num>,
        mut func: F,
    )->Result<(),RectIntersectErr> {
        for k in self.rects.iter() {
            if rect.0.intersects_rect(&k.0) {
                return Err(RectIntersectErr);
            }
        }

        self.rects.push(rect.clone());

        {
            let wrapper = |c: ColSingle<T>| {
                let (a, b) = (c.rect as *const AABBox<T::Num>, c.inner as *mut T::Inner);
                //Unsafely extend the lifetime to accocomate the
                //lifetime of RectsTrait.
                let (a, b) = unsafe { (&*a, &mut *b) };

                let cn = ColSingle { rect: a, inner: b };
                func(cn);
            };

            rect::for_all_in_rect(self.tree, &rect.0, wrapper);
        }
        return Ok(())
    }
}
