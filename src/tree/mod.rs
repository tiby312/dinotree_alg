
use crate::inner_prelude::*;

///Helper module for creating Vecs of different types of BBoxes.
pub mod bbox_helper{
    use crate::inner_prelude::*;

    ///Convenience function to create a `&mut (Rect<N>,T)` from a `(Rect<N>,T)` bounding box.
    pub fn create_bbox_indirect<'a,N:NumTrait,T>(bots:&'a mut [BBox<N,T>])->Vec<BBoxIndirect<'a,BBox<N,T>>>
    {
        bots.iter_mut().map(|a|BBoxIndirect::new(a)).collect()
    }


    ///Convenience function to create a `(Rect<N>,&mut T)` from a `T` and a Rect<N> generating function.
    pub fn create_bbox_mut<'a,Num:NumTrait,T>(bots:&'a mut [T],mut aabb_create:impl FnMut(&T)->Rect<Num>)->Vec<BBoxMut<'a,Num,T>>{
        bots.iter_mut()
            .map(move |k| BBoxMut::new(aabb_create(k),k))
            .collect()
    }    


    ///Helper struct to construct a DinoTree of `(Rect<N>,T)` from a dinotree of `(Rect<N>,&mut T)`
    pub struct IntoDirectHelper<N,T>(Vec<BBox<N,T>>);

    ///Convenience function to create a list of `(Rect<N>,T)` from a `(Rect<N>,&mut T)`. `T` must implement Copy.
    pub fn generate_direct<A:AxisTrait,N:NumTrait,T:Copy>(tree:&DinoTree<A,NodeMut<BBoxMut<N,T>>>)->IntoDirectHelper<N,T>{
        IntoDirectHelper(tree.inner.get_nodes().iter().flat_map(|a|a.range.iter()).map(|a|BBox::new(a.rect,*a.inner)).collect())
    }

    ///Take a DinoTree of `(Rect<N>,&mut T)` and creates a new one of type `(Rect<N>,T)`
    pub fn into_direct<'a,A:AxisTrait,N:NumTrait,T>(tree:&DinoTree<A,NodeMut<BBoxMut<N,T>>>,bots:&'a mut IntoDirectHelper<N,T>)->DinoTree<A,NodeMut<'a,BBox<N,T>>>{
        let mut bots=&mut bots.0 as &'a mut [_];
        let nodes:Vec<_> = tree.inner.get_nodes().iter().map(|node|{
            let mut k:&mut [_]=&mut [];
            core::mem::swap(&mut bots,&mut k);
            let (first,mut rest) = k.split_at_mut(node.range.len());
            core::mem::swap(&mut bots,&mut rest);
            NodeMut{range:first,cont:node.cont,div:node.div}
        }).collect();

        DinoTree{
            inner:compt::dfs_order::CompleteTreeContainer::from_preorder(nodes).unwrap(),
            axis:tree.axis
        }
    }
}



///A version of dinotree that is not lifetimed and uses unsafe{} to own the elements
///that are in its tree (as a self-referential struct). Composed of `(Rect<N>,*mut T)`. 
pub mod dinotree_owned;


pub mod analyze;


pub(crate) use self::notsorted::NotSorted;
mod notsorted{
    use super::*;
    ///A version of dinotree where the elements are not sorted along each axis, like a KD Tree.
    pub struct NotSorted<A: AxisTrait,N:NodeTrait>(pub(crate) DinoTree<A,N>);

    impl<'a,A:AxisTrait,T:HasAabb + Send + Sync> NotSorted<A,NodeMut<'a,T>>{
       #[must_use]
        pub fn new_par(axis:A,bots:&'a mut [T])->NotSorted<A,NodeMut<'a,T>>{
            DinoTreeBuilder::new(axis,bots).build_not_sorted_par()
        }
    }
    impl<'a,A:AxisTrait,T:HasAabb> NotSorted<A,NodeMut<'a,T>>{
        #[must_use]
        pub fn new(axis:A,bots:&'a mut [T])->NotSorted<A,NodeMut<'a,T>>{
            DinoTreeBuilder::new(axis,bots).build_not_sorted_seq()
        }
    }

    impl<A:AxisTrait,N:NodeTrait + Send + Sync> NotSorted<A,N> where N::T : Send + Sync{

