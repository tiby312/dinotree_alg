
use super::*;

///Builder pattern for dinotree.
///For most usecases, the user is suggested to use
///the built in new() functions to create the tree.
///This is provided in cases the user wants more control
///on the behavior of the tree for benching and debuging purposes.
pub struct DinoTreeBuilder<'a, A: Axis, T> {
    axis: A,
    bots: &'a mut [T],
    rebal_strat: BinStrat,
    height: usize,
    height_switch_seq: usize,
}

impl<'a, A: Axis, T: Aabb + Send + Sync> DinoTreeBuilder<'a, A, T> {
    ///Build not sorted in parallel
    pub fn build_not_sorted_par(&mut self) -> NotSorted<A, NodeMut<'a,T>> {
        let bots=core::mem::replace(&mut self.bots,&mut []);

        let dlevel = par::compute_level_switch_sequential(self.height_switch_seq, self.height);
        let inner = create_tree_par(
            self.axis,
            dlevel,
            bots,
            NoSorter,
            &mut SplitterEmpty,
            self.height,
            self.rebal_strat,
        );
        NotSorted(inner)
    }

    ///Build in parallel
    pub fn build_par(&mut self) -> DinoTree< A,NodeMut<'a, T>> {
        let bots=core::mem::replace(&mut self.bots,&mut []);

        let dlevel = par::compute_level_switch_sequential(self.height_switch_seq, self.height);
        create_tree_par(
            self.axis,
            dlevel,
            bots,
            DefaultSorter,
            &mut SplitterEmpty,
            self.height,
            self.rebal_strat,
        )
    }
}

impl<'a, T: Aabb> DinoTreeBuilder<'a, DefaultA, T> {
    ///Create a new builder with a slice of elements that implement `Aabb`.
    pub fn new(bots: &'a mut [T]) -> DinoTreeBuilder<'a, DefaultA, T> {
        Self::with_axis(default_axis(), bots)
    }
}

impl<'a, A: Axis, T: Aabb> DinoTreeBuilder<'a, A, T> {
    ///Create a new builder with a slice of elements that implement `Aabb`.
    pub fn with_axis(axis: A, bots: &'a mut [T]) -> DinoTreeBuilder<'a, A, T> {
        let rebal_strat = BinStrat::NotChecked;

        //we want each node to have space for around num_per_node bots.
        //there are 2^h nodes.
        //2^h*200>=num_bots.  Solve for h s.t. h is an integer.

        //Make this number too small, and the tree will have too many levels,
        //and too much time will be spent recursing.
        //Make this number too high, and you will lose the properties of a tree,
        //and you will end up with just sweep and prune.
        //This number was chosen emprically from running the dinotree_alg_data project,
        //on two different machines.
        let height = compute_tree_height_heuristic(bots.len(), DEFAULT_NUMBER_ELEM_PER_NODE);

        let height_switch_seq = par::SWITCH_SEQUENTIAL_DEFAULT;

        DinoTreeBuilder {
            axis,
            bots,
            rebal_strat,
            height,
            height_switch_seq,
        }
    }

    ///Build not sorted sequentially
    pub fn build_not_sorted_seq(&mut self) -> NotSorted<A, NodeMut<'a,T>> {
        let bots=core::mem::replace(&mut self.bots,&mut []);

        let inner = create_tree_seq(
            self.axis,
            bots,
            NoSorter,
            &mut SplitterEmpty,
            self.height,
            self.rebal_strat,
        );
        NotSorted(inner)
    }

    ///Build sequentially
    pub fn build_seq(&mut self) -> DinoTree< A, NodeMut<'a,T>> {
        let bots=core::mem::replace(&mut self.bots,&mut []);

        create_tree_seq(
            self.axis,
            bots,
            DefaultSorter,
            &mut SplitterEmpty,
            self.height,
            self.rebal_strat,
        )
    }

    #[inline(always)]
    pub fn with_bin_strat(&mut self, strat: BinStrat) -> &mut Self {
        self.rebal_strat = strat;
        self
    }

    #[inline(always)]
    pub fn with_height(&mut self, height: usize) -> &mut Self {
        self.height = height;
        self
        //TODO test corner cases of this
    }

    ///Choose the height at which to switch from parallel to sequential.
    ///If you end up building sequentially, this argument is ignored.
    #[inline(always)]
    pub fn with_height_switch_seq(&mut self, height: usize) -> &mut Self {
        self.height_switch_seq = height;
        self
    }

    ///Build with a Splitter.
    pub fn build_with_splitter_seq<S: Splitter>(
        &mut self,
        splitter: &mut S,
    ) -> DinoTree< A, NodeMut<'a,T>> {
        let bots=core::mem::replace(&mut self.bots,&mut []);

        create_tree_seq(
            self.axis,
            bots,
            DefaultSorter,
            splitter,
            self.height,
            self.rebal_strat,
        )
    }
}