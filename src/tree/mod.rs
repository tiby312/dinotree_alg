use crate::inner_prelude::*;

#[cfg(test)]
mod tests;

///A version of dinotree that is not lifetimed and uses unsafe{} to own the elements
///that are in its tree (as a self-referential struct). Composed of `(Rect<N>,*mut T)`.
pub mod owned;



pub mod analyze;

///Contains code to write generic code that can be run in parallel, or sequentially. The api is exposed
///in case users find it useful when writing parallel query code to operate on the tree.
pub mod par;

mod notsorted;
pub(crate) use self::notsorted::NotSorted;


use crate::query::*;


pub struct DinoTreeIndPtr<A:Axis,N:Num,T>{
    inner:owned::DinoTreeOwned<A,BBox<N,*mut T>>,
    orig:*mut [T]
}
impl<A:Axis,N:Num,T> DinoTreeIndPtr<A,N,T>{
    fn with_axis<'a>(arr:&'a mut [T],func:impl FnMut(&mut T)->Rect<N>)->DinoTreeIndPtr<A,N,T>{
        unimplemented!();
    }
    fn get_elements_mut(&mut self)->&mut [T]{
        unimplemented!();
    }
    fn as_tree<'a>(&'a mut self,arr:&'a mut [T])->DinoTree<A,NodeMut<'a,BBox<N,&mut T>>>{
        unimplemented!();
    }
}


pub struct DinoTreePtr<'a,A:Axis,T:Aabb>{
    inner:DinoTree<A,owned::NodePtr<T>>,
    orig:*const [T],
    _p:PhantomData<&'a mut T>
}

impl<'a,A:Axis,T:Aabb> DinoTreePtr<'a,A,T>{
    fn with_axis(a:A,arr:&'a mut [T])->DinoTreePtr<'a,A,T>{
        unimplemented!();
    }
    fn get_elements_mut(&mut self)->PMut<'a,[T]>{
        unimplemented!();
    }
    fn as_tree(&'a mut self,arr:PMut<'a,[T]>)->DinoTree<A,NodeMut<'a,T>>{
        unimplemented!();
    }
}


///The data structure this crate revoles around.
pub struct DinoTree<A: Axis, N:Node> {
    axis: A,
    inner: compt::dfs_order::CompleteTreeContainer<N, compt::dfs_order::PreOrder>
}

///The type of the axis of the first node in the dinotree.
///If it is the y axis, then the first divider will be a horizontal line,
///since it is partioning space based off of objects y value.
pub type DefaultA = YAXIS;
///Constructor of the default axis type. Needed since you cannot construct from type alias's.
pub const fn default_axis() -> YAXIS {
    YAXIS
}

impl<'a, T: Aabb> DinoTree<DefaultA,NodeMut<'a, T>> {
    /// # Examples
    ///
    ///```
    ///let mut bots = [axgeom::rect(0,10,0,10)];
    ///let tree = dinotree_alg::DinoTree::new(&mut bots);
    ///
    ///```
    pub fn new(bots: &'a mut [T]) -> DinoTree< DefaultA, NodeMut<'a, T>> {
        DinoTreeBuilder::new(bots).build_seq()
    }
}

impl<'a, T: Aabb + Send + Sync> DinoTree<DefaultA, NodeMut<'a, T>> {
    /// # Examples
    ///
    ///```
    ///let mut bots = [axgeom::rect(0,10,0,10)];
    ///let tree = dinotree_alg::DinoTree::new_par(&mut bots);
    ///
    ///```
    pub fn new_par(bots: &'a mut [T]) -> DinoTree<DefaultA, NodeMut<'a, T>> {
        DinoTreeBuilder::new(bots).build_par()
    }
}

impl<'a, A: Axis, T: Aabb> DinoTree<A, NodeMut<'a, T>> {
    /// # Examples
    ///
    ///```
    ///let mut bots = [axgeom::rect(0,10,0,10)];
    ///let tree = dinotree_alg::DinoTree::with_axis(axgeom::XAXIS,&mut bots);
    ///
    ///```
    pub fn with_axis(axis: A, bots: &'a mut [T]) -> DinoTree<A, NodeMut<'a, T>> {
        DinoTreeBuilder::with_axis(axis, bots).build_seq()
    }

}



impl<'a, A: Axis, T: Aabb + Send + Sync> DinoTree< A, NodeMut<'a, T>> {
    /// # Examples
    ///
    ///```
    ///let mut bots = [axgeom::rect(0,10,0,10)];
    ///let tree = dinotree_alg::DinoTree::with_axis(axgeom::XAXIS,&mut bots);
    ///
    ///```
    pub fn with_axis_par(axis: A, bots: &'a mut [T]) -> DinoTree< A, NodeMut<'a,T>> {
        DinoTreeBuilder::with_axis(axis, bots).build_par()
    }
}


impl<A:Axis,N:Node> Queries for DinoTree<A,N>{
    type A=A;
    type N=N;
    type T=N::T;
    type Num=N::Num;
    