        pub fn find_collisions_mut_par(&mut self,func:impl Fn(ProtectedBBox<N::T>,ProtectedBBox<N::T>) + Send + Sync){
            colfind::NotSortedQueryBuilder::new(self).query_par(|a,b|{
                func(a,b)
            });
        }
    }
    impl<A:AxisTrait,N:NodeTrait> NotSorted<A,N>{

        pub fn find_collisions_mut(&mut self,mut func:impl FnMut(ProtectedBBox<N::T>,ProtectedBBox<N::T>)){
            colfind::NotSortedQueryBuilder::new(self).query_seq(|a,b|{
                func(a,b)
            });
        }

        #[inline(always)]
        pub fn axis(&self)->A{
            self.0.axis()
        }

        #[inline(always)]
        pub fn get_height(&self)->usize{
            self.0.get_height()
        }

        #[inline(always)]
        pub fn vistr(&self)->Vistr<N>{
            self.0.inner.vistr()
        }

        #[inline(always)]
        pub fn vistr_mut(&mut self)->VistrMut<N>{
            VistrMut{
                inner:self.0.inner.vistr_mut()
            }
        }

    }
}


use crate::query::*;


///The data structure this crate revoles around.
pub struct DinoTree<A:AxisTrait,N:NodeTrait>{
    axis:A,
    inner: compt::dfs_order::CompleteTreeContainer<N, compt::dfs_order::PreOrder>,
}



impl<'a,A:AxisTrait,T:HasAabb> DinoTree<A,NodeMut<'a,T>>{
    #[must_use]
    pub fn new(axis:A,bots:&'a mut [T])->DinoTree<A,NodeMut<'a,T>>{
        DinoTreeBuilder::new(axis,bots).build_seq()
    }
}

impl<'a,A:AxisTrait,T:HasAabb + Send + Sync> DinoTree<A,NodeMut<'a,T>>{
    #[must_use]
    pub fn new_par(axis:A,bots:&'a mut [T])->DinoTree<A,NodeMut<'a,T>>{
        DinoTreeBuilder::new(axis,bots).build_par()
    }
}

impl<A:AxisTrait,N:NodeTrait + Send + Sync> DinoTree<A,N> where N::T : Send + Sync{
    pub fn find_collisions_mut_par(&mut self,func:impl Fn(ProtectedBBox<N::T>,ProtectedBBox<N::T>) + Send + Sync){
        query::colfind::QueryBuilder::new(self).query_par(|a,b|{
            func(a,b)
        });
    }

    #[cfg(feature = "nbody")]
    pub fn nbody_mut_par<X:query::nbody::NodeMassTrait<Num=N::Num,Item=N::T>+Sync+Send>(
        &mut self,ncontext:&X,rect:Rect<N::Num>) where X::No:Send, N::T:Send+Copy{
        query::nbody::nbody_par(self,ncontext,rect)
    }


    //TODO remove send/sync trait bounds
    #[cfg(feature = "nbody")]
    pub fn nbody_mut<X:query::nbody::NodeMassTrait<Num=N::Num,Item=N::T>+Sync+Send>(
        &mut self,ncontext:&X,rect:Rect<N::Num>) where X::No:Send, N::T:Send+Sync{
        query::nbody::nbody(self,ncontext,rect)
    }

}

impl<A:AxisTrait,N:NodeTrait> DinoTree<A,N>{


    pub fn draw(&self,drawer:&mut impl graphics::DividerDrawer<N=N::Num>,rect:&Rect<N::Num>){
        graphics::draw(self,drawer,rect)
    }
    pub fn intersect_with_mut<X:HasAabb<Num=N::Num>>(
        &mut self,
        other:&mut [X],
        func: impl Fn(ProtectedBBox<N::T>,ProtectedBBox<X>)){
        intersect_with::intersect_with_mut(self,other,func)
    }

    #[must_use]
    pub fn raycast_mut(
        &mut self,
        rect:Rect<N::Num>,
        ray:raycast::Ray<N::Num>,
        rtrait: &mut impl raycast::RayTrait<N=N::Num,T=N::T> )->raycast::RayCastResult<N::T>{
        raycast::raycast_mut(self,rect,ray,rtrait)
    }

    #[must_use]
    pub fn k_nearest_mut(
        &mut self,
        point:Vec2<N::Num>,
        num:usize,
        knear:&mut impl k_nearest::Knearest<N=N::Num,T=N::T>,
        rect:Rect<N::Num>) -> Vec<k_nearest::KnearestResult<N::T>>{
        k_nearest::k_nearest_mut(self,point,num,knear,rect)
    }

