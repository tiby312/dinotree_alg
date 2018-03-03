use inner_prelude::*;
use oned::Bleek;
use tree_alloc::NodeDyn;
use compt::LevelIter;
use compt::WrapGen;


pub trait ColMulti:Send+Sync+Clone{
    type T:SweepTrait;
    fn zero(&self,a:&mut <Self::T as SweepTrait>::Inner);
    //fn identity(&self)->T;
    fn add(&self,a:&mut <Self::T as SweepTrait>::Inner,&<Self::T as SweepTrait>::Inner);
    fn collide(&self,a:ColPair<Self::T>);
}

pub trait ColSeq{
  type T:SweepTrait;
  fn collide(&mut self,a:ColPair<Self::T>);
}

pub trait ColSing{
    type T:SweepTrait;
    fn collide(&mut self,a:ColSingle<Self::T>);  
}



pub struct ColMultiWrapper<'a,C:ColMulti+'a>(
    pub &'a mut C
);

impl<'a,C:ColMulti+'a> Bleek for ColMultiWrapper<'a,C>{
    type T=C::T;
    fn collide(&mut self,cc:ColPair<Self::T>){
        self.0.collide(cc);
    }
}


/*
struct BleekS<'a,B:BleekSync+'a>(
    pub &'a B
);

impl<'a,B:BleekSync+'a> Bleek for BleekS<'a,B>{
    type T=B::T;
    fn collide(&mut self,cc:ColPair<Self::T>){
        self.0.collide(cc);
    }
}
*/

fn go_down<'x,
    JJ:par::Joiner,
    H:DepthLevel,
    A:AxisTrait, //this axis
    B:AxisTrait, //parent axis
    C:CTreeIterator<Item=&'x mut NodeDyn<X>>+Send,
    X:SweepTrait+'x,
    F:ColMulti<T=X>>
    (
        sweeper:&mut Sweeper<F::T>,
        anchor:&mut &mut NodeDyn<X>,
        m:WrapGen<LevelIter<C>>,
        func:&mut F) {
    
    {
        let (mut bo,rest) = m.next();
        let &mut (level,ref mut nn)=bo.get_mut();
        
        let func={
            let mut b=ColMultiWrapper(func);
            self::for_every_bijective_pair::<A,B,_>(nn,anchor,sweeper,&mut b);       
            b.0
        };
        match rest{
            Some((left,right))=>{
                
                let div=*nn.divider();

                if B::get()==A::get(){
                    if !(div<anchor.get_container().start){
                        self::go_down::<JJ,H,A::Next,B,_,_,_>(sweeper,anchor,left,func);
                    };
                    if !(div>anchor.get_container().end){
                        self::go_down::<JJ,H,A::Next,B,_,_,_>(sweeper,anchor,right,func);
                    };
                }else{
                    if JJ::is_parallel() && !H::switch_to_sequential(level){ 
                        self::go_down_in_parallel::<H,A::Next,B,_,_,_>(sweeper,anchor,left,right,func.clone());
                        
                    }else{
                        self::go_down::<par::Sequential,H,A::Next,B,_,_,_>(sweeper,anchor,left,func);
                        self::go_down::<par::Sequential,H,A::Next,B,_,_,_>(sweeper,anchor,right,func);
                    }
                }               
            },
            _=>{}
        };
    }
}