    #[inline(always)]
    fn axis(&self)->Self::A{
        self.axis
    }

    #[inline(always)]
    fn vistr_mut(&mut self)->VistrMut<N>{
        VistrMut{inner:self.inner.vistr_mut()}
    }

    #[inline(always)]
    fn vistr(&self)->Vistr<N>{
        self.inner.vistr()
    }
}



pub struct IntersectionList<'a, T, D> {
    ///See collect_intersections_list()
    ///The same elements can be part of
    ///multiple intersecting pairs.
    ///So pointer aliasing rules are not
    ///being met if we were to just use this
    ///vec according to its type signature.
    cols: Vec<(*mut T, *mut T, D)>,
    _p:PhantomData<&'a mut T>
}
impl<'a,T,D> IntersectionList<'a,T,D>{
    pub fn for_every_pair_mut<'b, A: Axis, N: Num>(
        &'b mut self,
        mut func: impl FnMut(&mut T, &mut T, &mut D),
    ) {
        for (a, b, d) in self.cols.iter_mut() {
            func(unsafe{&mut **a}, unsafe{&mut **b}, d)
        }
    }
}


impl<'a,'b,A:Axis,N:Num,T> DinoTree<A,NodeMut<'a, BBox<N,&'b mut T>>>{
    

    pub fn collect_intersections_list<'c,D: Send + Sync>(
        &mut self,
        mut func: impl FnMut(&mut T, &mut T) -> Option<D> + Send + Sync,
    ) -> IntersectionList<'b, T, D> {
        let mut cols: Vec<_> = Vec::new();
    
        self.find_intersections_mut(|a, b| {
            if let Some(d) = func(a, b) {
                //We use unsafe to collect mutable references of
                //all colliding pairs.
                //This is safe to do because the user is forced
                //to iterate through all the colliding pairs
                //one at a time.
                let a=*a as *mut T;
                let b=*b as *mut T;
                
                cols.push((a,b,d));
            }
        });

        IntersectionList {
            cols,
            _p:PhantomData
        }
    }
}



impl< A: Axis, N:Node> DinoTree< A, N> {

    /// # Examples
    ///
    ///```
    ///use dinotree_alg::*;
    ///let mut bots = vec![axgeom::rect(0,10,0,10);400];
    ///let mut tree = DinoTree::new(&mut bots);
    ///
    ///assert_eq!(tree.get_height(),analyze::compute_tree_height_heuristic(400,analyze::DEFAULT_NUMBER_ELEM_PER_NODE));
    ///```
    ///
    #[must_use]
    #[inline(always)]
    pub fn get_height(&self) -> usize {
        self.inner.get_height()
    }

    

    /// # Examples
    ///
    ///```
    ///use dinotree_alg::*;
    ///let mut bots = vec![axgeom::rect(0,10,0,10);400];
    ///let mut tree = DinoTree::new(&mut bots);
    ///
    ///assert_eq!(tree.num_nodes(),analyze::nodes_left(0,tree.get_height() ));
    ///
    ///```
    #[must_use]
    #[inline(always)]
    pub fn num_nodes(&self) -> usize {
        self.inner.get_nodes().len()
    }
}


pub use self::builder::DinoTreeBuilder;
mod builder;

pub(crate) use self::node::*;
///Contains node-level building block structs and visitors used for a DinoTree.
pub mod node {
    use super::*;

