use inner_prelude::*;
use colfind::ColMulti;

struct Bl<'a,A: AxisTrait+'a, F: ColMulti+'a> {
    a: &'a mut F,
    axis:A,
}

impl<'a,A: AxisTrait+'a, F: ColMulti+'a> ColMulti for Bl<'a,A, F> {
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
        clos2: &mut F,
    ) {
        let mut b: Bl<A, _> = Bl {
            a: clos2,
            axis
        };
        self.find(axis,bots, &mut b);
    }


    pub(crate) fn find_parallel_2d<A: AxisTrait, F: ColMulti<T=I>>(
        &mut self,
        axis:A,
        bots1: &mut [F::T],
        bots2: &mut [F::T],
        clos2: &mut F,
    ) {
        let mut b: Bl<A, _> = Bl {
            a: clos2,
            axis
        };

        self.find_bijective_parallel(axis,(bots1, bots2), &mut b);
    }


    pub(crate) fn find_parallel_2d_no_check<A: AxisTrait, F: ColMulti<T=I>>(
        &mut self,
        axis:A,
        bots1: &mut [F::T],
        bots2: &mut [F::T],
        clos2: &mut F,
    ) {
        self.find_bijective_parallel(axis,(bots1, bots2), clos2);
    }

    pub(crate) fn find_perp_2d<F: ColMulti<T=I>>(&mut self,
        r1: &mut [F::T],
        r2: &mut [F::T],
        clos2: &mut F){

        for inda in r1.iter_mut() {
            for indb in r2.iter_mut() {
                if inda.get().intersects_rect(indb.get()){
                //if inda.get().get_intersect_rect(indb.get()).is_some() {
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
        func: &mut F,
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
        func: &mut F,
    ) {
        let mut xs=cols.0.iter_mut().peekable();
        let ys = cols.1.iter_mut();

        let active_x = self.helper.get_empty_vec_mut();

        for y in ys {
            //Add all the x's that are touching the y to the active x.
            for x in xs.peeking_take_while(|x|x.get().get_range(axis).left<=y.get().get_range(axis).right){
                active_x.push(x);
            }
            
            //Prune all the x's that are no longer touching the y.
            active_x.retain(|x| {
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

#[test]
fn test_parallel(){
    use std::collections::BTreeSet;



    #[derive(Copy,Clone,Debug)]
    struct Bot{
        id:usize
    }

    struct Test{
        set:BTreeSet<[usize;2]>
    };
    impl ColMulti for &mut Test{
        type T=BBox<isize,Bot>;
        fn collide(&mut self,a:&mut Self::T,b:&mut Self::T){
            let [a,b]=[a.inner.id,b.inner.id];

            let fin=if a<b{
                [a,b]
            }else{
                [b,a]
            };
            self.set.insert(fin);
        }
    }

    struct Counter{
        counter:usize
    }
    impl Counter{
        fn make(&mut self,x1:isize,x2:isize)->BBox<isize,Bot>{
            let b=unsafe{BBox::new(axgeom::Rect::new(x1,x2,0,10),Bot{id:self.counter})};
            self.counter+=1;
            b
        }
    }

    let mut b=Counter{counter:0};
    
    //let mut left=[b.make(0,10)];
    //let mut right=[b.make(-5,5),b.make(5,15),b.make(-5,15),b.make(2,8),b.make(-5,-6),b.make(12,13)];

    let mut left=[b.make(0,10),b.make(5,20),b.make(10,40)];
    let mut right=[b.make(1,2),b.make(-5,-4),b.make(2,3),b.make(-5,-4),b.make(3,4),b.make(-5,-4),b.make(4,5),b.make(-5,-4),b.make(5,6),b.make(-5,-4),b.make(6,7)];
    

    let mut left=[b.make(0,10),b.make(5,20)];
    let mut right=[b.make(16,20)];

    let mut sweeper=Sweeper::new();
    


    let mut test1=Test{set:BTreeSet::new()};
    sweeper.find_bijective_parallel(axgeom::XAXISS,(&mut left,&mut right),&mut test1);



    let mut test2=Test{set:BTreeSet::new()};
    sweeper.find_bijective_parallel(axgeom::XAXISS,(&mut right,&mut left),&mut test2);

    let num=test1.set.symmetric_difference(&test2.set).count();

    assert_eq!(num,0);


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
