use inner_prelude::*;
use rayon::prelude::*;


pub trait Bleek{
    type T:SweepTrait;
    fn collide(&mut self,a:ColSingle<Self::T>,b:ColSingle<Self::T>);
}



unsafe fn swap_unchecked<T>(list:&mut [T],a:usize,b:usize){
    let x=list.get_unchecked_mut(a) as *mut T;
    let y=list.get_unchecked_mut(b) as *mut T;
    std::ptr::swap(x,y)
}



pub struct Accessor<X:AxisTrait>{
    _p:PhantomData<X>
}
impl<X:AxisTrait> Accessor<X>{
   
    pub fn get<'b,Nu:NumTrait>(b:&'b Rect<Nu>)->&'b Range<Nu>{
        b.get_range(X::get())
    }
}



///Provides 1d collision detection.
pub struct Sweeper<T:SweepTrait>{
    helper: tools::PreVec<T>
}

impl<I:SweepTrait> Sweeper<I>{

    pub fn new()->Sweeper<I>{
        Sweeper{helper:tools::PreVec::new()} //TODO make callers decide?
    }


    ///Find colliding pairs using the mark and sweep algorithm.
    pub fn find<'a,A:AxisTrait,F: Bleek<T=I>>(
         &mut self,
         collision_botids: &'a mut[I],mut func:F) {

        //    Create a new temporary list called “activeList”.
        //    You begin on the left of your axisList, adding the first item to the activeList.
        //
        //    Now you have a look at the next item in the axisList and compare it with all items
        //     currently in the activeList (at the moment just one):
        //     - If the new item’s left is greater then the current activeList-item right, then remove
        //    the activeList-item from the activeList
        //     - otherwise report a possible collision between the new axisList-item and the current
        //     activeList-item.
        //
        //    Add the new item itself to the activeList and continue with the next item in the axisList.

        let active=self.helper.get_empty_vec_mut();
        
        //use odds::vec::VecExt;

        for curr_bot_id in collision_botids.iter_mut() {
           
            {
                let (curr_rect,curr_bot_id_val)=curr_bot_id.get_mut();
                let crr=Accessor::<A>::get(&(curr_rect.0));

                //change this to do retain and then iter
                active.retain(|that_bot_ind| {
                    let (that_rect,_)=that_bot_ind.get();
                    let brr=Accessor::<A>::get(&(that_rect.0));

                    if brr.right()<crr.left() {
                        false
                    } else {

                        true
                    }
                });
                for that_bot_ind in active.iter_mut(){
                    let (that_rect,that_val)=that_bot_ind.get_mut();
                    //let brr=Accessor::<A>::get(that_rect);
                    let a=ColSingle(curr_rect,curr_bot_id_val);
                    let b=ColSingle(that_rect,that_val);
                    func.collide( a,b);
                        
                }
            }
            active.push(curr_bot_id);
        }
    }


    pub fn find_bijective_parallel<A:AxisTrait,F: Bleek<T=I>>(
            &mut self,
            cols: (&mut [I], &mut [I]),
            mut func:F) {
    

        let mut xs=cols.0.iter_mut().peekable();
        let ys=cols.1.iter_mut();
        
        let active_x=self.helper.get_empty_vec_mut();

        for y in ys
        {
            while xs.peek().is_some(){
                let v={
                    let x=xs.peek().unwrap();
                    Accessor::<A>::get(&(x.get().0).0).left()>Accessor::<A>::get(&(y.get().0).0).right()
                };
                if v{
                    break;
                }else{

                    active_x.push(xs.next().unwrap());
                }
            }

            

            active_x.retain(|x:&mut &mut I|{
                if Accessor::<A>::get(&(x.get().0).0).right()<Accessor::<A>::get(&(y.get().0).0).left(){
                    false
                }else{
                    true
                }
            });

            let (y_rect,y_val)=y.get_mut();    
            for x in active_x.iter_mut(){

                let (x_rect,x_val)=x.get_mut();

                let a=ColSingle(x_rect,x_val);
                let b=ColSingle(y_rect,y_val);
                func.collide(a,b);
                            
            }
        }
    }
    
    /*
    fn assert_sorted<T:Accessor<T=I>>(col: &[I], accessor:&T)->bool {
        let mut last_val = -999999999999999999.0;
        for i in col {
            let pp=accessor.get(&i).left();
            assert!(pp >= last_val);
            last_val = pp;
        }
        true
    }*/
    
    //this can have some false positives.
    //but it will still prune a lot of bots.
    pub fn get_section<'a,A:AxisTrait>(arr:&'a mut [I],range:&Range<I::Num>)->&'a mut [I]{
    
        if arr.len()==0{
            return &mut [];
        }

        let mut start=0;
        for (e,i) in arr.iter().enumerate(){
            let rr=Accessor::<A>::get(&(i.get().0).0);
            if rr.right()>=range.left(){
                start=e;
                break;
            }
        }
        
        let mut end=arr.len();
        for (e,i) in arr[start..].iter().enumerate(){
            let rr=Accessor::<A>::get(&(i.get().0).0);
            if rr.left()>range.right(){
                end=start+e;
                break;
            }
        }

        return &mut arr[start..end]
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