    ///When we traverse the tree in read-only mode, we can simply return a reference to each node.
    ///We don't need to protect the user from only mutating parts of the BBox's since they can't
    ///change anything.
    pub type Vistr<'a, N> = compt::dfs_order::Vistr<'a, N, compt::dfs_order::PreOrder>;
    
    mod vistr_mut {
        use crate::inner_prelude::*;

        //Cannot use since we need create_wrap_mut()
        //We must create our own new type.
        //pub type VistrMut<'a,N> = compt::MapStruct<compt::dfs_order::VistrMut<'a,N,compt::dfs_order::PreOrder>,Foo<'a,N>>;

        /// Tree Iterator that returns a protected mutable reference to each node.
        #[repr(transparent)]
        pub struct VistrMut<'a, N> {
            pub(crate) inner: compt::dfs_order::VistrMut<'a, N, compt::dfs_order::PreOrder>,
        }

        impl<'a, N> VistrMut<'a, N> {
            ///It is safe to borrow the iterator and then produce mutable references from that
            ///as long as by the time the borrow ends, all the produced references also go away.
            #[inline(always)]
            pub fn create_wrap_mut(&mut self) -> VistrMut<N> {
                VistrMut {
                    inner: self.inner.create_wrap_mut(),
                }
            }

            #[inline(always)]
            pub fn as_slice_mut(&mut self) -> PMut<[N]> {
                PMut::new(self.inner.as_slice_mut())
            }

            
            #[inline(always)]
            pub fn into_slice(self) -> PMut<'a,[N]> {
                PMut::new(self.inner.into_slice())
            }
        }

        impl<'a, N> core::ops::Deref for VistrMut<'a, N> {
            type Target = Vistr<'a, N>;

            #[inline(always)]
            fn deref(&self) -> &Vistr<'a, N> {
                unsafe { &*(self as *const VistrMut<_> as *const Vistr<_>) }
            }
        }

        unsafe impl<'a, N> compt::FixedDepthVisitor for VistrMut<'a, N> {}

        impl<'a, N> Visitor for VistrMut<'a, N> {
            type Item = PMut<'a, N>;

            #[inline(always)]
            fn next(self) -> (Self::Item, Option<[Self; 2]>) {
                let (nn, rest) = self.inner.next();

                let k = match rest {
                    Some([left, right]) => {
                        Some([VistrMut { inner: left }, VistrMut { inner: right }])
                    }
                    None => None,
                };
                (PMut::new(nn), k)
            }

            #[inline(always)]
            fn level_remaining_hint(&self) -> (usize, Option<usize>) {
                self.inner.level_remaining_hint()
            }

            #[inline(always)]
            fn dfs_preorder(self, mut func: impl FnMut(Self::Item)) {
                self.inner.dfs_preorder(move |a| func(PMut::new(a)));
            }
        }
    }
    pub use vistr_mut::VistrMut;
    

    ///Expose a node trait api to hide the lifetime of NodeMut.
    ///This way query algorithms do not need to worry about this lifetime.
    pub trait Node {
        type T: Aabb<Num = Self::Num>;
        type Num: Num;
        fn get(&self) -> NodeRef<Self::T>;
        fn get_mut(&mut self) -> NodeRefMut<Self::T>;
    }

    impl<'a, T: Aabb> Node for NodeMut<'a, T> {
        type T = T;
        type Num = T::Num;
        fn get(&self) -> NodeRef<Self::T> {
            //TODO point as struct impl
            NodeRef {
                bots: self.range.as_ref(),
                cont: &self.cont,
                div: &self.div,
            }
        }
        fn get_mut(&mut self) -> NodeRefMut<Self::T> {
            NodeRefMut {
                bots: self.range.as_mut(),
                cont: &self.cont,
                div: &self.div,
            }
        }
    }

    ///A lifetimed node in a dinotree.
    pub struct NodeMut<'a, T: Aabb> {
        pub(crate) range: PMut<'a, [T]>,

        //range is empty iff cont is none.
        pub(crate) cont: Option<axgeom::Range<T::Num>>,
        //for non leafs:
        //  div is some iff mid is nonempty.
        //  div is none iff mid is empty.
        //for leafs:
        //  div is none
        pub(crate) div: Option<T::Num>,
    }

    impl<'a, T: Aabb> NodeMut<'a, T> {
        pub fn get(&self) -> NodeRef<T> {
            NodeRef {
                bots: self.range.as_ref(),
                cont: &self.cont,
                div: &self.div,
            }
        }
        pub fn get_mut(&mut self) -> NodeRefMut<T> {
            NodeRefMut {
                bots: self.range.as_mut(),
                cont: &self.cont,
                div: &self.div,
            }
        }
    }

    ///Mutable reference to a node in the dinotree.
    pub struct NodeRefMut<'a, T: Aabb> {
        ///The bots that belong to this node.
        pub bots: PMut<'a, [T]>,

        ///Is None iff bots is empty.
        pub cont: &'a Option<axgeom::Range<T::Num>>,

        ///Is None if node is a leaf, or there are no bots in this node or in any decendants.
        pub div: &'a Option<T::Num>,
    }

    ///Reference to a node in the dinotree.
    pub struct NodeRef<'a, T: Aabb> {
        ///The bots that belong to this node.
        pub bots: &'a [T],

        ///Is None iff bots is empty.
        pub cont: &'a Option<axgeom::Range<T::Num>>,

        ///Is None if node is a leaf, or there are no bots in this node or in any decendants.
        pub div: &'a Option<T::Num>,
    }
}

