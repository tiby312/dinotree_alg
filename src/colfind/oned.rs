use crate::inner_prelude::*;
use crate::colfind::ColMulti;

struct Bl<'a,A: AxisTrait+'a, F: ColMulti+'a> {
    a: &'a mut F,
    axis:A,
}

impl<'a,A: AxisTrait+'a, F: ColMulti+'a> ColMulti for Bl<'a,A, F> {
    type T = F::T;

    #[inline(always)]
    fn collide(&mut self,
        a:BBoxRefMut<<F::T as HasAabb>::Num,<F::T as HasAabb>::Inner>,
        b:BBoxRefMut<<F::T as HasAabb>::Num,<F::T as HasAabb>::Inner>)
    {
        //only check if the opoosite axis intersects.
        //already know they intersect
        let a2 = self.axis.next();
        if a.rect.get_range(a2).intersects(b.rect.get_range(a2))
        {
            self.a.collide(a, b);
        }
    }
}

/*
#[repr(transparent)]
pub struct WrapT<'a,T:HasAabb+'a>{
    pub inner:&'a mut T
}

unsafe impl<'a,T:HasAabb> HasAabb for WrapT<'a,T>{
    type Num=T::Num;
    fn get(&self)->&axgeom::Rect<T::Num>{
        self.inner.get()
    }
}
*/


///Provides 1d collision detection.
pub struct Sweeper<T: HasAabbMut> {
    helper: tools::PreVecMut<T>,
    //helper2: tools::PreVecMut<T>,
}

impl<T:HasAabbMut> core::default::Default for Sweeper<T>{
    #[inline(always)]
    fn default()->Sweeper<T>{
        Sweeper::new()
    }
}
impl<I: HasAabbMut> Sweeper<I> {
    #[inline(always)]
    pub fn new() -> Sweeper<I> {
        Sweeper {
            helper: tools::PreVecMut::new(),
            //helper2: tools::PreVecMut::new()
        }
    }


    //Bots a sorted along the axis.
    #[inline(always)]
    pub(crate) fn find_2d<A: AxisTrait, F: ColMulti<T=I>>(
        &mut self,
        axis:A,
        bots: ElemSliceMut<F::T>,
        clos2: &mut F,
    ) {
        let mut b: Bl<A, _> = Bl {
            a: clos2,
            axis
        };
        self.find(axis,bots, &mut b);
    }


    /*
    pub(crate) fn find_parallel_2d_ptr<A: AxisTrait, F: ColMulti<T=I>>(
        &mut self,
        axis:A,
        bots1: &mut [F::T],
        bots2: &mut [WrapT<F::T>],
        clos2: &mut F,
    ) {
        let mut b: Bl<A, _> = Bl {
            a: clos2,
            axis
        };

        self.find_bijective_parallel_ptr(axis,(bots1, bots2), &mut b);
    }
    */


    #[inline(always)]
    pub(crate) fn find_parallel_2d<A: AxisTrait, F: ColMulti<T=I>>(
        &mut self,
        axis:A,
        bots1: ElemSliceMut<F::T>,
        bots2: ElemSliceMut<F::T>,
        clos2: &mut F,
    ) {
        let mut b: Bl<A, _> = Bl {
            a: clos2,
            axis
        };

        self.find_bijective_parallel(axis,(bots1, bots2), &mut b);
    }
    

    #[inline(always)]
    pub(crate) fn find_parallel_2d_no_check<A: AxisTrait, F: ColMulti<T=I>>(
        &mut self,
        axis:A,
        bots1: ElemSliceMut<F::T>,
        bots2: ElemSliceMut<F::T>,
        clos2: &mut F,
    ) {
        self.find_bijective_parallel(axis,(bots1, bots2), clos2);
    }


    pub(crate) fn find_perp_2d1<A:AxisTrait,F: ColMulti<T=I>>(&mut self,
        _axis:A,
        mut r1: ElemSliceMut<F::T>,
        mut r2: ElemSliceMut<F::T>,
        clos2: &mut F){
        /*
        let mut bots2:&mut Vec<WrapT<I>>=unsafe{&mut *(self.helper2.get_empty_vec_mut() as *mut Vec<&mut I> as *mut Vec<WrapT<I>>)};
        for b in r2.iter_mut().map(|a|WrapT{inner:a}){
            bots2.push(b);
        }
        dinotree::advanced::sweeper_update(axis,&mut bots2);
        self.find_parallel_2d_ptr(axis,r1,&mut bots2,clos2);
        */
        
        for mut inda in r1.iter_mut() {
            for mut indb in r2.iter_mut() {
                if inda.rect.intersects_rect(indb.rect){
                    clos2.collide(inda.as_mut(), indb.as_mut());
                }
            }
        } 
        
           
    }