    #[must_use]
    pub fn multi_rect(&mut self)->rect::MultiRectMut<A,N>{
        rect::MultiRectMut::new(self)
    }
    pub fn for_all_not_in_rect_mut(&mut self,rect:&Rect<N::Num>,func:impl FnMut(ProtectedBBox<N::T>)){
        rect::for_all_not_in_rect_mut(self,rect,func);
    }
    pub fn for_all_intersect_rect_mut(&mut self,rect:&Rect<N::Num>,func:impl FnMut(ProtectedBBox<N::T>)){
        rect::for_all_intersect_rect_mut(self,rect,func);
    }
    
    pub fn for_all_intersect_rect<'a>(&'a self,rect:&Rect<N::Num>,func:impl FnMut(&'a N::T)){
        rect::for_all_intersect_rect(self,rect,func);
    }
    pub fn for_all_in_rect_mut(&mut self,rect:&Rect<N::Num>,func:impl FnMut(ProtectedBBox<N::T>)){
        rect::for_all_in_rect_mut(self,rect,func);
    }
    pub fn for_all_in_rect(&self,rect:&Rect<N::Num>,func:impl FnMut(&N::T)){
        rect::for_all_in_rect(self,rect,func);
    }

    pub fn find_collisions_mut(&mut self,mut func:impl FnMut(ProtectedBBox<N::T>,ProtectedBBox<N::T>)){
        colfind::QueryBuilder::new(self).query_seq(|a,b|{
            func(a,b)
        });
    }


    #[must_use]
    pub fn axis(&self)->A{
        self.axis
    }

    #[must_use]
    pub fn vistr_mut(&mut self)->VistrMut<N>{
        VistrMut{
            inner:self.inner.vistr_mut()
        }
    }

    #[must_use]
    pub fn vistr(&self)->Vistr<N>{
        self.inner.vistr()
    }

    #[must_use]
    pub fn get_height(&self)->usize{
        self.inner.get_height()
    }

    #[must_use]
    pub fn num_nodes(&self)->usize{
        self.inner.get_nodes().len()
    }
}



use self::builder::DinoTreeBuilder;
mod builder{
    use super::*;

    ///Builder pattern for dinotree.
    pub struct DinoTreeBuilder<'a, A: AxisTrait, T> {
        axis: A,
        bots: &'a mut [T],
        rebal_strat: BinStrat,
        height: usize,
        height_switch_seq: usize,
    }


    impl<'a,A: AxisTrait, T:HasAabb+Send+Sync>
        DinoTreeBuilder<'a,A,  T>
    {
        ///Build not sorted in parallel
        pub fn build_not_sorted_par(&mut self) -> NotSorted<A,NodeMut<'a,T>> {
            let mut bots:&mut [T]=&mut [];
            core::mem::swap(&mut bots,&mut self.bots);
            
            let dlevel = par::compute_level_switch_sequential(self.height_switch_seq, self.height);
            let inner = create_tree_par(self.axis,dlevel, bots, NoSorter, &mut SplitterEmpty, self.height, self.rebal_strat);
            NotSorted(DinoTree{axis:self.axis,inner})
        }

        ///Build in parallel
        pub fn build_par(&mut self) -> DinoTree<A,NodeMut<'a,T>> {
            let mut bots:&mut [T]=&mut [];
            core::mem::swap(&mut bots,&mut self.bots);
            
            let dlevel = par::compute_level_switch_sequential(self.height_switch_seq, self.height);
            let inner = create_tree_par(self.axis,dlevel, bots, DefaultSorter, &mut SplitterEmpty, self.height, self.rebal_strat);
            DinoTree{axis:self.axis,inner}
        }
    }


    impl<'a, A: AxisTrait, T:HasAabb> DinoTreeBuilder<'a,A,T>{