fn create_tree_seq<'a, A: Axis, T: Aabb, K: Splitter>(
    div_axis: A,
    rest: &'a mut [T],
    sorter: impl Sorter,
    splitter: &mut K,
    height: usize,
    binstrat: BinStrat,
) -> DinoTree<A,NodeMut<'a,T>> {

    let num_bots = rest.len();

    let mut nodes = Vec::with_capacity(tree::nodes_left(0, height));

    let r = Recurser {
        height,
        binstrat,
        sorter,
        _p: PhantomData,
    };
    r.recurse_preorder_seq(div_axis, rest, &mut nodes, splitter, 0);

    let tree = compt::dfs_order::CompleteTreeContainer::from_preorder(nodes).unwrap();

    let k = tree
        .get_nodes()
        .iter()
        .fold(0, move |acc, a| acc + a.range.len());
    debug_assert_eq!(k, num_bots);

    DinoTree{axis:div_axis,inner:tree}
}

fn create_tree_par<
    'a,
    A: Axis,
    JJ: par::Joiner,
    T: Aabb + Send + Sync,
    K: Splitter + Send + Sync,
>(
    div_axis: A,
    dlevel: JJ,
    rest: &'a mut [T],
    sorter: impl Sorter,
    splitter: &mut K,
    height: usize,
    binstrat: BinStrat,
) ->DinoTree<A,NodeMut<'a,T>> {

    let num_bots = rest.len();

    let mut nodes = Vec::with_capacity(tree::nodes_left(0, height));

    let r = Recurser {
        height,
        binstrat,
        sorter,
        _p: PhantomData,
    };
    r.recurse_preorder(div_axis, dlevel, rest, &mut nodes, splitter, 0);

    let tree = compt::dfs_order::CompleteTreeContainer::from_preorder(nodes).unwrap();

    let k = tree
        .get_nodes()
        .iter()
        .fold(0, move |acc, a| acc + a.range.len());
    debug_assert_eq!(k, num_bots);

    DinoTree{
        axis:div_axis,
        inner:tree
    }
}

struct Recurser<'a, T: Aabb, K: Splitter, S: Sorter> {
    height: usize,
    binstrat: BinStrat,
    sorter: S,
    _p: PhantomData<(K, &'a T)>,
}

