use axgeom;
use oned::Sweeper;
use super::median::MedianStrat;
use compt;
use compt::CTreeIterator;
use tools::par;
use axgeom::AxisTrait;

use std::marker::PhantomData;
use treetimer::*;
use *;



#[derive(Copy,Clone,Debug)]
pub struct DivNode<Nu:Ord+Copy+std::fmt::Debug>{
    divider:Nu    
}
impl<Nu:Ord+Copy+std::fmt::Debug> DivNode<Nu>{
    pub fn divider(&self)->&Nu{
        &self.divider
    }
}

///This preserves some state of the medians at each level between kdtree constructions.
pub struct TreeCache<A:AxisTrait,Nu:NumTrait>{
    height:usize,
    num_nodes:usize,
    medtree:compt::GenTree<DivNode<Nu>>,
    _p:PhantomData<A>
}
impl<A:AxisTrait,Nu:NumTrait> TreeCache<A,Nu>{
    ///The tree cache contains within it a tree to keep a persistant state between construction of the kdtree.
    ///So the height of the kdtree is decided here, before the creation of the tree.
    pub fn new(height:usize)->TreeCache<A,Nu>{
        let num_nodes=compt::compute_num_nodes(height);
        
        let t= compt::GenTree::from_bfs(&mut ||{DivNode{divider:std::default::Default::default()}},height);

        TreeCache{medtree:t,num_nodes:num_nodes,height:height,_p:PhantomData}
    }

    pub fn get_tree(&self)->&compt::GenTree<DivNode<Nu>>{
        &self.medtree
    }
    pub fn get_num_nodes(&self)->usize{
        self.num_nodes
    }

    pub fn get_height(&self)->usize{
        self.height
    }  
}


///A KdTree construction
pub struct KdTree<'a,A:AxisTrait,T:SweepTrait+'a> {
    tree: compt::GenTree<Node2<'a,T>>,
    _p:PhantomData<A>
}

impl<'a,A:AxisTrait,T:SweepTrait+'a> KdTree<'a,A,T>{

    pub fn new<JJ:par::Joiner,H:DepthLevel,Z:MedianStrat<Num=T::Num>,K:TreeTimerTrait>(rest:&'a mut [T],tc:&mut TreeCache<A,T::Num>,medianstrat:&Z) -> (KdTree<'a,A,T>,K::Bag) {
        let height=tc.height;
        
        let mut ttree=compt::GenTree::from_bfs(&mut ||{
            //let rect=axgeom::Rect::new(0.0,0.0,0.0,0.0);
            let rest=&mut [];
            use std;

            let co=self::create_container_rect::<A,_>(rest);
            
            Node2{divider:std::default::Default::default(),container_box:co,range:rest}
        },height);

        let bag={

            let level=ttree.get_level_desc();
            let m=tc.medtree.create_down_mut();
            let j=compt::LevelIter::new(m.zip(ttree.create_down_mut()),level);
            let t=K::new(height);
            self::recurse_rebal::<A,T,H,_,JJ,K>(rest,j,t,medianstrat)
        };
        (KdTree{tree:ttree,_p:PhantomData},bag)
    }

    pub fn get_tree(&self)->&compt::GenTree<Node2<'a,T>>{
        &self.tree
    }

    pub fn into_tree(self)->compt::GenTree<Node2<'a,T>>{
        let KdTree{tree,_p}=self;
        tree
    }
}


pub struct Node2<'a,T:SweepTrait+'a>{ 

    pub divider:T::Num,

    //only valid if the node has bots in it.
    pub container_box:axgeom::Range<T::Num>,

    pub range:&'a mut [T]
}


fn recurse_rebal<'b,A:AxisTrait,T:SweepTrait,H:DepthLevel,Z:MedianStrat<Num=T::Num>,JJ:par::Joiner,K:TreeTimerTrait>(
    rest:&'b mut [T],
    down:compt::LevelIter<compt::Zip<compt::DownTMut<DivNode<T::Num>>,compt::DownTMut<Node2<'b,T>>>>,
    mut timer_log:K,medianstrat:&Z)->K::Bag{

    timer_log.start();
    
    let ((level,(div,nn)),restt)=down.next();
    //let depth=level.get_depth();
    fn create_node<A:AxisTrait,T:SweepTrait,JJ:par::Joiner>(divider:T::Num,range:&mut [T])->Node2<T>{
        Sweeper::update::<A::Next,JJ>(range);
            
        let container_box=self::create_container_rect::<A,_>(range);
        Node2{divider,container_box,range}
    }

    match restt{
        None=>{
 
            *nn=create_node::<A,_,JJ>(std::default::Default::default(),rest);
            
            timer_log.leaf_finish()
        },
        Some((lleft,rright))=>{

            //let depth=level.get_depth();
            
            let (med,binned)=medianstrat.compute::<A,_>(rest,&mut div.divider);

            let binned_left=binned.left;
            let binned_middile=binned.middile;
            let binned_right=binned.right;                

            //let elapsed=timer.elapsed();
            //timer_log.add_to_depth(depth,elapsed);
            let (ta,tb)=timer_log.next();

            let (nj,ba,bb)=if JJ::is_parallel() && !H::switch_to_sequential(level){
                //let mut ll2=timer_log.clone_one_less_depth(); 
                
                let ((nj,ba),bb)={
                    let af=move || {
                        let nj=create_node::<A,_,JJ>(med,binned_middile);
                        let ba=self::recurse_rebal::<A::Next,T,H,Z,par::Parallel,K>(binned_left,lleft,ta,medianstrat);
                        (nj,ba)
                    };

                    let bf=move || {
                        self::recurse_rebal::<A::Next,T,H,Z,par::Parallel,K>(binned_right,rright,tb,medianstrat)
                    };
                    rayon::join(af,bf)
                };
                //timer_log.combine_one_less(ll2);  
                (nj,ba,bb)
            }else{
                let nj=create_node::<A,_,JJ>(med,binned_middile);
                let ba=self::recurse_rebal::<A::Next,T,H,Z,par::Sequential,K>(binned_left,lleft,ta,medianstrat);
                let bb=self::recurse_rebal::<A::Next,T,H,Z,par::Sequential,K>(binned_right,rright,tb,medianstrat);
                (nj,ba,bb)
            };

                

            *nn=nj;
            K::combine(ba,bb)
        }
    }
}



fn create_container_rect<A:AxisTrait,T:SweepTrait>(middile:&[T])->axgeom::Range<T::Num>{
    
    let container_rect=match middile.split_first(){
        Some((first,rest))=>{
            let mut container_rect=first.get().0.get_range2::<A>().clone();
            for i in rest{
                container_rect.grow_to_fit(i.get().0.get_range2::<A>());
            }
            container_rect
        },
        None=>{
            //TODO this wont accidentaly collide with anything?
            
            let d=std::default::Default::default();
            axgeom::Range{start:d,end:d}
        }
    };
    container_rect
}