    ///Find colliding pairs using the mark and sweep algorithm.
    fn find<'a, A: AxisTrait, F: ColMulti<T = I>>(
        &mut self,
        axis:A,
        mut collision_botids: ElemSliceMut<'a,I>,
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

        for mut curr_bot in collision_botids.iter_mut() {
            {
                {
                    let crr = curr_bot.rect.get_range(axis);
                    //change this to do retain and then iter
                    active.retain(|that_bot|that_bot.rect.get_range(axis).right >= crr.left);
                }

                for that_bot in active.iter_mut() {
                    
                    //TODO this fails! Okay?
                    //debug_assert!(curr_bot.get().get_range(axis).intersects(that_bot.get().get_range(axis)));
                
                    func.collide(curr_bot.as_mut(), that_bot.as_mut());
                }
            }
            active.push(curr_bot);
        }
    }





    fn find_bijective_parallel<A: AxisTrait, F: ColMulti<T = I>>(
        &mut self,
        axis:A,
        mut cols: (ElemSliceMut<I>, ElemSliceMut<I>),
        func: &mut F,
    ) {
        let mut xs=cols.0.iter_mut().peekable();
        let ys = cols.1.iter_mut();

        let active_x = self.helper.get_empty_vec_mut();

        for mut y in ys {
            //Add all the x's that are touching the y to the active x.
            for x in xs.peeking_take_while(|x|x.rect.get_range(axis).left<=y.rect.get_range(axis).right){
                active_x.push(x);
            }
            
            //Prune all the x's that are no longer touching the y.
            active_x.retain(|x| x.rect.get_range(axis).right >= y.rect.get_range(axis).left);

            //So at this point some of the x's could actualy not intersect y.
            //These are the x's that are to the complete right of y.
            //So to handle collisions, we want to make sure to not hit these.
            //That is why we have that condition to break out of the below loop
            for x in active_x.iter_mut() {
                if x.rect.get_range(axis).left>y.rect.get_range(axis).right{
                    break;
                }

                debug_assert!(x.rect.get_range(axis).intersects(y.rect.get_range(axis)));
                func.collide(x.as_mut(), y.as_mut());
            }
        }
    }

    /*
    fn find_bijective_parallel_ptr<A: AxisTrait, F: ColMulti<T = I>>(
        &mut self,
        axis:A,
        cols: (&mut [I], &mut [WrapT<I>]),
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
            active_x.retain(|x|x.get().get_range(axis).right >= y.get().get_range(axis).left);

            //So at this point some of the x's could actualy not intersect y.
            //These are the x's that are to the complete right of y.
            //So to handle collisions, we want to make sure to not hit these.
            //That is why we have that condition to break out of the below loop
            for x in active_x.iter_mut() {
                if x.get().get_range(axis).left>y.get().get_range(axis).right{
                    break;
                }

                debug_assert!(x.get().get_range(axis).intersects(y.get().get_range(axis)));
                func.collide(x, y.inner);
            }
        }
    }
    */

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
    impl ColMulti for Test{
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
    

    //let mut left=[b.make(0,10),b.make(5,20)];
    //let mut right=[b.make(16,20)];

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
pub fn get_section<'a, I:HasAabb,A: AxisTrait>(axis:A,arr: &'a ElemSlice<I>, range: &Range<I::Num>) -> &'a ElemSlice<I> {
    let mut start = 0;
    for (e, i) in arr.iter().enumerate() {
        let rr = i.rect.get_range(axis);
        if rr.right >= range.left {
            start = e;
            break;
        }
    }

    let mut end = arr.len();
    for (e, i) in arr.truncate_start(start).iter().enumerate() {
        let rr = i.rect.get_range(axis);
        if rr.left > range.right {
            end = start + e;
            break;
        }
    }

    arr.truncate(start,end)
}

//this can have some false positives.
//but it will still prune a lot of bots.
pub fn get_section_mut<'a,I:HasAabbMut, A: AxisTrait>(axis:A,mut arr: ElemSliceMut<'a,I>, range: &Range<I::Num>) -> ElemSliceMut<'a,I> {
    let mut start = 0;
    for (e, i) in arr.iter().enumerate() {
        let rr = i.rect.get_range(axis);
        if rr.right >= range.left {
            start = e;
            break;
        }
    }

    let mut end = arr.len();
    //for (e, i) in arr[start..].iter().enumerate() {
    for (e, i) in arr.as_mut().truncate_start_mut(start).iter().enumerate() {
    
        let rr = i.rect.get_range(axis);
        if rr.left > range.right {
            end = start + e;
            break;
        }
    }

    arr.truncate_mut(start,end)
    //&mut arr[start..end]
}
