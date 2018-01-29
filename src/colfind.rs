use axgeom::Rect;
use oned::Sweeper;
use super::DepthLevel;

use rayon;
use compt::CTreeIterator;
use tools::par;
use compt::WrapGen;
use dyntree::DynTree;
use compt::LevelIter;
use axgeom::AxisTrait;
use tree_alloc::NodeDyn;
use compt;


use *;

use support::DefaultDepthLevel;
use treetimer::*;


use oned::BleekSync;
use oned::Bleek;



struct BleekS<'a,B:BleekSync+'a>(
    pub &'a B
);

impl<'a,B:BleekSync+'a> Bleek for BleekS<'a,B>{
    type T=B::T;
    fn collide(&mut self,cc:ColPair<Self::T>){
        self.0.collide(cc);
    }
}

fn go_down<'x,
    A:AxisTrait, //this axis
    B:AxisTrait, //parent axis
    C:CTreeIterator<Item=&'x mut NodeDyn<X>>,
    X:SweepTrait+'x,F:Bleek<T=X>>
    (
        sweeper:&mut Sweeper<F::T>,
        anchor:&mut &mut NodeDyn<X>,
        m:WrapGen<LevelIter<C>>,
        func:&mut F) {
    
    {
        let (mut bo,rest) = m.next();
        let &mut (_level,ref mut nn)=bo.get_mut();
        
        self::for_every_bijective_pair::<A,B,_>(nn,anchor,sweeper,func);       
        
        match rest{
            Some((left,right))=>{
                
                let div=*nn.divider();

                if B::get()==A::get(){
                    if !(div<anchor.get_container().start){
                        self::go_down::<A::Next,B,_,_,_>(sweeper,anchor,left,func);
                    };
                    if !(div>anchor.get_container().end){
                        self::go_down::<A::Next,B,_,_,_>(sweeper,anchor,right,func);
                    };
                }else{
                    self::go_down::<A::Next,B,_,_,_>(sweeper,anchor,left,func);
                    self::go_down::<A::Next,B,_,_,_>(sweeper,anchor,right,func);
                }               
            },
            _=>{}
        };
    }
}

fn recurse<'x,
        A:AxisTrait,
        JJ:par::Joiner,
        X:SweepTrait+'x,
        H:DepthLevel,
        F:BleekSync<T=X>,
        C:CTreeIterator<Item=&'x mut NodeDyn<X>>+Send,
        K:TreeTimerTrait>(
        sweeper:&mut Sweeper<F::T>,
        m:LevelIter<C>,
        clos:&F,mut timer_log:K) -> K::Bag{
    
    timer_log.start();
    
    let ((level,mut nn),rest)=m.next();
 
    let depth=level.get_depth(); 
    let mut b=BleekS(clos);

    self::sweeper_find_2d::<A::Next,_>(sweeper,nn.get_bots(),&mut b); 

    match rest{
        None=>{
            timer_log.leaf_finish()
        },
        Some((mut left,mut right))=>{
            
            {
                let left=compt::WrapGen::new(&mut left);
                let right=compt::WrapGen::new(&mut right);
                
                self::go_down::<A::Next,A,_,_,_>(sweeper,&mut nn,left,&mut b);
                self::go_down::<A::Next,A,_,_,_>(sweeper,&mut nn,right,&mut b); 
            }

            let (ta,tb)=timer_log.next();      
            
            let (ta,tb)=if JJ::is_parallel() && !H::switch_to_sequential(level)
            {             
                let af=|| {   
                    self::recurse::<A::Next,par::Parallel,_,H,_,_,_>(sweeper,left,clos,ta)
                };
                let bf= || {
                    let mut sweeper=Sweeper::new();  
                    self::recurse::<A::Next,par::Parallel,_,H,_,_,_>(&mut sweeper,right,clos,tb)
                };
                rayon::join(af,bf)
            }else{
                (
                    self::recurse::<A::Next,par::Sequential,_,H,_,_,_>(sweeper,left,clos,ta),
                    self::recurse::<A::Next,par::Sequential,_,H,_,_,_>(sweeper,right,clos,tb)
                )
            };
        
            K::combine(ta,tb)
        }
    }
}



