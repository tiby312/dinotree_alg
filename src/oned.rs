use inner_prelude::*;
use unsafe_unwrap::UnsafeUnwrap;

//TODO bench these without bounds checking and unwrap()ing.


pub mod mod_mut{
    use colfind::mutable::ColMulti;
    use super::*;

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
            if a.get().as_axis().get(a2).intersects(b.get().as_axis().get(a2))
            {
                self.a.collide(a, b);
            }
        }
        fn div(self)->(Self,Self){
            unreachable!();
        }
        fn add(self,_:Self)->Self{
            unreachable!();
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
        pub fn find_2d<A: AxisTrait, F: ColMulti<T=I>>(
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


        pub fn find_parallel_2d<A: AxisTrait, F: ColMulti<T=I>>(
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


        pub fn find_parallel_2d_no_check<A: AxisTrait, F: ColMulti<T=I>>(
            &mut self,
            axis:A,
            bots1: &mut [F::T],
            bots2: &mut [F::T],
            clos2: F,
        ) {
            self.find_bijective_parallel(axis,(bots1, bots2), clos2);
        }
  
        pub fn find_perp_2d<F: ColMulti<T=I>>(&mut self,
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
                        let crr = curr_bot.get().as_axis().get(axis);
                        //change this to do retain and then iter
                        active.retain(|that_bot| {
                            let brr = that_bot.get().as_axis().get(axis);

                            if brr.right < crr.left {
                                false
                            } else {
                                true
                            }
                        });
                    }

                    for that_bot in active.iter_mut() {
                        
                        debug_assert!(curr_bot.get().as_axis().get(axis).intersects(that_bot.get().as_axis().get(axis)));
                    
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
            let mut xs = cols.0.iter_mut().peekable();
            let ys = cols.1.iter_mut();

            let active_x = self.helper.get_empty_vec_mut();

            for y in ys {

                //Add all the x's that are touching the y to the active x.
                while xs.peek().is_some() {
                    unsafe{
                        let v = {
                            let x = xs.peek().unsafe_unwrap();
                            x.get().as_axis().get(axis).left
                                > y.get().as_axis().get(axis).right
                        };
                        if v {
                            break;
                        } else {
                            active_x.push(xs.next().unsafe_unwrap());
                        }
                    }
                }

                

                //Prune all the x's that are no longer touching the y.
                active_x.retain(|x: &mut &mut I| {
                    if x.get().as_axis().get(axis).right
                        < y.get().as_axis().get(axis).left
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
                    if x.get().as_axis().get(axis).left>y.get().as_axis().get(axis).right{
                        break;
                    }

                    debug_assert!(x.get().as_axis().get(axis).intersects(y.get().as_axis().get(axis)));
                    func.collide(x, y);
                }
            }
        }

        //this can have some false positives.
        //but it will still prune a lot of bots.
        fn get_section_general<'a, A: AxisTrait>(axis:A,arr: &'a mut [I], range: &Range<I::Num>) -> (&'a mut [I],usize,usize) {
            let mut start = 0;
            for (e, i) in arr.iter().enumerate() {
                let rr = i.get().as_axis().get(axis);
                if rr.right >= range.left {
                    start = e;
                    break;
                }
            }

            let mut end = arr.len();
            for (e, i) in arr[start..].iter().enumerate() {
                let rr = i.get().as_axis().get(axis);
                if rr.left > range.right {
                    end = start + e;
                    break;
                }
            }

            
            return (&mut arr[start..end],start,end);
        }
        //this can have some false positives.
        //but it will still prune a lot of bots.
        pub fn get_section<'a, A: AxisTrait>(&self,axis:A,arr: &'a mut [I], range: &Range<I::Num>) -> &'a mut [I] {
            Self::get_section_general(axis,arr,range).0
        }
    }
}

pub mod mod_const{   

    use super::*;
    use colfind::constant::ColMulti;

    struct Bl<A: AxisTrait, F: ColMulti> {
        a: F,
        axis: A,
    }

