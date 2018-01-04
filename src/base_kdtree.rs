use axgeom;
use oned::Sweeper;
//use super::Blee;
use super::median::MedianStrat;
use compt;
use SweepTrait;
//use kdtree::base_kdtree::kd_axis::AxisIter;
//use kdtree::colfind::NodeTrait;
use compt::CTreeIterator;
//use compt::LevelDesc;
use std;
//use std::fmt::Debug;
use compt::DownTMut;
use compt::LevelIter;
//use axgeom::Axis;
//use DefaultDepthLevel;
use TreeTimer;
//use tools;
use NumTrait;
use tools::par::Joiner;
use tools::par;
use axgeom::AxisTrait;
//use compt::LevelDesc;
use std::marker::PhantomData;
//use self::div_axis::*;
use DepthLevel;
use rayon;
use Bag;
use TreeTimer2;
use TreeTimerTrait;
/*
pub mod div_axis{
    use compt::CTreeIterator;
    use axgeom::Axis;
    use std::marker::PhantomData;
    //use compt::CTreeIterator;
    use axgeom;

   
    /*
    //Signify's that this axis that the divider is splitting.
    //so for example if the axis was the xaxis, then the divider
    //would be imagined as a vertical line.
    #[derive(Copy,Clone,Debug,PartialEq)]
    pub struct DivAxis(Axis);

    impl DivAxis{
        pub fn new(a:Axis)->DivAxis{
            DivAxis(a)
        }
        pub fn get(&self)->Axis{
            self.0
        }
        pub fn get_line(&self)->LineAxis{
            LineAxis(self.0.next())
        }
    }

    //Signifies the axis perpendicular to the axis the divider is partitioning against.
    #[derive(Copy,Clone,Debug,PartialEq)]
    pub struct LineAxis(Axis);
    impl LineAxis{
        pub fn get(&self)->Axis{
            self.0
        }
    }

    pub struct DivAxisIter<C:CTreeIterator>{
        c:C,
        a:DivAxis
    }
    impl<C:CTreeIterator> DivAxisIter<C>{
        pub fn new(a:DivAxis,c:C)->DivAxisIter<C>{
            DivAxisIter{c,a}
        }
    }

    impl<C:CTreeIterator> CTreeIterator for DivAxisIter<C>{
        type Item=(DivAxis,C::Item);
        fn next(self)->(Self::Item,Option<(Self,Self)>){
            let (nn,rest)=self.c.next();

            let newrest=match rest{
                Some((left,right))=>{
                    let n=DivAxis(self.a.0.next());
                    Some((DivAxisIter{c:left,a:n},DivAxisIter{c:right,a:n}))
                },
                None=>{
                    None
                }
            };
            ((self.a,nn),newrest)
        }
    }
    */
}
*/







///A KdTree construction
pub struct KdTree<'a,A:AxisTrait,T:SweepTrait+'a> {
    tree: compt::GenTree<Node2<'a,T>>,
    _p:PhantomData<A>
}

/*
impl<'a,A:AxisTrait,T:SweepTrait+'a> KdTree<'a,A,T>{

    pub fn get_height(&self)->usize{
        self.tree.get_height()
    }
}
*/

