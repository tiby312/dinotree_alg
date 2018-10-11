use inner_prelude::*;
use colfind::ColMulti;

struct Bl<A: AxisTrait, F: ColMulti> {
    a: F,
    axis:A,
}

impl<A: AxisTrait, F: ColMulti> ColMulti for Bl<A, F> {
    type T = F::T;

    fn collide(&mut self, a: &mut Self::T, b: &mut Self::T) {
        //only check if the opoosite axis intersects.
        //already know they intersect
        let a2 = self.axis.next();
        if a.get().get_range(a2).intersects(b.get().get_range(a2))
        {
            self.a.collide(a, b);
        }
    }
}




///Provides 1d collision detection.
pub struct Sweeper<T: HasAabb> {
    helper: tools::PreVecMut<T>,
}

impl<I: HasAabb> Sweeper<I> {
    pub fn new() -> Sweeper<I> {
        Sweeper {
            helper: tools::PreVecMut::new(),
        }
    }


    //Bots a sorted along the axis.
    pub(crate) fn find_2d<A: AxisTrait, F: ColMulti<T=I>>(
        &mut self,
        axis:A,
        bots: &mut [F::T],
        clos2: F,
    ) {
        let b: Bl<A, _> = Bl {
            a: clos2,
            axis
        };
        self.find(axis,bots, b);
    }


    pub(crate) fn find_parallel_2d<A: AxisTrait, F: ColMulti<T=I>>(
        &mut self,
        axis:A,
        bots1: &mut [F::T],
        bots2: &mut [F::T],
        clos2: F,
    ) {
        let b: Bl<A, _> = Bl {
            a: clos2,
            axis
        };

        self.find_bijective_parallel(axis,(bots1, bots2), b);
    }


    pub(crate) fn find_parallel_2d_no_check<A: AxisTrait, F: ColMulti<T=I>>(
        &mut self,
        axis:A,
        bots1: &mut [F::T],
        bots2: &mut [F::T],
        clos2: F,
    ) {
        self.find_bijective_parallel(axis,(bots1, bots2), clos2);
    }

    pub(crate) fn find_perp_2d<F: ColMulti<T=I>>(&mut self,
        r1: &mut [F::T],
        r2: &mut [F::T],
        mut clos2: F){

       
        for inda in r1.iter_mut() {
            for indb in r2.iter_mut() {
                if inda.get().get_intersect_rect(indb.get()).is_some() {
                    clos2.collide(inda, indb);
                }
            }
        }
    }

    ///Find colliding pairs using the mark and sweep algorithm.
    fn find<'a, A: AxisTrait, F: ColMulti<T = I>>(
        &mut self,
        axis:A,
        collision_botids: &'a mut [I],
        mut func: F,
    ) {
        //    Create a new temporary list called “activeList”.
        //    You begin on the left of your axisList, adding the first item to the activeList.
        //
        //    Now you have a look at the next item in the axisList and compare it with all items
        //     currently in the activeList (at the moment just one):
        //     - If the new item’s left is greater then the current activeList-item right,
        //       then remove
        //    the activeList-item from the activeList
        //     - otherwise report a possible collision between the new axisList-item and the current
        //     activeList-item.
        //
        //    Add the new item itself to the activeList and continue with the next item
        //     in the axisList.

        let active = self.helper.get_empty_vec_mut();

        for curr_bot in collision_botids.iter_mut() {
            {
                {
                    let crr = curr_bot.get().get_range(axis);
                    //change this to do retain and then iter
                    active.retain(|that_bot| {
                        let brr = that_bot.get().get_range(axis);

                        if brr.right < crr.left {
                            false
                        } else {
                            true
                        }
                    });
                }

                for that_bot in active.iter_mut() {
                    
                    debug_assert!(curr_bot.get().get_range(axis).intersects(that_bot.get().get_range(axis)));
                
                    func.collide(curr_bot, that_bot);
                }
            }
            active.push(curr_bot);
        }
    }


    fn find_bijective_parallel<A: AxisTrait, F: ColMulti<T = I>>(
        &mut self,
        axis:A,
        cols: (&mut [I], &mut [I]),
        mut func: F,
    ) {
    
        //let mut xs = cols.0.iter_mut().peekable();
        let mut xs= tools::UndoIterator::new(cols.0.iter_mut());
        let ys = cols.1.iter_mut();

        let active_x = self.helper.get_empty_vec_mut();

        for y in ys {

            //Add all the x's that are touching the y to the active x.
            loop{
                match xs.next(){
                    Some(x)=>{
                        if x.get().get_range(axis).left > y.get().get_range(axis).right{
                            xs.add_back(x);
                            break;
                        }else{
                            active_x.push(x);
                        }
                    },
                    None=>{
                        break;
                    }
                }
            }

            //Prune all the x's that are no longer touching the y.
            active_x.retain(|x: &mut &mut I| {
                if x.get().get_range(axis).right
                    < y.get().get_range(axis).left
                {
                    false
                } else {
                    true
                }
            });

            //So at this point some of the x's could actualy not intersect y.
            //These are the x's that are to the complete right of y.
            //So to handle collisions, we want to make sure to not hit these.
            //That is why we have that condition to break out of the below loop

            for x in active_x.iter_mut() {
                if x.get().get_range(axis).left>y.get().get_range(axis).right{
                    break;
                }

                debug_assert!(x.get().get_range(axis).intersects(y.get().get_range(axis)));
                func.collide(x, y);
            }
        }
    }

}


//this can have some false positives.
//but it will still prune a lot of bots.
pub fn get_section<'a, I:HasAabb,A: AxisTrait>(axis:A,arr: &'a [I], range: &Range<I::Num>) -> &'a [I] {
    let mut start = 0;
    for (e, i) in arr.iter().enumerate() {
        let rr = i.get().get_range(axis);
        if rr.right >= range.left {
            start = e;
            break;
        }
    }

    let mut end = arr.len();
    for (e, i) in arr[start..].iter().enumerate() {
        let rr = i.get().get_range(axis);
        if rr.left > range.right {
            end = start + e;
            break;
        }
    }

    return &arr[start..end];
}

//this can have some false positives.
//but it will still prune a lot of bots.
pub fn get_section_mut<'a,I:HasAabb, A: AxisTrait>(axis:A,arr: &'a mut [I], range: &Range<I::Num>) -> &'a mut [I] {
    let mut start = 0;
    for (e, i) in arr.iter().enumerate() {
        let rr = i.get().get_range(axis);
        if rr.right >= range.left {
            start = e;
            break;
        }
    }

    let mut end = arr.len();
    for (e, i) in arr[start..].iter().enumerate() {
        let rr = i.get().get_range(axis);
        if rr.left > range.right {
            end = start + e;
            break;
        }
    }

    
    return &mut arr[start..end];
}
