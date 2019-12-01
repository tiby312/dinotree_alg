//! Contains code to manipulate the dinotree data structure and some of its query algorithms
//! to help analyze and measure their performance.


use crate::query::*;
use crate::inner_prelude::*;


pub use crate::tree::notsorted::NotSorted;
pub use crate::query::colfind::QueryBuilder;
pub use crate::query::colfind::NotSortedQueryBuilder;

pub use crate::tree::builder::DinoTreeBuilder;

pub trait HasId{
    fn get_id(&self)->usize;
}

#[derive(Debug,Eq,Ord,PartialOrd,PartialEq)]
struct IDPair{
    a:usize,
    b:usize
}

impl IDPair{
    fn new(a:usize,b:usize)->IDPair{
        let (a,b)=if a<=b{
            (a,b)
        }else{
            (b,a)
        };
        IDPair{a,b}
    }
}


struct UnitMut2<N>{
    id:usize,
    mag:N
}

pub struct NaiveAlgs<'a,T>{
    bots:&'a mut [T]
}

impl<'a,T:Aabb> NaiveAlgs<'a,T>{
    #[must_use]
    pub fn new(bots:&'a mut [T])->NaiveAlgs<T>{
        NaiveAlgs{bots}
    }


    #[must_use]
    pub fn raycast_mut(
        &mut self,
        ray:raycast::Ray<T::Num>,
        rtrait: &mut impl raycast::RayCast<N=T::Num,T=T> )->raycast::RayCastResult<T>{
        let bots=PMut::new(self.bots);
        raycast::raycast_naive_mut(bots,ray,rtrait)
    }

    #[must_use]
    pub fn k_nearest_mut(
        &mut self,
        point:Vec2<T::Num>,
        num:usize,
        knear:&mut impl k_nearest::Knearest<N=T::Num,T=T>) -> Vec<k_nearest::KnearestResult<T>>{
        let bots=PMut::new(self.bots);
        k_nearest::k_nearest_naive_mut(bots,point,num,knear)
    }

    pub fn for_all_not_in_rect_mut(&mut self,rect:&Rect<T::Num>,func:impl FnMut(PMut<T>)){
        let bots=PMut::new(self.bots);
        rect::naive_for_all_not_in_rect_mut(bots,rect,func);
    }

    #[cfg(feature = "nbody")]
    pub fn nbody(&mut self,func:impl FnMut(PMut<T>,PMut<T>)){
        nbody::naive_mut(self.bots,func);
    }

    pub fn for_all_intersect_rect_mut(&mut self,rect:&Rect<T::Num>,func:impl FnMut(PMut<T>)){
        let bots=PMut::new(self.bots);
        rect::naive_for_all_intersect_rect_mut(bots,rect,func);
    }

    pub fn find_collisions_mut(&mut self,mut func:impl FnMut(PMut<T>,PMut<T>)){
        let bots=PMut::new(self.bots);
        colfind::query_naive_mut(bots,|a,b|{
            func(a,b)
        });
    }
    
    pub fn find_collisions_sweep_mut<A:Axis>(&mut self,axis:A,mut func:impl FnMut(PMut<T>,PMut<T>)){
        colfind::query_sweep_mut(axis,self.bots,|a,b|{
            func(a,b)
        });
    }


}

impl<'a,T:HasInner> NaiveAlgs<'a,T>{

    pub fn assert_k_nearest_mut(&mut self,point:Vec2<T::Num>,num:usize,knear:&mut impl k_nearest::Knearest<N=T::Num,T=T>,rect:Rect<T::Num>) where T::Inner: HasId{

        let mut res_naive:Vec<_>={
            let bots=PMut::new(self.bots);
            k_nearest::k_nearest_naive_mut(bots,point,num,knear).drain(..).map(|a|UnitMut2{id:a.bot.inner().get_id(),mag:a.mag}).collect()
        };
        let mut tree=DinoTreeBuilder::new(self.bots).build_seq(); 

        let mut res_dinotree:Vec<_>=k_nearest::k_nearest_mut(&mut tree,point,num,knear,rect).drain(..).map(|a|UnitMut2{id:a.bot.inner().get_id(),mag:a.mag}).collect();

        assert_eq!(res_naive.len(),res_dinotree.len());
        
        let r_naive=util::SliceSplitMut::new(&mut res_naive,|a,b|a.mag==b.mag);
        let r_dinotree=util::SliceSplitMut::new(&mut res_dinotree,|a,b|a.mag==b.mag);

        for (a,b) in r_naive.zip(r_dinotree){
            assert_eq!(a.len(),b.len());
            a.sort_by(|a,b|a.id.cmp(&b.id));
            b.sort_by(|a,b|a.id.cmp(&b.id));

            let res = a.iter().zip(b.iter()).fold(true,|acc,(a,b)|{
                acc & ((a.id==b.id) && (a.mag==b.mag))
            });

            assert!(res);
        }
    }

