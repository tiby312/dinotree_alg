//! Contains code to manipulate the dinotree data structure and some of its query algorithms
//! to help analyze and measure their performance.

use crate::inner_prelude::*;
use crate::query::*;

pub use crate::tree::par;
pub use crate::tree::node;
pub use crate::query::colfind::ColMulti;
pub use crate::query::colfind::NotSortedQueryBuilder;
pub use crate::query::colfind::QueryBuilder;
pub use crate::tree::notsorted::NotSorted;

pub use crate::tree::builder::DinoTreeBuilder;


///Helper module for creating Vecs of different types of BBoxes.
pub mod bbox_helper {
    use crate::inner_prelude::*;

    ///Helper struct to construct a DinoTree of `(Rect<N>,T)` from a dinotree of `(Rect<N>,&mut T)`
    pub struct IntoDirectHelper<N, T>(Vec<BBox<N, T>>);

    ///Convenience function to create a list of `(Rect<N>,T)` from a `(Rect<N>,&mut T)`. `T` must implement Copy.
    pub fn generate_direct<A: Axis, N: Num, T: Copy>(
        tree: &DinoTree<A, BBox<N,&mut T>>,
    ) -> IntoDirectHelper<N, T> {
        IntoDirectHelper(
            tree.inner
                .get_nodes()
                .iter()
                .flat_map(|a| a.range.as_ref().iter())
                .map(move |a| BBox::new(a.rect, *a.inner))
                .collect(),
        )
    }

    ///Take a DinoTree of `(Rect<N>,&mut T)` and creates a new one of type `(Rect<N>,T)`
    pub fn into_direct<'a, A: Axis, N: Num, T>(
        tree: &DinoTree<'a,A, BBox<N,&mut T>>,
        bots: &'a mut IntoDirectHelper<N, T>,
    ) -> DinoTree<'a,A, BBox<N, T>> {
        let mut bots = &mut bots.0 as &'a mut [_];

        let b=PMut::new(bots).as_ptr();
        let nodes: Vec<_> = tree
            .inner
            .get_nodes()
            .iter()
            .map(move |node| {
                let mut k: &mut [_] = &mut [];
                core::mem::swap(&mut bots, &mut k);
                let (first, mut rest) = k.split_at_mut(node.range.len());
                core::mem::swap(&mut bots, &mut rest);
                NodeMut {
                    range: PMut::new(first),
                    cont: node.cont,
                    div: node.div,
                }
            })
            .collect();

        DinoTree {
            inner: compt::dfs_order::CompleteTreeContainer::from_preorder(nodes).unwrap(),
            axis: tree.axis,
            bots:b
        }
    }
}


pub fn find_collisions_sweep_mut<A: Axis,T:Aabb+HasInner>(
    bots: &mut [T] ,
    axis: A,
    mut func: impl FnMut(&mut T::Inner, &mut T::Inner),
) {
    colfind::query_sweep_mut(axis, bots, |a, b| func(a.into_inner(), b.into_inner()));
}


pub struct NaiveAlgs<'a, T> {
    bots: PMut<'a,[T]>,
}




impl<'a, T: Aabb+HasInner> NaiveAlgs<'a, T> {

    #[must_use]
    pub fn raycast_mut<Acc>(
        &mut self,
        ray: axgeom::Ray<T::Num>,
        start: &mut Acc,
        broad:impl FnMut(&mut Acc,&Ray<T::Num>,&Rect<T::Num>)->CastResult<T::Num>,
        fine:impl FnMut(&mut Acc,&Ray<T::Num>,&T)->CastResult<T::Num>,
    ) -> raycast::RayCastResult<T::Inner,T::Num> {
        let mut rtrait=raycast::RayCastClosure{a:start,broad,fine,_p:PhantomData};
        raycast::raycast_naive_mut(self.bots.as_mut(), ray, &mut rtrait)
    }