impl<'a, T: Aabb, K: Splitter, S: Sorter> Recurser<'a, T, K, S> {
    fn create_leaf<A: Axis>(&self, axis: A, rest: &'a mut [T]) -> NodeMut<'a, T> {
        self.sorter.sort(axis.next(), rest);

        let cont = create_cont(axis, rest);

        NodeMut {
            range: PMut::new(rest),
            cont,
            div: None,
        }
    }

    fn create_non_leaf<A: Axis>(
        &self,
        axis: A,
        rest: &'a mut [T],
    ) -> (NodeMut<'a, T>, &'a mut [T], &'a mut [T]) {
        match construct_non_leaf(self.binstrat, self.sorter, axis, rest) {
            ConstructResult::NonEmpty {
                cont,
                div,
                mid,
                left,
                right,
            } => (
                NodeMut {
                    range: PMut::new(mid),
                    cont,
                    div: Some(div),
                },
                left,
                right,
            ),
            ConstructResult::Empty(empty) => {
                //let (a,empty) = tools::duplicate_empty_slice(empty);
                //let (b,c) = tools::duplicate_empty_slice(empty);
                let node = NodeMut {
                    range: PMut::new(empty),
                    cont: None,
                    div: None,
                };

                (node, &mut [], &mut [])
            }
        }
    }

    fn recurse_preorder_seq<A: Axis>(
        &self,
        axis: A,
        rest: &'a mut [T],
        nodes: &mut Vec<NodeMut<'a, T>>,
        splitter: &mut K,
        depth: usize,
    ) {
        splitter.node_start();

        if depth < self.height - 1 {
            let (node, left, right) = self.create_non_leaf(axis, rest);
            nodes.push(node);

            let mut splitter2 = splitter.div();

            self.recurse_preorder_seq(axis.next(), left, nodes, splitter, depth + 1);
            self.recurse_preorder_seq(axis.next(), right, nodes, &mut splitter2, depth + 1);

            splitter.add(splitter2);
        } else {
            let node = self.create_leaf(axis, rest);
            nodes.push(node);
            splitter.node_end();
        }
    }
}
impl<'a, T: Aabb + Send + Sync, K: Splitter + Send + Sync, S: Sorter> Recurser<'a, T, K, S> {
    fn recurse_preorder<A: Axis, JJ: par::Joiner>(
        &self,
        axis: A,
        dlevel: JJ,
        rest: &'a mut [T],
        nodes: &mut Vec<NodeMut<'a, T>>,
        splitter: &mut K,
        depth: usize,
    ) {
        splitter.node_start();

        if depth < self.height - 1 {
            let (node, left, right) = self.create_non_leaf(axis, rest);

            nodes.push(node);

            let mut splitter2 = splitter.div();

            let splitter = match dlevel.next() {
                par::ParResult::Parallel([dleft, dright]) => {
                    let splitter2 = &mut splitter2;

                    //dbg!("PAR SPLIT");

                    let ((splitter, nodes), mut nodes2) = rayon::join(
                        move || {
                            self.recurse_preorder(
                                axis.next(),
                                dleft,
                                left,
                                nodes,
                                splitter,
                                depth + 1,
                            );
                            (splitter, nodes)
                        },
                        move || {
                            let mut nodes2: Vec<_> =
                                Vec::with_capacity(nodes_left(depth, self.height));
                            self.recurse_preorder(
                                axis.next(),
                                dright,
                                right,
                                &mut nodes2,
                                splitter2,
                                depth + 1,
                            );
                            nodes2
                        },
                    );

                    nodes.append(&mut nodes2);
                    splitter
                }
                par::ParResult::Sequential(_) => {
                    //dbg!("SEQ SPLIT");

                    self.recurse_preorder_seq(axis.next(), left, nodes, splitter, depth + 1);
                    self.recurse_preorder_seq(axis.next(), right, nodes, &mut splitter2, depth + 1);
                    splitter
                }
            };

            splitter.add(splitter2);
        } else {
            let node = self.create_leaf(axis, rest);
            nodes.push(node);
            splitter.node_end();
        }
    }
}

#[bench]
#[cfg(all(feature = "unstable", test))]
fn bench_cont(b: &mut test::Bencher) {
    let grow = 2.0;
    let s = dists::spiral::Spiral::new([400.0, 400.0], 17.0, grow);

    fn aabb_create_isize(pos: [isize; 2], radius: isize) -> axgeom::Rect<isize> {
        axgeom::Rect::new(
            pos[0] - radius,
            pos[0] + radius,
            pos[1] - radius,
            pos[1] + radius,
        )
    }
    let bots: Vec<_> = s
        .as_isize()
        .take(100_000)
        .map(move |pos| BBox::new(aabb_create_isize(pos, 5), ()))
        .collect();

    b.iter(|| {
        let k = create_cont(axgeom::XAXISS, &bots);
        let _ = test::black_box(k);
    });
}

#[bench]
#[cfg(all(feature = "unstable", test))]
fn bench_cont2(b: &mut test::Bencher) {
    fn create_cont2<A: Axis, T: Aabb>(axis: A, middle: &[T]) -> axgeom::Range<T::Num> {
        let left = middle
            .iter()
            .map(|a| a.get().get_range(axis).left)
            .min()
            .unwrap();
        let right = middle
            .iter()
            .map(|a| a.get().get_range(axis).right)
            .max()
            .unwrap();
        axgeom::Range { left, right }
    }

    let grow = 2.0;
    let s = dists::spiral::Spiral::new([400.0, 400.0], 17.0, grow);

    fn aabb_create_isize(pos: [isize; 2], radius: isize) -> axgeom::Rect<isize> {
        axgeom::Rect::new(
            pos[0] - radius,
            pos[0] + radius,
            pos[1] - radius,
            pos[1] + radius,
        )
    }
    let bots: Vec<_> = s
        .as_isize()
        .take(100_000)
        .map(|pos| BBox::new(aabb_create_isize(pos, 5), ()))
        .collect();

    b.iter(|| {
        let k = create_cont2(axgeom::XAXISS, &bots);
        let _ = test::black_box(k);
    });
}

fn create_cont<A: Axis, T: Aabb>(axis: A, middle: &[T]) -> Option<axgeom::Range<T::Num>> {
    match middle.split_first() {
        Some((first, rest)) => {
            let mut min = first.get().get_range(axis).start;
            let mut max = first.get().get_range(axis).end;

            for a in rest.iter() {
                let start = &a.get().get_range(axis).start;
                let end = &a.get().get_range(axis).end;

                if *start < min {
                    min = *start;
                }

                if *end > max {
                    max = *end;
                }
            }

            Some(axgeom::Range {
                start: min,
                end: max,
            })
        }
        None => None,
    }
}

enum ConstructResult<'a, T: Aabb> {
    NonEmpty {
        div: T::Num,
        cont: Option<axgeom::Range<T::Num>>,
        mid: &'a mut [T],
        right: &'a mut [T],
        left: &'a mut [T],
    },
    Empty(&'a mut [T]),
}

fn construct_non_leaf<T: Aabb>(
    bin_strat: BinStrat,
    sorter: impl Sorter,
    div_axis: impl Axis,
    bots: &mut [T],
) -> ConstructResult<T> {
    let med = if bots.is_empty() {
        return ConstructResult::Empty(bots);
    } else {
        let closure =
            move |a: &T, b: &T| -> core::cmp::Ordering { oned::compare_bots(div_axis, a, b) };

        let k = {
            let mm = bots.len() / 2;
            pdqselect::select_by(bots, mm, closure);
            &bots[mm]
        };

        k.get().get_range(div_axis).start
    };

    //TODO. its possible that middle is empty is the ranges inserted had
    //zero length.
    //It is very important that the median bot end up be binned into the middile bin.
    //We know this must be true because we chose the divider to be the medians left border,
    //and we binned so that all bots who intersect with the divider end up in the middle bin.
    //Very important that if a bots border is exactly on the divider, it is put in the middle.
    //If this were not true, there is no guarentee that the middile bin has bots in it even
    //though we did pick a divider.
    let binned = match bin_strat {
        BinStrat::Checked => oned::bin_middle_left_right(div_axis, &med, bots),
        BinStrat::NotChecked => unsafe {
            oned::bin_middle_left_right_unchecked(div_axis, &med, bots)
        },
    };

    //debug_assert!(!binned.middle.is_empty());
    sorter.sort(div_axis.next(), binned.middle);

    let cont = create_cont(div_axis, binned.middle);

    //We already know that the middile is non zero in length.

    ConstructResult::NonEmpty {
        mid: binned.middle,
        cont,
        div: med,
        left: binned.left,
        right: binned.right,
    }
}