    pub fn assert_for_all_not_in_rect_mut(&mut self,rect:&axgeom::Rect<T::Num>) where T::Inner:HasId{
        let mut naive_res=Vec::new();
        {
            let bots=PMut::new(self.bots);
            rect::naive_for_all_not_in_rect_mut(bots,rect,|a|naive_res.push(a.inner().get_id()));
        }

        let mut dinotree_res=Vec::new();
        let mut tree=DinoTreeBuilder::new(self.bots).build_seq(); 
        rect::for_all_not_in_rect_mut(&mut tree,rect,|a|dinotree_res.push(a.inner().get_id()));

        assert_eq!(naive_res.len(),dinotree_res.len());
        naive_res.sort();
        dinotree_res.sort();

        let res = naive_res.iter().zip(dinotree_res.iter()).fold(true,|acc,(a,b)|{
            acc & (*a==*b)
        });

        assert!(res);
    }

    pub fn assert_for_all_in_rect_mut(&mut self,rect:&axgeom::Rect<T::Num>) where T::Inner:HasId{
        let mut naive_res=Vec::new();
        {
            let bots=PMut::new(self.bots);
            rect::naive_for_all_in_rect_mut(bots,rect,|a|naive_res.push(a.inner().get_id()));
        }
        let mut dinotree_res=Vec::new();
        let mut tree=DinoTreeBuilder::new(self.bots).build_seq(); 
        rect::for_all_in_rect_mut(&mut tree,rect,|a|dinotree_res.push(a.inner().get_id()));

        assert_eq!(naive_res.len(),dinotree_res.len());
        naive_res.sort();
        dinotree_res.sort();

        let res = naive_res.iter().zip(dinotree_res.iter()).fold(true,|acc,(a,b)|{
            acc & (*a==*b)
        });

        assert!(res);
    }


    pub fn assert_for_all_intersect_rect_mut(&mut self,rect:&axgeom::Rect<T::Num>) where T::Inner:HasId{
        let mut naive_res=Vec::new();
        {
            let bots=PMut::new(self.bots);
            rect::naive_for_all_intersect_rect_mut(bots,rect,|a|naive_res.push(a.inner().get_id()));
        }
        let mut dinotree_res=Vec::new();
        let mut tree=DinoTreeBuilder::new(self.bots).build_seq(); 
        rect::for_all_intersect_rect_mut(&mut tree,rect,|a|dinotree_res.push(a.inner().get_id()));

        assert_eq!(naive_res.len(),dinotree_res.len());
        naive_res.sort();
        dinotree_res.sort();

        let res = naive_res.iter().zip(dinotree_res.iter()).fold(true,|acc,(a,b)|{
            acc & (*a==*b)
        });

        assert!(res);
    }