fn go_down_in_parallel<'x,
    H:DepthLevel,
    A:AxisTrait, //this axis
    B:AxisTrait, //parent axis
    C:CTreeIterator<Item=&'x mut NodeDyn<X>>+Send,
    X:SweepTrait+'x,F:ColMulti<T=X>>
    (
        sweeper:&mut Sweeper<F::T>,
        anchor:&mut &mut NodeDyn<X>,
        left:WrapGen<LevelIter<C>>,
        right:WrapGen<LevelIter<C>>,
        clos:F) 
{
    //unsafely make a copy of the anchor
    //so that we can pass one to the other thread.
    //after both left and right functions have finished,
    //merge them back together.
    let (space,mut anchor_copy)=unsafe{
      
        struct Repr<X>{
            start:*mut X,
            len:usize
        }

        let siz=std::mem::size_of_val(*anchor);
        let anchor=(*anchor) as *mut NodeDyn<X>;
        let k:Repr<X>=unsafe{std::mem::transmute(anchor)};
        let lenn=k.len;
        //use u64 for alignment reasons.
        //wont work if needs to be aligned on a bigger boundary
        let mut space:Vec<u64>=Vec::with_capacity((siz/4)+1);
        let ptr:*mut u8=space.as_mut_ptr() as *mut u8;

        unsafe{std::ptr::copy(k.start as *mut u8,ptr,siz)};

        let mut anchor_copy:&mut NodeDyn<X>=std::mem::transmute(Repr{start:ptr,len:lenn});

        for i in anchor_copy.range.iter_mut(){
            clos.zero(i.get_mut().1);
        }
        (space,anchor_copy)
    };

    {
        let af=||{
            self::go_down::<par::Parallel,H,A,B,_,_,_>(sweeper,anchor,left,&mut clos.clone());
        };
        let bf=||{
            let mut sweeper=Sweeper::new(); 
            self::go_down::<par::Parallel,H,A,B,_,_,_>(&mut sweeper,&mut anchor_copy,right,&mut clos.clone());
        };

        rayon::join(af,bf);
    }

    for (a,b) in anchor.range.iter_mut().zip(anchor_copy.range.iter()){
        clos.add(a.get_mut().1,b.get().1);
    }
}

fn recurse<'x,
        A:AxisTrait,
        JJ:par::Joiner,
        X:SweepTrait+'x,
        H:DepthLevel,
        F:ColMulti<T=X>,
        C:CTreeIterator<Item=&'x mut NodeDyn<X>>+Send,
        K:TreeTimerTrait>(
        sweeper:&mut Sweeper<F::T>,
        m:LevelIter<C>,
        mut clos:F,mut timer_log:K) -> (K::Bag,F){
    
    timer_log.start();
    
    let ((level,mut nn),rest)=m.next();
 
    
    let mut tot_time=[0.0f64;3];
    

    let tt0=tools::Timer2::new();     
    {
        //TODO two many indirection?
        let mut b=ColMultiWrapper(&mut clos);
        self::sweeper_find_2d::<A::Next,_>(sweeper,nn.get_bots(),&mut b); 
       
    }

    tot_time[0]=tt0.elapsed();


    let tt1=tools::Timer2::new();
    let k=match rest{
        None=>{
            (timer_log.leaf_finish(),clos)
        },
        Some((mut left,mut right))=>{
            
            {
                let left=compt::WrapGen::new(&mut left);
                let right=compt::WrapGen::new(&mut right);
                
                if JJ::is_parallel() && !H::switch_to_sequential(level){
                    self::go_down_in_parallel::<H,A::Next,A,_,_,_>(sweeper,&mut nn,left,right,clos.clone());
                }else{
                    self::go_down::<par::Sequential,H,A::Next,A,_,_,_>(sweeper,&mut nn,left,&mut clos);
                    self::go_down::<par::Sequential,H,A::Next,A,_,_,_>(sweeper,&mut nn,right,&mut clos);
                }

            }

            let (ta,tb)=timer_log.next();      
            
            let (ta,tb,clos)=if JJ::is_parallel() && !H::switch_to_sequential(level)
            {             
                let af=|| {   
                    self::recurse::<A::Next,par::Parallel,_,H,_,_,_>(sweeper,left,clos.clone(),ta)
                };
                let bf= || {
                    let mut sweeper=Sweeper::new();  
                    self::recurse::<A::Next,par::Parallel,_,H,_,_,_>(&mut sweeper,right,clos.clone(),tb)
                };
                let ((ta,clos1),(tb,clos2))=rayon::join(af,bf);
                (ta,tb,clos1)
            }else{
                
                let (ta,clos)=self::recurse::<A::Next,par::Sequential,_,H,_,_,_>(sweeper,left,clos,ta);
                let (tb,clos)=self::recurse::<A::Next,par::Sequential,_,H,_,_,_>(sweeper,right,clos,tb);
                (ta,tb,clos)
            };
        
            (K::combine(ta,tb),clos)
        }
    };
    tot_time[1]=tt1.elapsed();

    k
}