        ///Create a new builder with a slice of elements that implement `HasAabb`.
        pub fn new(axis: A, bots: &'a mut [T]) -> DinoTreeBuilder<'a,A, T> {
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
            let height = compute_tree_height_heuristic(bots.len(),DEFAULT_NUMBER_ELEM_PER_NODE);

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
        pub fn build_not_sorted_seq(&mut self) -> NotSorted<A,NodeMut<'a,T>> {
            let mut bots:&mut [T]=&mut [];
            core::mem::swap(&mut bots,&mut self.bots);
            
            let inner = create_tree_seq(self.axis, bots, NoSorter, &mut SplitterEmpty, self.height, self.rebal_strat);
            NotSorted(DinoTree{axis:self.axis,inner})
        }

        ///Build sequentially
        pub fn build_seq(&mut self)->DinoTree<A,NodeMut<'a,T>>{
            let mut bots:&mut [T]=&mut [];
            core::mem::swap(&mut bots,&mut self.bots);
            let inner = create_tree_seq(self.axis, bots, DefaultSorter, &mut SplitterEmpty, self.height, self.rebal_strat);
            DinoTree{axis:self.axis,inner}
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
        ) -> DinoTree<A,NodeMut<'a,T>> {
            let mut bots:&mut [T]=&mut [];
            core::mem::swap(&mut bots,&mut self.bots);
            
            let inner = create_tree_seq(self.axis, bots, DefaultSorter, splitter, self.height, self.rebal_strat);
            DinoTree{axis:self.axis,inner} 
        }
    }
}



pub(crate) use self::node::*;
///Contains node-level building block structs and visitors used for a DinoTree.
pub mod node{
    use super::*;


    ///When we traverse the tree in read-only mode, we can simply return a reference to each node.
    ///We don't need to protect the user from only mutating parts of the BBox's since they can't
    ///change anything.
    pub type Vistr<'a,N> = compt::dfs_order::Vistr<'a,N,compt::dfs_order::PreOrder>;

    mod vistr_mut{
        use crate::inner_prelude::*;

        //Cannot use since we need create_wrap_mut()
        //We must create our own new type.
        //pub type VistrMut<'a,N> = compt::MapStruct<compt::dfs_order::VistrMut<'a,N,compt::dfs_order::PreOrder>,Foo<'a,N>>;



        /// Tree Iterator that returns a protected mutable reference to each node.
        #[repr(transparent)]
        pub struct VistrMut<'a, N:NodeTrait> {
            pub(crate) inner: compt::dfs_order::VistrMut<'a, N, compt::dfs_order::PreOrder>,
        }

        impl<'a, N:NodeTrait> VistrMut<'a, N> {

            ///It is safe to borrow the iterator and then produce mutable references from that
            ///as long as by the time the borrow ends, all the produced references also go away.
            #[inline(always)]
            pub fn create_wrap_mut(&mut self) -> VistrMut<N> {
                VistrMut {
                    inner: self.inner.create_wrap_mut(),
                }
            }

        }


        impl<'a, N:NodeTrait> core::ops::Deref for VistrMut<'a, N> {
            type Target = Vistr<'a, N>;
            
            #[inline(always)]
            fn deref(&self) -> &Vistr<'a, N> {
                unsafe { &*(self as *const VistrMut<_> as *const Vistr<_>) }
            }
        }



        unsafe impl<'a, N:NodeTrait> compt::FixedDepthVisitor for VistrMut<'a, N> {}

        impl<'a, N:NodeTrait> Visitor for VistrMut<'a, N> {
            type Item = ProtectedNode<'a, N>;

            
            #[inline(always)]
            fn next(self) -> (Self::Item, Option<[Self; 2]>) {
                let (nn, rest) = self.inner.next();

                let k = match rest {
                    Some([left, right]) => Some([VistrMut { inner: left }, VistrMut { inner: right }]),
                    None => None,
                };
                (ProtectedNode::new(nn), k)
            }
            
            #[inline(always)]
            fn level_remaining_hint(&self) -> (usize, Option<usize>) {
                self.inner.level_remaining_hint()
            }


            
            #[inline(always)]
            fn dfs_preorder(self,mut func:impl FnMut(Self::Item)){
                self.inner.dfs_preorder(|a|{
                    func(ProtectedNode::new(a))
                });
            }
        }
    }
    pub use vistr_mut::VistrMut;


    ///Expose a node trait api so that we can have nodes made up of both
    ///&mut [T] and *mut [T].
    ///We ideally want to use the lifetimed version of `NodeMut`, but 
    ///but for `DinoTreeOwned` we must use `NodePtr`.
    pub trait NodeTrait{
        type T:HasAabb<Num=Self::Num>;
        type Num:NumTrait;
        fn get(&self)->NodeRef<Self::T>;
        fn get_mut(&mut self)->NodeRefMut<Self::T>;
    }

    impl<'a,T:HasAabb> NodeTrait for NodeMut<'a,T>{
        type T=T;
        type Num=T::Num;
        fn get(&self)->NodeRef<Self::T>{
            NodeRef{bots:self.range,cont:&self.cont,div:&self.div}
        }
        fn get_mut(&mut self)->NodeRefMut<Self::T>{
            NodeRefMut{bots:ProtectedBBoxSlice::new(self.range),cont:&self.cont,div:&self.div}
        }
    }

    ///A lifetimed node in a dinotree.
    pub struct NodeMut<'a,T: HasAabb> {
        pub range:&'a mut [T],

        //range is empty iff cont is none.
        pub cont: Option<axgeom::Range<T::Num>>,
        //for non leafs:
        //  div is some iff mid is nonempty.
        //  div is none iff mid is empty.
        //for leafs:
        //  div is none
        pub div: Option<T::Num>,
    }



    ///Mutable reference to a node in the dinotree.
    pub struct NodeRefMut<'a, T:HasAabb> {
        ///The bots that belong to this node.
        pub bots: ProtectedBBoxSlice<'a,T>,

        ///Is None iff bots is empty.
        pub cont: &'a Option<axgeom::Range<T::Num>>,

        ///Is None if node is a leaf, or there are no bots in this node or in any decendants.
        pub div: &'a Option<T::Num>,
    }


    ///Reference to a node in the dinotree.
    pub struct NodeRef<'a, T:HasAabb> {
        ///The bots that belong to this node.
        pub bots: &'a [T],

        ///Is None iff bots is empty.
        pub cont: &'a Option<axgeom::Range<T::Num>>,

        ///Is None if node is a leaf, or there are no bots in this node or in any decendants.
        pub div: &'a Option<T::Num>,
    }





}