    pub fn assert_raycast_mut(
        &mut self,
        rect:axgeom::Rect<T::Num>,
        ray:raycast::Ray<T::Num>,
        rtrait:&mut impl raycast::RayCast<N=T::Num,T=T>) where T::Inner: HasId, T::Num:core::fmt::Debug{

        //TODO need to make sure naive also restricts its search to be in just the rect.
        //Otherwise in some cases this function will panic when it shouldnt.


        let res_naive=match raycast::raycast_naive_mut(PMut::new(self.bots),ray,rtrait){
            raycast::RayCastResult::Hit(mut a,b)=>{
                Some( (a.drain(..).map(|a|a.inner().get_id()).collect::<Vec<_>>() ,b) )   
            },
            raycast::RayCastResult::NoHit=>{
                None
            }
        };

        let mut tree=DinoTreeBuilder::new(self.bots).build_seq(); 

        let res_dinotree=match raycast::raycast_mut(&mut tree,rect,ray,rtrait){
            raycast::RayCastResult::Hit(mut a,b)=>{
                Some((a.drain(..).map(|a|a.inner().get_id()).collect::<Vec<_>>() ,b))
            },
            raycast::RayCastResult::NoHit=>{
                None
            }
        };

        match (res_naive,res_dinotree){
            (Some((mut naive_bots,naive_dis)),Some((mut dinotree_bots,dinotree_dis)))=>{
                assert_eq!(naive_dis,dinotree_dis);
                assert_eq!(naive_bots.len(),dinotree_bots.len());
                //let mut naive_bots:Vec<_> = naive_bots.iter().map(|a|a.inner().get_id()).collect();
                //let mut dinotree_bots:Vec<_> = dinotree_bots.iter().map(|a|a.inner().get_id()).collect();
                naive_bots.sort();
                dinotree_bots.sort();

                let res = naive_bots.iter().zip(dinotree_bots.iter()).fold(true,|acc,(a,b)|{
                    acc & (*a==*b)
                });

                assert!(res);
            },
            (None,None)=>{},
            _=>{
                panic!("fail");
            }
        }
        
    }

    pub fn assert_find_collisions_mut(&mut self) where T::Inner: HasId{
        
        let mut naive_pairs=Vec::new();
        colfind::query_naive_mut(PMut::new(self.bots),|a,b|{
            naive_pairs.push(IDPair::new(a.inner().get_id(),b.inner().get_id()));
        });


        let mut tree=DinoTreeBuilder::new(self.bots).build_seq(); 
        
        let mut dinotree_pairs=Vec::new();
        colfind::QueryBuilder::new(&mut tree).query_seq(|a,b| {
            dinotree_pairs.push(IDPair::new(a.inner().get_id(),b.inner().get_id()));
        });


        naive_pairs.sort();
        dinotree_pairs.sort();

        let res = naive_pairs.iter().zip(dinotree_pairs.iter()).fold(true,|acc,(a,b)|{
            acc & (*a==*b)
        });
        assert!(res,"naive={} dinotree={}",naive_pairs.len(),dinotree_pairs.len());
    }

}




///Expose a common Sorter trait so that we may have two version of the tree
///where one implementation actually does sort the tree, while the other one
///does nothing when sort() is called.
pub(crate) trait Sorter: Copy + Clone + Send + Sync {
    fn sort(&self, axis: impl Axis, bots: &mut [impl Aabb]);
}

#[derive(Copy, Clone)]
pub(crate) struct DefaultSorter;

impl Sorter for DefaultSorter {
    fn sort(&self, axis: impl Axis, bots: &mut [impl Aabb]) {
        oned::sweeper_update(axis, bots);
    }
}

#[derive(Copy, Clone)]
pub(crate) struct NoSorter;

impl Sorter for NoSorter {
    fn sort(&self, _axis: impl Axis, _bots: &mut [impl Aabb]) {}
}

pub fn nodes_left(depth: usize, height: usize) -> usize {
    let levels = height - depth;
    2usize.rotate_left(levels as u32) - 1
}

    
///Passed to the binning algorithm to determine
///if the binning algorithm should check for index out of bounds.
#[derive(Copy, Clone, Debug)]
pub enum BinStrat {
    Checked,
    NotChecked,
}

///Returns the height of a dyn tree for a given number of bots.
///The height is chosen such that the nodes will each have a small amount of bots.
///If we had a node per bot, the tree would have too many levels. Too much time would be spent recursing.
///If we had too many bots per node, you would lose the properties of a tree, and end up with plain sweep and prune.
///Theory would tell you to just make a node per bot, but there is
///a sweet spot inbetween determined by the real-word properties of your computer. 
pub const DEFAULT_NUMBER_ELEM_PER_NODE:usize=128;