    impl<A: AxisTrait, F: ColMulti> ColMulti for Bl<A, F> {
        type T = F::T;

        fn collide(&mut self, a: &Self::T, b: &Self::T) {
            //only check if the opoosite axis intersects.
            //already know they intersect
            let a2 = self.axis.next();
            if a.get().as_axis().get(a2).intersects(b.get().as_axis().get(a2))
            {
                self.a.collide(a, b);
            }
        }
         fn div(self)->(Self,Self){
            unreachable!();
        }
        fn add(self,_:Self)->Self{
            unreachable!();
        }
    }

    ///Provides 1d collision detection.
    pub struct Sweeper<T: HasAabb> {
        helper: tools::PreVec<T>,
    }

    impl<I: HasAabb> Sweeper<I> {
        pub fn new() -> Sweeper<I> {
            Sweeper {
                helper: tools::PreVec::new(),
            }
        }


        //Bots a sorted along the axis.
        pub fn find_2d<A: AxisTrait, F: ColMulti<T=I>>(
            &mut self,
            axis:A,
            bots: &[F::T],
            clos2: F,
        ) {
            let b: Bl<A, _> = Bl {
                a: clos2,
                axis,
            };
            self.find(axis,bots, b);
        }


        pub fn find_parallel_2d<A: AxisTrait, F: ColMulti<T=I>>(
            &mut self,
            axis:A,
            bots1: &[F::T],
            bots2: &[F::T],
            clos2: F,
        ) {
            let b: Bl<A, _> = Bl {
                a: clos2,
                axis
            };

            self.find_bijective_parallel(axis,(bots1, bots2), b);
        }