fn create_tree_seq<'a,A:AxisTrait,T:HasAabb,K:Splitter>(
        div_axis: A,
        rest: &'a mut [T],
        sorter: impl Sorter,
        splitter: &mut K,
        height: usize,
        binstrat: BinStrat,
        ) -> compt::dfs_order::CompleteTreeContainer<NodeMut<'a,T>, compt::dfs_order::PreOrder>{
    let num_bots = rest.len();
    
    let mut nodes = Vec::with_capacity(tree::nodes_left(0, height));

    let r = Recurser {
        height,
        binstrat,
        sorter,
        _p: PhantomData,
    };
    r.recurse_preorder_seq(div_axis, rest, &mut nodes, splitter, 0);

    let tree =
        compt::dfs_order::CompleteTreeContainer::from_preorder(
            nodes
        )
        .unwrap();


    let k = tree
        .get_nodes()
        .iter()
        .fold(0, |acc, a| acc + a.range.len());
    debug_assert_eq!(k, num_bots);

    tree
}

fn create_tree_par<'a,A:AxisTrait,JJ:par::Joiner,T:HasAabb+Send+Sync,K:Splitter+Send+Sync>(
        div_axis: A,
        dlevel: JJ,
        rest: &'a mut [T],
        sorter: impl Sorter,
        splitter: &mut K,
        height: usize,
        binstrat: BinStrat,
        ) -> compt::dfs_order::CompleteTreeContainer<NodeMut<'a,T>, compt::dfs_order::PreOrder>{
    let num_bots = rest.len();
    
    let mut nodes = Vec::with_capacity(tree::nodes_left(0, height));

    let r = Recurser {
        height,
        binstrat,
        sorter,
        _p: PhantomData,
    };
    r.recurse_preorder(div_axis, dlevel, rest, &mut nodes, splitter, 0);

    let tree =
        compt::dfs_order::CompleteTreeContainer::from_preorder(
            nodes
        )
        .unwrap();


    let k = tree
        .get_nodes()
        .iter()
        .fold(0, |acc, a| acc + a.range.len());
    debug_assert_eq!(k, num_bots);

    tree
}

struct Recurser<'a, T: HasAabb, K: Splitter, S: Sorter> {
    height: usize,
    binstrat: BinStrat,
    sorter: S,
    _p :PhantomData<(K,&'a T)>
}


impl<'a, T: HasAabb, K: Splitter , S: Sorter> Recurser<'a, T, K, S> {