pub fn for_every_col_pair_seq<A:AxisTrait,T:SweepTrait,H:DepthLevel,F:ColSeq<T=T>,K:TreeTimerTrait>
        (kdtree:&mut DynTree<A,T>,mut clos:F)->K::Bag{
    
    use std::marker::PhantomData;
    use std::cell::UnsafeCell;


    //#[derive(Copy,Clone)]
    pub struct Wrapper<'a,F:ColSeq+'a>(
        UnsafeCell<&'a mut F>
    );

    impl<'a,F:ColSeq+'a> Clone for Wrapper<'a,F> {
        fn clone(&self) -> Wrapper<'a,F> {
            unreachable!()
        }
    }

    impl<'a,F:ColSeq+'a> ColMulti for Wrapper<'a,F> {
        type T=F::T;
        fn zero(&self,a:&mut <Self::T as SweepTrait>::Inner){
            unreachable!()
        }
        fn add(&self,a:&mut <Self::T as SweepTrait>::Inner,b:&<Self::T as SweepTrait>::Inner){
            unreachable!()
        }
        fn collide(&self,a:ColPair<Self::T>){
            //TODO fix this
            //let k:&mut Wrapper<F>=unsafe{std::mem::transmute(self)};
            let k=unsafe{&mut *self.0.get()};
            k.collide(a);
        }
    }

    //Unsafely implement send and Sync
    //Safe to do since our algorithms do not do anything parallel,
    //Since we are starting it as sequential.
    //This is an invariant that we have to manuall uphold.
    unsafe impl<'a,F:ColSeq+'a> Send for Wrapper<'a,F>{}
    unsafe impl<'a,F:ColSeq+'a> Sync for Wrapper<'a,F>{}
    

    let wrapper=Wrapper(UnsafeCell::new(&mut clos));
    
    //All of the above is okay because we start with SEQUENTIAL
    self::for_every_col_pair_inner::<_,par::Sequential,_,DefaultDepthLevel,_,K>(kdtree,wrapper)
            
}


pub fn for_every_col_pair<A:AxisTrait,T:SweepTrait,H:DepthLevel,F:ColMulti<T=T>,K:TreeTimerTrait>
        (kdtree:&mut DynTree<A,T>,clos:F)->K::Bag
{
    self::for_every_col_pair_inner::<_,par::Parallel,_,DefaultDepthLevel,_,K>(kdtree,clos)    
}

fn for_every_col_pair_inner<A:AxisTrait,JJ:par::Joiner,T:SweepTrait,H:DepthLevel,F:ColMulti<T=T>,K:TreeTimerTrait>
        (kdtree:&mut DynTree<A,T>,clos:F)->K::Bag{

    let height=kdtree.get_height();
    let level=kdtree.get_level_desc();
    let dt=kdtree.get_iter_mut();
    let dt=compt::LevelIter::new(dt,level);
    let mut sweeper=Sweeper::new();  
    
    let h=K::new(height);
    let (bag,_)=self::recurse::<A,JJ,_,H,_,_,_>(&mut sweeper,dt,clos,h);
    bag
}