        pub fn find_perp_2d<F: ColMulti<T=I>>(&mut self,
            r1: &[F::T],
            r2: &[F::T],
            mut clos2: F){

           
            for inda in r1.iter() {
                for indb in r2.iter() {
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
            collision_botids: &'a [I],
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

            let active = self.helper.get_empty_vec();

            for curr_bot in collision_botids.iter() {
                {
                    {
                        let crr = curr_bot.get().as_axis().get(axis);
                        //change this to do retain and then iter
                        active.retain(|that_bot| {
                            let brr = that_bot.get().as_axis().get(axis);

                            if brr.right < crr.left {
                                false
                            } else {
                                true
                            }
                        });
                    }

                    for that_bot in active.iter() {
                        
                        debug_assert!(curr_bot.get().as_axis().get(axis).intersects(that_bot.get().as_axis().get(axis)));
                    
                        func.collide(curr_bot, that_bot);
                    }
                }
                active.push(curr_bot);
            }
        }

        fn find_bijective_parallel<A: AxisTrait, F: ColMulti<T = I>>(
            &mut self,
            axis:A,
            cols: (&[I], &[I]),
            mut func: F,
        ) {
            let mut xs = cols.0.iter().peekable();
            let ys = cols.1.iter();

            let active_x = self.helper.get_empty_vec();

            for y in ys {

                //Add all the x's that are touching the y to the active x.
                while xs.peek().is_some() {
                    unsafe{
                        let v = {
                            let x = xs.peek().unsafe_unwrap();
                            x.get().as_axis().get(axis).left
                                > y.get().as_axis().get(axis).right
                        };
                        if v {
                            break;
                        } else {
                            active_x.push(xs.next().unsafe_unwrap());
                        }
                    }
                }

                

                //Prune all the x's that are no longer touching the y.
                active_x.retain(|x: &mut &I| {
                    if x.get().as_axis().get(axis).right
                        < y.get().as_axis().get(axis).left
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

                for x in active_x.iter() {
                    if x.get().as_axis().get(axis).left>y.get().as_axis().get(axis).right{
                        break;
                    }

                    debug_assert!(x.get().as_axis().get(axis).intersects(y.get().as_axis().get(axis)));
                    func.collide(x, y);
                }
            }
        }

        //this can have some false positives.
        //but it will still prune a lot of bots.
        fn get_section_general<'a, A: AxisTrait>(axis:A,arr: &'a [I], range: &Range<I::Num>) -> (&'a [I],usize,usize) {
            let mut start = 0;
            for (e, i) in arr.iter().enumerate() {
                let rr = i.get().as_axis().get(axis);
                if rr.right >= range.left {
                    start = e;
                    break;
                }
            }

            let mut end = arr.len();
            for (e, i) in arr[start..].iter().enumerate() {
                let rr = i.get().as_axis().get(axis);
                if rr.left > range.right {
                    end = start + e;
                    break;
                }
            }

            
            return (&arr[start..end],start,end);
        }
        //this can have some false positives.
        //but it will still prune a lot of bots.
        pub fn get_section<'a, A: AxisTrait>(&self,axis:A,arr: &'a [I], range: &Range<I::Num>) -> &'a [I] {
            Self::get_section_general(axis,arr,range).0
        }
    }
}
/*
    #[cfg(test)]
    mod test{
        use test_support;
        use test_support::Bot;
        use test_support::create_unordered;
        use super::*;
        use axgeom;
        //use Blee;
        use support::BBox;
        use *;
        use ordered_float::NotNaN;
        #[test]
        fn test_get_section(){
            for _ in 0..100{
                let world=test_support::create_word();
                let axis=axgeom::XAXIS;
                let rr=Range{start:100.0,end:110.0};


                  let mut vec1:Vec<BBox<NotNaN<f32>,Bot>>=(0..1000).map(|a|
                {
                    let rect=test_support::get_random_rect(&world);
                    let bot=Bot::new(a);
                    BBox::new(bot,rect)
                }
                    ).collect();
                
                //let mut vec1:Vec<Bot>=(0..500).map(|a|Bot{id:a,rect:support::get_random_rect(&world)}).collect();



                let src:Vec<usize>={
                    let mut src_temp=Vec::new();

                    for a in vec1.iter(){

                        if rr.intersects(a.rect.get_range(axis)){
                            src_temp.push(a.val.id);
                        }
                    
                    }
                    src_temp
                };


                let mut sw=Sweeper::new();
                let a=Blee::new(axis);            
                Sweeper::update(&mut vec1,&a);
            
                /*
                println!("Bots:");
                for b in vec1.iter(){
                    println!("{:?}",(b.id,b.rect.get_range(axis)));
                }
                */


                let target=sw.get_section(&mut vec1,&rr,&a);