    fn create_leaf<A:AxisTrait>(&self,axis:A,rest:&'a mut [T]) -> NodeMut<'a,T>{
        self.sorter.sort(axis.next(),rest);
                
        let cont = create_cont(axis,rest);

        NodeMut {
            range:rest,
            cont,
            div: None,
        }
    }

    fn create_non_leaf<A:AxisTrait>(&self,axis:A,rest:&'a mut [T]) -> (NodeMut<'a,T>,&'a mut [T],&'a mut [T]){
        match construct_non_leaf(self.binstrat, self.sorter, axis, rest) {
            ConstructResult::NonEmpty {
                cont,
                div,
                mid,
                left,
                right,
            } => (
                NodeMut {
                    range: mid,
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
                    range: empty,
                    cont:None,
                    div: None,
                };

                (node,&mut [],&mut [])
            }
        }
    }

    fn recurse_preorder_seq<A:AxisTrait>(
        &self,
        axis: A,
        rest: &'a mut [T],
        nodes: &mut Vec<NodeMut<'a,T>>,
        splitter: &mut K,
        depth: usize,
        )
    {
        splitter.node_start();

        if depth < self.height - 1 {
            let (node, left, right) = self.create_non_leaf(axis,rest); 
            nodes.push(node);

            let mut splitter2 = splitter.div();

            self.recurse_preorder_seq(
                axis.next(),
                left,
                nodes,
                splitter,
                depth + 1,
            );
            self.recurse_preorder_seq(
                axis.next(),
                right,
                nodes,
                &mut splitter2,
                depth + 1,
            );
            
            splitter.add(splitter2);
        } else {
            let node = self.create_leaf(axis,rest);
            nodes.push(node);
            splitter.node_end();
        }
    }
}
impl<'a, T: HasAabb + Send + Sync, K: Splitter + Send+ Sync , S: Sorter> Recurser<'a, T, K, S> {



    fn recurse_preorder<A: AxisTrait, JJ: par::Joiner>(
        &self,
        axis: A,
        dlevel: JJ,
        rest: &'a mut [T],
        nodes: &mut Vec<NodeMut<'a,T>>,
        splitter: &mut K,
        depth: usize,
    ) {
        splitter.node_start();

        if depth < self.height - 1 {
            let (node, left, right) = self.create_non_leaf(axis,rest);
                
            nodes.push(node);

            let mut splitter2 = splitter.div();

            let splitter = match dlevel.next(){
                par::ParResult::Parallel([dleft,dright])=>{
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
                },
                par::ParResult::Sequential(_)=>{
                    
                    //dbg!("SEQ SPLIT");
                    
                    self.recurse_preorder_seq(
                        axis.next(),
                        left,
                        nodes,
                        splitter,
                        depth + 1,
                    );
                    self.recurse_preorder_seq(
                        axis.next(),
                        right,
                        nodes,
                        &mut splitter2,
                        depth + 1,
                    );
                    splitter
                }
            };

            splitter.add(splitter2);
        } else {
            let node = self.create_leaf(axis,rest);
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
        .map(|pos| BBox::new(aabb_create_isize(pos, 5), ()))
        .collect();

    b.iter(|| {
        let k = create_cont(axgeom::XAXISS, &bots);
        let _ = test::black_box(k);
    });
}

#[bench]
#[cfg(all(feature = "unstable", test))]
fn bench_cont2(b: &mut test::Bencher) {
    fn create_cont2<A: AxisTrait, T: HasAabb>(axis: A, middle: &[T]) -> axgeom::Range<T::Num> {
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
        .map(|pos|  BBox::new(aabb_create_isize(pos, 5), ()) )
        .collect();

    b.iter(|| {
        let k = create_cont2(axgeom::XAXISS, &bots);
        let _ = test::black_box(k);
    });
}

fn create_cont<A: AxisTrait, T: HasAabb>(axis: A, middle: &[T]) -> Option<axgeom::Range<T::Num>> {
    match middle.split_first(){
        Some((first,rest))=>{
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
        },
        None=>{
            None
        }
    }
    

}


enum ConstructResult<'a, T: HasAabb> {
    NonEmpty {
        div: T::Num,
        cont: Option<axgeom::Range<T::Num>>,
        mid: &'a mut [T],
        right: &'a mut [T],
        left: &'a mut [T],
    },
    Empty(&'a mut [T]),
}

fn construct_non_leaf<T: HasAabb>(
    bin_strat: BinStrat,
    sorter: impl Sorter,
    div_axis: impl AxisTrait,
    bots: &mut [T],
) -> ConstructResult<T> {
    let med = if bots.is_empty() {
        return ConstructResult::Empty(bots);
    } else {
        let closure = |a: &T, b: &T| -> core::cmp::Ordering { oned::compare_bots(div_axis, a, b) };

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