///Outputs the height given an desirned number of bots per node.
#[inline]
pub fn compute_tree_height_heuristic(num_bots: usize, num_per_node: usize) -> usize {
    //we want each node to have space for around 300 bots.
    //there are 2^h nodes.
    //2^h*200>=num_bots.  Solve for h s.t. h is an integer.

    if num_bots <= num_per_node {
        1
    } else {
        let a=num_bots as f32 / num_per_node as f32;
        let b=a.log2()/2.0;
        (b.ceil() as usize)*2+1
    }
}


///A trait that gives the user callbacks at events in a recursive algorithm on the tree.
///The main motivation behind this trait was to track the time spent taken at each level of the tree
///during construction.
pub trait Splitter: Sized {
    ///Called to split this into two to be passed to the children.
    fn div(&mut self) -> Self;

    ///Called to add the results of the recursive calls on the children.
    fn add(&mut self, b: Self);

    ///Called at the start of the recursive call.
    fn node_start(&mut self);

    ///It is important to note that this gets called in other places besides in the final recursive call of a leaf.
    ///They may get called in a non leaf if its found that there is no more need to recurse further.
    fn node_end(&mut self);
}

///For cases where you don't care about any of the callbacks that Splitter provides, this implements them all to do nothing.
pub struct SplitterEmpty;

impl Splitter for SplitterEmpty {
    #[inline(always)]
    fn div(&mut self) -> Self {
        SplitterEmpty
    }
    #[inline(always)]
    fn add(&mut self, _: Self) {}
    #[inline(always)]
    fn node_start(&mut self) {}
    #[inline(always)]
    fn node_end(&mut self) {}
}



///Returns false if the tree's invariants are not met.
#[must_use]
pub fn assert_invariants<A:Axis,N:Node>(tree:&DinoTree<A,N>)->bool{
    inner(tree.axis(), tree.vistr().with_depth(compt::Depth(0))).is_ok()
}

fn inner<A: Axis, N: Node>(
    axis: A,
    iter: compt::LevelIter<Vistr<N>>,
) -> Result<(), ()> {
    fn a_bot_has_value<N: Num>(it: impl Iterator<Item = N>, val: N) -> bool {
        for b in it {
            if b == val {
                return true;
            }
        }
        false
    }


    macro_rules! assert2 {
        ($bla:expr) => {
            if !$bla {
                return Err(());
            }
        };
    }

    let ((_depth, nn), rest) = iter.next();
    let nn=nn.get();
    let axis_next = axis.next();

    let f = |a: &&N::T, b: &&N::T| -> Option<core::cmp::Ordering> {
        let j=a.get()
            .get_range(axis_next)
            .start
            .cmp(&b.get().get_range(axis_next).start);
        Some(j)
    };

    {
        use is_sorted::IsSorted;
        assert2!(IsSorted::is_sorted_by(&mut nn.bots.iter(),f));
    }

    if let Some([start, end]) = rest {
        match nn.div {
            Some(div) => {
                match nn.cont {
                    Some(cont) => {
                        for bot in nn.bots.iter() {
                            assert2!(bot.get().get_range(axis).contains(*div));
                        }

                        assert2!(a_bot_has_value(
                            nn.bots.iter().map(|b| b.get().get_range(axis).start),
                            *div
                        ));

                        for bot in nn.bots.iter() {
                            assert2!(cont.contains_range(bot.get().get_range(axis)));
                        }

                        assert2!(a_bot_has_value(
                            nn.bots.iter().map(|b| b.get().get_range(axis).start),
                            cont.start
                        ));
                        assert2!(a_bot_has_value(
                            nn.bots.iter().map(|b| b.get().get_range(axis).end),
                            cont.end
                        ));
                    }
                    None => assert2!(nn.bots.is_empty()),
                }

                inner(axis_next, start)?;
                inner(axis_next, end)?;
            }
            None => {
                for (_depth, n) in start.dfs_preorder_iter().chain(end.dfs_preorder_iter()) {
                    let n=n.get();
                    assert2!(n.bots.is_empty());
                    assert2!(n.cont.is_none());
                    assert2!(n.div.is_none());
                }
            }
        }
    }
    Ok(())
}