                match target{
                    Some(x)=>{

                        //Assert that all bots that intersect the rect are somewhere in the list outputted by get_setion().
                        for k in src.iter(){
                            let mut found=false;
                            for j in x.iter(){
                                if *k==j.val.id{
                                    found=true;
                                    break;
                                }
                            }
                            assert!(found);
                        }

                        //Assert that the first bot in the outputted list intersects with get_section().
                        let first=x.first().unwrap();
                        let mut found=false;
                        for j in src.iter(){
                            if first.val.id==*j{
                                found=true;
                                break;
                            }
                        }
                        assert!(found);

                        //Assert that the last bot in the outputted list intersects with get_section(). 
                        let last=&x[x.len()-1];
                        let mut found=false;
                        for j in src.iter(){
                            if last.val.id==*j{
                                found=true;
                                break;
                            }
                        }
                        assert!(found);
                    },
                    None=>{
                        assert!(src.len()==0);
                    }
                }

            } 
        }
        

        #[test]
        fn test_bijective_parallel(){       
            for _ in 0..100{
               let world=test_support::create_word();
                //let mut vec1:Vec<BBox<Bot>>=(0..5).map(|a|Bot{id:a,rect:support::get_random_rect(&world)}).collect();
                //let mut vec2:Vec<BBox<Bot>>=(0..5).map(|a|Bot{id:vec1.len()+a,rect:support::get_random_rect(&world)}).collect();
                 

                  let mut vec1:Vec<BBox<NotNaN<f32>,Bot>>=(0..5).map(|a|
                {
                    let rect=test_support::get_random_rect(&world);
                    let bot=Bot::new(a);
                    BBox::new(bot,rect)
                }
                    ).collect();


                  let mut vec2:Vec<BBox<NotNaN<f32>,Bot>>=(0..5).map(|a|
                {
                    let rect=test_support::get_random_rect(&world);
                    let bot=Bot::new(vec1.len()+a);
                    BBox::new(bot,rect)
                }
                    ).collect();


                let axis=axgeom::XAXIS;
                            
                let mut src:Vec<(usize,usize)>={
                    let mut src_temp=Vec::new();

                    for i in vec1.iter(){
                        for j in vec2.iter(){
                            let (a,b):(&BBox<NotNaN<f32>,Bot>,&BBox<NotNaN<f32>,NotNaN<f32>,Bot>)=(i,j);

                            if a.rect.get_range(axis).intersects(b.rect.get_range(axis)){
                                src_temp.push(create_unordered(&a.val,&b.val));
                            }
                        }
                    }
                    src_temp
                };

                let mut sw=Sweeper::new();
                let a=Blee::new(axis);
                Sweeper::update(&mut vec1,&a);
                Sweeper::update(&mut vec2,&a);


                let mut val=Vec::new();
                //let rr=world.get_range(axis);

                {
                    let mut f=|cc:ColPair<BBox<NotNaN<f32>,Bot>>|{
                        val.push(create_unordered(cc.a.1,cc.b.1));
                    };
                    let mut bk=BleekSF::new(&mut f);
                    sw.find_bijective_parallel((&mut vec1,&mut vec2),&a,&mut bk);
                }
                src.sort_by(&test_support::compair_bot_pair);
                val.sort_by(&test_support::compair_bot_pair);

                /*
                println!("naive result:\n{:?}",(src.len(),&src));
                println!("sweep result:\n{:?}",(val.len(),&val));

                println!("Bots:");
                for b in vec1{
                    println!("{:?}",(b.id,b.rect.get_range(axis)));
                }
                println!();
                
                for b in vec2{
                    println!("{:?}",(b.id,b.rect.get_range(axis)));
                }
                */
                assert!(src==val);
            }
        }

        #[test]
        fn test_find(){

            //let world=axgeom::Rect::new(-1000.0,1000.0,-1000.0,1000.0);
            let world=test_support::create_word();

                  let mut vec:Vec<BBox<NotNaN<f32>,Bot>>=(0..500).map(|a|
                {
                    let rect=test_support::get_random_rect(&world);
                    let bot=Bot::new(a);
                    BBox::new(bot,rect)
                }
                    ).collect();
            

            //Lets always order the ids smaller to larger to make it easier to look up.
            // let mut map:HashMap<(usize,usize),()>=HashMap::new();
            let mut src:Vec<(usize,usize)>=Vec::new();

            let axis=axgeom::XAXIS;
            for (e,i) in vec.iter().enumerate(){
                for j in vec[e+1..].iter(){
                    let (a,b):(&BBox<NotNaN<f32>,Bot>,&BBox<NotNaN<f32>,Bot>)=(i,j);

                    if a.rect.get_range(axis).intersects(b.rect.get_range(axis)){
                        src.push(create_unordered(&a.val,&b.val));
                    }
                }
            }

            let mut sw=Sweeper::new();
            
            let a=Blee::new(axis);
            Sweeper::update(&mut vec,&a);

            let mut val=Vec::new();

            {
                let mut f=|cc:ColPair<BBox<NotNaN<f32>,Bot>>|{
                    val.push(create_unordered(cc.a.1,cc.b.1));
                };
                let mut bk=BleekSF::new(&mut f);
                sw.find(&mut vec,&a,&mut bk);
            }
            src.sort_by(&test_support::compair_bot_pair);
            val.sort_by(&test_support::compair_bot_pair);

            //println!("{:?}",(src.len(),val.len()));
            //println!("{:?}",val);
            assert!(src==val);
        }
    }


*/