impl<'a,A:AxisTrait,T:SweepTrait+'a> KdTree<'a,A,T>{
    /*
    pub fn get_tree_mut(&mut self)->&mut compt::GenTree<Node2<'a,T>>{
        &mut self.tree
    }
    */
    pub fn get_tree(&self)->&compt::GenTree<Node2<'a,T>>{
        &self.tree
    }
    ///Construct new tree by swapping the elements of the passed slice.
    //TODO use impl trait for these
    /*
    pub fn new<JJ:Joiner,M:MedianStrat<Num=T::Num>>(tc:&mut TreeCache<A,T::Num>,bots:&'a mut [T])->KdTree<'a,A,T>{
        new_tree::<A,JJ,T,DefaultDepthLevel,M>(bots,tc)
    }
    */

    
    
    //not this
    //   |-------------------|===|----------------------------|
    //   |--------|===|------|   |-----------|=====|----------|
    //   |--|==|--|   |-|=|--|   |---|====|--|     |--|===|---|
    //   |==|  |==|   |=| |==|   |===|    |==|     |==|   |===|

    //pre order has better space locality as you traverse down the tree
    //   |===|-------------------|----------------------------|
    //       |===|--------|------|=====|-----------|----------|
    //           |==|--|--|=|-|--|     |====|---|--|===|--|---|
    //              |==|==| |=|==|          |===|==|   |==|===|
    ///Order of the bots will be in bfs
    


    ///This will traverse the tree in dfs and rejoin all the slices to return the original slice.
    ///This is normally redundant since one could simply let the KdTree exit scope to be able to use the original slice.
    ///TODO test
    /*
    pub fn deconstruct(self)->&'a mut [T]{

        let mut head=None;
        self.tree.into_dfs_preorder(&mut |n:Node2<'a,T>|{
            
            match head.take(){
                Some(x)=>{
                    head=Some(tools::join_mut(x,n.range));
                },
                None=>{
                    head=Some(n.range);
                }
            }
        });
        
        //tree necessailty has a root.
        head.unwrap()
    }
    */

    pub fn into_tree(self)->compt::GenTree<Node2<'a,T>>{
        let KdTree{tree,_p}=self;
        tree
    }
}





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
    pub fn new<JJ:Joiner>(height:usize)->TreeCache<A,Nu>{
        let num_nodes=compt::compute_num_nodes(height);
        
        //HEIGHT IS ONE LESS
        let treecache_height=height-1;
        let t= compt::GenTree::from_bfs(&mut ||{DivNode{divider:std::default::Default::default()}},treecache_height);

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





pub struct Node2<'a,T:SweepTrait+'a>{ 

    pub divider:T::Num,

    //only valid if the node has bots in it.
    pub container_box:axgeom::Range<T::Num>,

    pub range:&'a mut [T]
}

//TODO move to kdtree
//The border Rect is used purely for graphics!!!!!!!!!!!!!!!!!!!!!!!
//TODO not true. it is used by the relax median to bound the meds. Consider passing two rects.
pub fn new_tree<'a,A:AxisTrait,JJ:par::Joiner,T:SweepTrait,H:DepthLevel,Z:MedianStrat<Num=T::Num>>(rest:&'a mut [T],tc:&mut TreeCache<A,T::Num>) -> (KdTree<'a,A,T>,Bag) {
    
    let height=tc.height;
    
    
    //let ptrs=rest as *mut [T];//(&mut rest[0] as *mut T,rest.len());
    

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
        let j=LevelIter::new(ttree.create_down_mut(),level);
        let t=TreeTimer2::new(height);
        self::recurse_rebal::<A,T,H,Z,JJ>(rest,j,Some(m),t)
    };
    (KdTree{tree:ttree,_p:PhantomData},bag)
}


fn recurse_rebal<'b,A:AxisTrait,T:SweepTrait,H:DepthLevel,Z:MedianStrat<Num=T::Num>,JJ:par::Joiner>(
    rest:&'b mut [T],
    down:LevelIter<DownTMut<Node2<'b,T>>>,
    down2:Option<DownTMut<DivNode<T::Num>>>,mut timer_log:TreeTimer2)->Bag{

    timer_log.start();
    
    let ((level,nn),restt)=down.next();
    let depth=level.get_depth();
    fn create_node<A:AxisTrait,T:SweepTrait>(divider:T::Num,range:&mut [T])->Node2<T>{
        Sweeper::update::<A::Next>(range);
            
        let container_box=self::create_container_rect::<A,_>(range);
        Node2{divider,container_box,range}
    }

    match restt{
        None=>{
            debug_assert!(down2.is_none());
            //println!("sorting size={} depth={}",rest.len(),depth);
            *nn=create_node::<A,_>(std::default::Default::default(),rest);

            //let elapsed=timer.elapsed();
            //timer_log.add_to_depth(depth,elapsed);
            timer_log.leaf_finish()
        },
        Some((lleft,rright))=>{
            let (div,div_rest)=down2.unwrap().next();

            let (divl,divr)=match div_rest{
                Some((l,r))=>{
                    (Some(l),Some(r))
                },
                None=>{
                    (None,None)
                }
            };

            let depth=level.get_depth();
            
            let (med,binned)=Z::compute::<A,_>(depth,rest,&mut div.divider);
            

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
                        let nj=create_node::<A,_>(med,binned_middile);
                        let ba=self::recurse_rebal::<A::Next,T,H,Z,par::Parallel>(binned_left,lleft,divl,ta);
                        (nj,ba)
                    };

                    let bf=move || {
                        self::recurse_rebal::<A::Next,T,H,Z,par::Parallel>(binned_right,rright,divr,tb)
                    };
                    rayon::join(af,bf)
                };
                //timer_log.combine_one_less(ll2);  
                (nj,ba,bb)
            }else{
                let nj=create_node::<A,_>(med,binned_middile);
                let ba=self::recurse_rebal::<A::Next,T,H,Z,par::Sequential>(binned_left,lleft,divl,ta);
                let bb=self::recurse_rebal::<A::Next,T,H,Z,par::Sequential>(binned_right,rright,divr,tb);
                (nj,ba,bb)
            };

                

            *nn=nj;
            TreeTimer2::combine(ba,bb)
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