fn for_every_bijective_pair<A:AxisTrait,B:AxisTrait,F:Bleek>(
    
    this: &mut NodeDyn<F::T>,
    parent:&mut &mut NodeDyn<F::T>,
    sweeper:&mut Sweeper<F::T>,
    func:&mut F){
    let this_axis=A::get();
    let parent_axis=B::get();

    if this_axis != parent_axis {
        let r1 = Sweeper::get_section::<B>(&mut this.range,
                         &parent.container_box);
        let r2 = Sweeper::get_section::<A>(&mut parent.range,
                        &this.container_box);

        for inda in r1.iter_mut() {
            let (rect_a,aval)=inda.get_mut();
            for indb in r2.iter_mut() {
                let (rect_b,bval)=indb.get_mut();
                if rect_a.0.intersects_rect(&rect_b.0){
                    func.collide(ColPair{a:(rect_a,aval),b:(rect_b,bval)});
                }
            }
        }
    
    } else {
        self::sweeper_find_parallel_2d::<A::Next,_>(sweeper,this.get_bots(),parent.get_bots(),func);
    }
}


fn rect_recurse<'x,
    A:AxisTrait,
    T:SweepTrait+'x,
    C:CTreeIterator<Item=&'x mut NodeDyn<T>>,
    F:ColSing<T=T>>
    (m:C,rect:&Rect<T::Num>,func:&mut F){

    let (nn,rest)=m.next();
    {
        let sl=Sweeper::get_section::<A::Next>(nn.get_bots(),rect.get_range2::<A::Next>());
        
        for i in sl{
            let a = i.get_mut();
            func.collide(ColSingle(a.0,a.1)); 
        }
        
    }
    match rest{
        Some((left,right))=>{
            let div=nn.divider();

            let rr=rect.get_range2::<A>();
     
            if !(*div<rr.start){
                self::rect_recurse::<A::Next,_,_,_>(left,rect,func);
            }
            if !(*div>rr.end){
                self::rect_recurse::<A::Next,_,_,_>(right,rect,func);
            }
        },
        _=>{}
    }

}

pub fn for_all_in_rect<A:AxisTrait,T:SweepTrait,F:ColSing<T=T>>(
        tree:&mut DynTree<A,T>,rect: &Rect<T::Num>, closure:F) {
    
    struct Wrapper<F:ColSing>{
        rect:Rect<<F::T as SweepTrait>::Num>,
        closure:F
    };


    impl<F:ColSing> ColSing for Wrapper<F>{
        type T=F::T;
        fn collide(&mut self,a:ColSingle<Self::T>){
            if self.rect.contains_rect(&(a.0).0){
                self.closure.collide(a);
            }
        }
    }

    let mut wrapper=Wrapper{rect:*rect,closure};
    
    let ta=tree.get_iter_mut();
    self::rect_recurse::<A,_,_,_>(ta,rect,&mut wrapper);
}


///UNIMPLEMENTED
/*
pub fn nearest_k<A:AxisTrait,T:SweepTrait,F:FnMut(ColSingle<T>)>(
     _tree:&mut DynTree<A,T>,_rect: &Rect<T::Num>, _closure: &mut F) {
    unimplemented!()
}
*/



use colfind::bl::sweeper_find_2d;
use colfind::bl::sweeper_find_parallel_2d;
mod bl{
    use super::*;
    use std::marker::PhantomData;
    struct Bl<'a,A:AxisTrait,F:Bleek+'a>{
        a:&'a mut F,
        _p:PhantomData<A>
    }

    impl<'a,A:AxisTrait,F:Bleek+'a> Bleek for Bl<'a,A,F>{
        type T=F::T;
        fn collide(&mut self,cc:ColPair<F::T>){
            //only check if the opoosite axis intersects.
            //already know they intersect
            let a2=A::Next::get();//self.axis.next();
            if (cc.a.0).0.get_range(a2).intersects((cc.b.0).0.get_range(a2)){
                self.a.collide(cc);
            }
        }
    }

    //Bots a sorted along the axis.
    pub fn sweeper_find_2d<A:AxisTrait,F:Bleek>(sweeper:&mut Sweeper<F::T>,bots:&mut [F::T],clos2:&mut F){

        let b:Bl<A,_>=Bl{a:clos2,_p:PhantomData};
        sweeper.find::<A,_>(bots,b);   
    }
    pub fn sweeper_find_parallel_2d<A:AxisTrait,F:Bleek>(sweeper:&mut Sweeper<F::T>,bots1:&mut [F::T],bots2:&mut [F::T],clos2:&mut F){
        let b:Bl<A,_>=Bl{a:clos2,_p:PhantomData};
          
        sweeper.find_bijective_parallel::<A,_>((bots1, bots2), b );
    }
}

