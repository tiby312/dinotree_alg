
//!A collection of 1d functions that operate on lists of 2d objects.

use std;
use axgeom::Range;
use axgeom::Rect;
use tools::PreVec;
use SweepTrait;
use ColPair;
use BleekSync;
use Bleek;
use NumTrait;
use axgeom::AxisTrait;
use std::marker::PhantomData;

///Provides contains that support converting a closure to a struct that implements Bleek.
///Working with closures, you have to be carful with the recursion limit. This avoids
///having to be careful of how many wrapper closures you make.
pub mod sup{
    use super::*;
    use std::marker::PhantomData;
    use tools::PhantomSendSync;


    pub struct BleekBF<'a,T:SweepTrait+'a,F:Fn(ColPair<T>)+Sync+'a>{
        a:&'a F,
        _p:PhantomSendSync<T>
    }
    impl<'a,T:SweepTrait+'a,F:Fn(ColPair<T>)+Sync+'a> Copy for BleekBF<'a,T,F> { }
    impl<'a,T:SweepTrait+'a,F:Fn(ColPair<T>)+Sync+'a> Clone for BleekBF<'a,T,F> { 
        fn clone(&self) -> BleekBF<'a,T,F> {
            *self
        }
    }

    impl<'a,T:SweepTrait+'a,F:Fn(ColPair<T>)+Sync+'a> BleekBF<'a,T,F>{
        pub fn new(a:&'a F)->BleekBF<'a,T,F>{
            BleekBF{a:a,_p:PhantomSendSync(PhantomData)}
        }
    }

    impl<'a,T:SweepTrait+'a,F:Fn(ColPair<T>)+Sync+'a> BleekSync for BleekBF<'a,T,F>{
        type T=T;
        fn collide(&self,cc:ColPair<Self::T>){
            (self.a)(cc);
        }
    }

    pub struct BleekSF<'a,T:SweepTrait+'a,F:FnMut(ColPair<T>)+'a>{
        a:&'a mut F,
        _p:PhantomData<T>
    }
    impl<'a,T:SweepTrait+'a,F:FnMut(ColPair<T>)+'a> BleekSF<'a,T,F>{
        pub fn new(a:&'a mut F)->BleekSF<'a,T,F>{
            BleekSF{a:a,_p:PhantomData}
        }
    }

    impl<'a,T:SweepTrait+'a,F:FnMut(ColPair<T>)+'a> Bleek for BleekSF<'a,T,F>{
        type T=T;
        fn collide(&mut self,cc:ColPair<Self::T>){
            (self.a)(cc);
        }
    }
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




///The results of the binning process.
pub struct Binned<'a,T:'a>{
    pub left:&'a mut [T],
    pub middile:&'a mut [T],
    pub right:&'a mut [T],
}

/// Sorts the bots into three bins. Those to the left of the divider, those that intersect with the divider, and those to the right.
/// They will be laid out in memory s.t.  middile<left<right
pub fn bin<'a,'b,A:AxisTrait,X:SweepTrait>(med:&X::Num,bots:&'b mut [X])->Binned<'b,X>{
    let bot_len=bots.len();
        
    let mut left_end=0;
    let mut middile_end=0;
    
    //     |    middile   |   left|              right              |---------|
    //
    //                ^           ^                                  ^
    //              middile_end    left_end                      index_at

    for index_at in 0..bot_len{
        unsafe{
            match Accessor::<A>::get(bots.get_unchecked(index_at).get().0).left_or_right_or_contain(med){
                
                //If the divider is less than the bot
                std::cmp::Ordering::Equal=>{
                    //left
                    swap_unchecked(bots,index_at,left_end);
                    swap_unchecked(bots,left_end,middile_end);
                    middile_end+=1;
                    left_end+=1;  
                },
                //If the divider is greater than the bot
                std::cmp::Ordering::Greater=>{
                    //middile
                    swap_unchecked(bots,index_at,left_end);
                    left_end+=1;
                },
                std::cmp::Ordering::Less=>{
                    //right                    
                }
            }
        }
        
    }

    let (rest,right)=bots.split_at_mut(left_end);
    let (middile,left)=rest.split_at_mut(middile_end);

    debug_assert!(left.len()+right.len()+middile.len()==bot_len);
    //debug_assert!(bot_len==index_at,"{:?} ,{:?}",bot_len,index_at);

    Binned{left:left,middile:middile,right:right}
}


///Provides 1d collision detection.
pub struct Sweeper<T:SweepTrait>{
    helper: PreVec<T>
}

impl<I:SweepTrait> Sweeper<I>{

    pub fn new()->Sweeper<I>{
        Sweeper{helper:PreVec::with_capacity(32)} //TODO make callers decide?
    }

    ///Sorts the bots.
    pub fn update<A:AxisTrait>(collision_botids: &mut [I]) {

        let sclosure = |a: &I, b: &I| -> std::cmp::Ordering {
            let (p1,p2)=(Accessor::<A>::get(a.get().0).left(),Accessor::<A>::get(b.get().0).left());
            if p1 > p2 {
                return std::cmp::Ordering::Greater;
            }
            std::cmp::Ordering::Less
        };


        collision_botids.sort_unstable_by(sclosure);

        //debug_assert!(Self::assert_sorted(collision_botids,accessor));
    }
 

    ///Find colliding pairs using the mark and sweep algorithm.
    pub fn find<'a,A:AxisTrait,F: Bleek<T=I>>(
         &mut self,
         collision_botids: &'a mut[I],func:&mut F) {

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
                let crr=Accessor::<A>::get(curr_rect);

                //change this to do retain and then iter
                active.retain(|that_bot_ind| {
                    let (that_rect,_)=that_bot_ind.get();
                    let brr=Accessor::<A>::get(that_rect);

                    if brr.right()<crr.left() {
                        false
                    } else {

                        true
                    }
                });
                for that_bot_ind in active.iter_mut(){
                    let (that_rect,that_val)=that_bot_ind.get_mut();
                    //let brr=Accessor::<A>::get(that_rect);

                    func.collide( ColPair{a:(curr_rect,curr_bot_id_val), b: (that_rect,that_val)});
                        
                }
            }
            active.push(curr_bot_id);
        }
    }


    pub fn find_bijective_parallel<A:AxisTrait,F: Bleek<T=I>>(
            &mut self,
            cols: (&mut [I], &mut [I]),
            func:&mut F) {
    

        let mut xs=cols.0.iter_mut().peekable();
        let ys=cols.1.iter_mut();
        
        let active_x=self.helper.get_empty_vec_mut();

        for y in ys
        {
            while xs.peek().is_some(){
                let v={
                    let x=xs.peek().unwrap();
                    Accessor::<A>::get(x.get().0).left()>Accessor::<A>::get(y.get().0).right()
                };
                if v{
                    break;
                }else{

                    active_x.push(xs.next().unwrap());
                }
            }

            

            active_x.retain(|x:&&mut I|{
                if Accessor::<A>::get(x.get().0).right()<Accessor::<A>::get(y.get().0).left(){
                    false
                }else{
                    true
                }
            });

            let (y_rect,y_val)=y.get_mut();    
            for x in active_x.iter_mut(){

                let (x_rect,x_val)=x.get_mut();

                func.collide( ColPair{a:(x_rect,x_val), b: (y_rect,y_val)});
                            
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
            let rr=Accessor::<A>::get(i.get().0);
            if rr.right()>=range.left(){
                start=e;
                break;
            }
        }
        
        let mut end=arr.len();
        for (e,i) in arr[start..].iter().enumerate(){
            let rr=Accessor::<A>::get(i.get().0);
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