    #[must_use]
    pub fn k_nearest_mut<Acc>(
        &mut self,
        point: Vec2<T::Num>,
        num: usize,
        start:&mut Acc,
        broad:impl FnMut(&mut Acc,Vec2<T::Num>,&Rect<T::Num>)->T::Num,
        fine:impl FnMut(&mut Acc,Vec2<T::Num>,&T)->T::Num,
    ) -> Vec<k_nearest::KnearestResult<T::Inner,T::Num>> {
        let mut knear=k_nearest::KnearestClosure{acc:start,broad,fine,_p:PhantomData};
        k_nearest::k_nearest_naive_mut(self.bots.as_mut(), point, num, &mut knear)
    }

}

impl<'a, T: Aabb+HasInner> NaiveAlgs<'a, T> {
    pub fn for_all_in_rect_mut(&mut self, rect: &Rect<T::Num>, mut func: impl FnMut(&mut T::Inner)) {
        rect::naive_for_all_in_rect_mut(self.bots.as_mut(), rect, |a|(func)(a.into_inner()));
    }
    pub fn for_all_not_in_rect_mut(&mut self, rect: &Rect<T::Num>, mut func: impl FnMut(&mut T::Inner)) {
        rect::naive_for_all_not_in_rect_mut(self.bots.as_mut(), rect, |a|(func)(a.into_inner()));
    }

    pub fn for_all_intersect_rect_mut(&mut self, rect: &Rect<T::Num>, mut func: impl FnMut(&mut T::Inner)) {
        rect::naive_for_all_intersect_rect_mut(self.bots.as_mut(), rect, |a|(func)(a.into_inner()));
    }

    pub fn find_intersections_mut(&mut self, mut func: impl FnMut(&mut T::Inner, &mut T::Inner)) {
        colfind::query_naive_mut(self.bots.as_mut(), |a, b| func(a.into_inner(), b.into_inner()));
    }

}

impl<'a, T: Aabb> NaiveAlgs<'a, T> {
    #[must_use]
    pub fn new(bots: PMut<[T]>) -> NaiveAlgs<T> {
        NaiveAlgs { bots}
    }


    #[cfg(feature = "nbody")]
    pub fn nbody(&mut self, func: impl FnMut(PMut<T>, PMut<T>)) {
        nbody::naive_mut(self.bots.as_mut(), func);
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
    let levels = height - 1 - depth;
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
pub const DEFAULT_NUMBER_ELEM_PER_NODE: usize = 128;

///Outputs the height given an desirned number of bots per node.
#[inline]
pub fn compute_tree_height_heuristic(num_bots: usize, num_per_node: usize) -> usize {
    //we want each node to have space for around 300 bots.
    //there are 2^h nodes.
    //2^h*200>=num_bots.  Solve for h s.t. h is an integer.

    if num_bots <= num_per_node {
        1
    } else {
        let a = num_bots as f32 / num_per_node as f32;
        let b = a.log2() / 2.0;
        (b.ceil() as usize) * 2 + 1
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
pub fn assert_invariants<A: Axis, T: Aabb>(tree: &DinoTree<A, T>) -> bool {
    inner(tree.axis(), tree.vistr().with_depth(compt::Depth(0))).is_ok()
}

fn inner<A: Axis, N: Node>(axis: A, iter: compt::LevelIter<Vistr<N>>) -> Result<(), ()> {
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
    let nn = nn.get();
    let axis_next = axis.next();

    let f = |a: &&N::T, b: &&N::T| -> Option<core::cmp::Ordering> {
        let j = a
            .get()
            .get_range(axis_next)
            .start
            .cmp(&b.get().get_range(axis_next).start);
        Some(j)
    };

    {
        use is_sorted::IsSorted;
        assert2!(IsSorted::is_sorted_by(&mut nn.bots.iter(), f));
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
                    let n = n.get();
                    assert2!(n.bots.is_empty());
                    assert2!(n.cont.is_none());
                    assert2!(n.div.is_none());
                }
            }
        }
    }
    Ok(())
}