/*
fn assert_correctness(&self,tree:&KdTree,botman:&BotMan)->bool{
    for (level,axis) in kd_axis::AxisIter::with_axis(tree.tree.get_level_iter()) {
        if level.get_depth()!=tree.tree.get_height()-1{
            for n in level.iter(){
                let no=tree.tree.get_node(n);
                let cont_box=&no.container_box;// no.get_divider_box(&botman.prop,axis);

                let arr=&tree.collision_botids[no.container.get_range().as_int_range()];
                for b in arr{
                    let bot=botman.cont.get_bot(*b);
                    let circle=&botman.as_circle(bot);
                    assert!(cont_box.contains_circle(circle),"{:?}\n{:?}\n{:?}\n{:?}",no,(level,axis),cont_box,circle);
                }
            }
        }
        
    }
     

    let arr=&tree.collision_botids[tree.no_fit.end.0..];
    let mut cols=0;
    for (i, el1) in arr.iter().enumerate() {
        for el2 in arr[i + 1..].iter() {
            let bb=(*el1,*el2);
            let bots = botman.cont.get_bbotpair(bb);

            match bot::is_colliding(&botman.prop, bots) {
                Some(_) => {
                    cols+=1;
                }
                None => {
                }
            }
        }
    }

    let mut cls=0;
    for k in self.binner_helps.iter(){
        cls+=k.cols_found.len();
    }

    let lookup=|a:(BotIndex, BotIndex)|{
        for k in self.binner_helps.iter(){
            for j in k.cols_found.iter(){
                let aa=( (j.inds.0).0 ,(j.inds.1).0);
                let bb=((a.0).0,(a.1).0);
                if aa.0==bb.0 &&aa.1==bb.1{
                    return true;
                }
                if aa.0==bb.1 && aa.1==bb.0{
                    return true;
                }
            }
        }
        false            
    };
    if cols!=cls{
        println!("Cols fail! num collision exp:{:?},  calculated:{:?}",cols,cls);

        for (i, el1) in arr.iter().enumerate() {
            for el2 in arr[i + 1..].iter() {
                let bb=(*el1,*el2);
                let bots = botman.cont.get_bbotpair(bb);

                match bot::is_colliding(&botman.prop, bots) {
                    Some(_) => {
                        if !lookup(bb){
                            println!("Couldnt find {:?}",(bb,bots));

                            println!("in node:{:?}",(lookup_in_tree(tree,bb.0),lookup_in_tree(tree,bb.1)));
                            let n1=lookup_in_tree(tree,bb.0).unwrap();
                            let n2=lookup_in_tree(tree,bb.1).unwrap();
                            let no1=tree.tree.get_node(n1);
                            let no2=tree.tree.get_node(n2);
                            
                            println!("INTERSECTS={:?}",no1.cont.border.intersects_rect(&no2.cont.border));

                        }
                    }
                    None => {
                    }
                }
            }
        }
        assert!(false);
    }
    
    fn lookup_in_tree(tree:&BaseTree,b:BotIndex)->Option<NodeIndex>{
        for level in tree.tree.get_level_iter(){
            for nodeid in level.iter().rev() {
                
                let n = tree.tree.get_node(nodeid);
            
                let k=n.container.get_range().as_int_range();

                let arr=&tree.collision_botids[k];
                for i in arr{
                    if b.0==i.0{
                        return Some(nodeid);
                    }
                }
            }
        }
        return None
    }
    true
}*/