pub fn for_every_col_pair_seq<A:AxisTrait,T:SweepTrait+Copy,H:DepthLevel,F:Bleek<T=T>,K:TreeTimerTrait>
        (kdtree:&mut DynTree<A,T>,clos:&mut F)->K::Bag{
           

    pub struct BleekSF2<T:SweepTrait+Copy,B:Bleek<T=T>>{
        a:*mut B,
    }
    
    unsafe impl<T:SweepTrait+Copy,B:Bleek<T=T>> Send for BleekSF2<T,B>{}
    unsafe impl<T:SweepTrait+Copy,B:Bleek<T=T>> Sync for BleekSF2<T,B>{}
    impl<T:SweepTrait+Copy,B:Bleek<T=T>> Copy for BleekSF2<T,B>{}
    impl<T:SweepTrait+Copy,B:Bleek<T=T>> Clone for BleekSF2<T,B>{
        fn clone(&self) -> Self {
            *self
        }
    }
    impl<T:SweepTrait+Copy,B:Bleek<T=T>> BleekSync for BleekSF2<T,B>{
        type T=B::T;
        fn collide(&self,cc:ColPair<Self::T>){
            unsafe{(*self.a).collide(cc)};
        }
    }

    let b=BleekSF2{a:clos};

    //All of the above is okay because we start with SEQUENTIAL
    self::for_every_col_pair_inner::<A,par::Sequential,_,DefaultDepthLevel,_,K>(kdtree,&b)
            
}

pub fn for_every_col_pair<A:AxisTrait,T:SweepTrait+Copy,H:DepthLevel,F:BleekSync<T=T>,K:TreeTimerTrait>
        (kdtree:&mut DynTree<A,T>,clos:&F)->K::Bag
{
    self::for_every_col_pair_inner::<A,par::Parallel,_,DefaultDepthLevel,_,K>(kdtree,clos)    
}

fn for_every_col_pair_inner<A:AxisTrait,JJ:par::Joiner,T:SweepTrait+Copy,H:DepthLevel,F:BleekSync<T=T>,K:TreeTimerTrait>
        (kdtree:&mut DynTree<A,T>,clos:&F)->K::Bag{

    let height=kdtree.get_height();
    let level=kdtree.get_level_desc();
    let dt=kdtree.get_iter_mut();
    let dt=compt::LevelIter::new(dt,level);
    let mut sweeper=Sweeper::new();  
    
    let h=K::new(height);
    self::recurse::<A,JJ,_,H,_,_,_>(&mut sweeper,dt,clos,h) 
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
                if rect_a.intersects_rect(rect_b){
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
    T:SweepTrait+Copy+'x,
    C:CTreeIterator<Item=&'x mut NodeDyn<T>>,
    F:FnMut(ColSingle<T>)>(
    m:C,rect:&Rect<T::Num>,func:&mut F){

    let (nn,rest)=m.next();
    {
        let sl=Sweeper::get_section::<A::Next>(nn.get_bots(),rect.get_range2::<A::Next>());
        
        for i in sl{
            let a = i.get_mut();
            func(ColSingle(a.0,a.1)); 
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

pub fn for_all_in_rect<A:AxisTrait,T:SweepTrait+Copy,F:FnMut(ColSingle<T>)>(
        tree:&mut DynTree<A,T>,rect: &Rect<T::Num>, closure: &mut F) {
    
    let mut fu=|a:ColSingle<T>|{
                if rect.contains_rect(a.0){
                    closure(a);
                }
            };
    
    let ta=tree.get_iter_mut();
    self::rect_recurse::<A,_,_,_>(ta,rect,&mut fu);
}

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
            if cc.a.0.get_range(a2).intersects(cc.b.0.get_range(a2)){
                self.a.collide(cc);
            }
        }
    }

    //Bots a sorted along the axis.
    pub fn sweeper_find_2d<A:AxisTrait,F:Bleek>(sweeper:&mut Sweeper<F::T>,bots:&mut [F::T],clos2:&mut F){

        let mut b:Bl<A,_>=Bl{a:clos2,_p:PhantomData};
        sweeper.find::<A,_>(bots,&mut b);   
    }
    pub fn sweeper_find_parallel_2d<A:AxisTrait,F:Bleek>(sweeper:&mut Sweeper<F::T>,bots1:&mut [F::T],bots2:&mut [F::T],clos2:&mut F){
        let mut b:Bl<A,_>=Bl{a:clos2,_p:PhantomData};
          
        sweeper.find_bijective_parallel::<A,_>((bots1, bots2),&mut b );
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






