//! # Examples
//!
//!```
//!
//!use dinotree_alg::*;
//!use axgeom::{vec2,Rect,Vec2};
//!
//!struct Bot{
//!    pos:Vec2<isize>,
//!    inner:isize
//!}
//!
//!let radius=3;
//!let mut bots = [Bot{pos:vec2(4isize,8),inner:0isize},Bot{pos:vec2(4isize,6),inner:0}];
//!
//!//Create a collectable dinotree.
//!let mut tree = collectable::CollectableDinoTree::new(&mut bots,|r|{
//!     Rect::from_point(r.pos,vec2(radius,radius))
//!});
//!
//!//Collect all intersections so we can iterate through them later
//!let mut intersections=tree.collect_intersections_list(|a,b|{
//!     a.inner+=1;
//!     b.inner+=2;
//!     Some(1)
//!});
//!
//!//We can perform other queries while we have the intersections stored from earlier
//!tree.get_mut().for_all_in_rect_mut(&axgeom::rect(0,10,0,10),|a|{
//!    a.inner+=10;
//!});
//!
//!
//!for _ in 0..3{
//!     //Query all other intersections
//!     intersections.for_every_pair_mut(&mut tree,|a,b,d|{
//!         a.inner+=*d;
//!         b.inner+=*d;
//!     });
//!}
//!
//!assert_eq!(bots[0].inner,4);
//!assert_eq!(bots[1].inner,15);
//!
//!```

use super::*;
use owned::myptr;
use owned::MyPtr;
use owned::*;

pub struct CollectableDinoTree<'a, A: Axis, N: Num, T> {
    bots: &'a mut [T],
    tree: DinoTreeOwned<A, BBox<N, MyPtr<T>>>,
}
impl<'a, N: Num, T> CollectableDinoTree<'a, DefaultA, N, T> {
    pub fn new(
        bots: &'a mut [T],
        mut func: impl FnMut(&mut T) -> Rect<N>,
    ) -> CollectableDinoTree<'a, DefaultA, N, T> {
        let bboxes: Vec<_> = bots
            .iter_mut()
            .map(|a| BBox::new(func(a), myptr(a)))
            .collect();

        let tree = DinoTreeOwned::new(bboxes);

        CollectableDinoTree { bots, tree }
    }
}
impl<'a, A: Axis, N: Num, T> CollectableDinoTree<'a, A, N, T> {
    pub fn get_bots(&self) -> &[T] {
        self.bots
    }
    pub fn get_bots_mut(&mut self) -> &mut [T] {
        self.bots
    }

    pub fn get_mut(&mut self) -> &mut DinoTree<A, BBox<N, &mut T>> {
        let k = self.tree.as_tree_mut() as *mut _;
        let j = k as *mut DinoTree<A, BBox<N, &mut T>>;
        unsafe { &mut *j }
    }

    pub fn collect_all<D>(
        &mut self,
        mut func: impl FnMut(&Rect<N>, &mut T) -> Option<D>,
    ) -> SingleCollisionList<'a, T, D> {
        let tree = self.tree.as_tree_mut();
        let mut res = Vec::new();
        for node in tree.inner.get_nodes_mut().iter_mut() {
            for b in node.get_mut().bots.iter_mut() {
                let (x, y) = b.unpack();
                if let Some(d) = func(x, unsafe { y.as_mut() }) {
                    res.push((*y, d));
                }
            }
        }
        SingleCollisionList {
            _p: PhantomData,
            a: res,
            orig: myptr(self.get_bots_mut()),
        }
    }

    pub fn collect_intersections_list<D: Send + Sync>(
        &mut self,
        func: impl Fn(&mut T, &mut T) -> Option<D> + Send + Sync + Copy,
    ) -> BotCollision<'a, T, D> {
        
        let cols = create_collision_list(self.tree.as_tree_mut(), |a, b| {
            match func(unsafe { a.as_mut() }, unsafe { b.as_mut() }) {
                Some(d) => Some((*a, *b, d)),
                None => None,
            }
        });
        BotCollision {
            cols,
            _p: PhantomData,
            orig: myptr(self.get_bots_mut()),
        }
    }
}

impl<'a, A: Axis + Send + Sync, N: Num + Send + Sync, T: Send + Sync>
    CollectableDinoTree<'a, A, N, T>
{
    pub fn collect_all_par<D: Send + Sync>(
        &mut self,
        func: impl Fn(&Rect<N>, &mut T) -> Option<D> + Send + Sync + Copy,
    ) -> SingleCollisionList<'a, T, D> {
        let tree = self.tree.as_tree_mut();
        use rayon::prelude::*;

        let par = tree
            .inner
            .get_nodes_mut()
            .par_iter_mut()
            .map(|node| {
                let mut a = Vec::new();
                for b in node.get_mut().bots.iter_mut() {
                    let (x, y) = b.unpack();
                    if let Some(d) = func(x, unsafe { y.as_mut() }) {
                        a.push((*y, d))
                    }
                }
                a
            })
            .flat_map(|a| a);

        let a: Vec<_> = par.collect();

        SingleCollisionList {
            _p: PhantomData,
            a,
            orig: myptr(self.get_bots_mut()),
        }
    }

    pub fn collect_intersections_list_par<D: Send + Sync>(
        &mut self,
        func: impl Fn(&mut T, &mut T) -> Option<D> + Send + Sync + Copy,
    ) -> BotCollisionPar<'a, T, D> {
        let cols = create_collision_list_par(self.tree.as_tree_mut(), |a, b| {
            match func(unsafe { a.as_mut() }, unsafe { b.as_mut() }) {
                Some(d) => Some((*a, *b, d)),
                None => None,
            }
        });
        BotCollisionPar {
            cols,
            _p: PhantomData,
            orig: myptr(self.get_bots_mut()),
        }
    }
}

use core::marker::PhantomData;
pub struct SingleCollisionList<'a, T, D> {
    _p: PhantomData<&'a mut T>,
    a: Vec<(MyPtr<T>, D)>,
    orig: MyPtr<[T]>,
}
impl<'a, T, D> SingleCollisionList<'a, T, D> {
    pub fn for_every_mut<'b, A: Axis, N: Num>(
        &'b mut self,
        c: &'b mut CollectableDinoTree<'a, A, N, T>,
        mut func: impl FnMut(&mut T, &mut D),
    ) {
        assert_eq!(self.orig.as_ptr(), c.get_bots() as *const _);
        for (a, d) in self.a.iter_mut() {
            func(unsafe { &mut *a.as_mut() }, d)
        }
    }

    pub fn get<'b, A: Axis, N: Num>(&self, c: &'b CollectableDinoTree<'a, A, N, T>) -> &[(&T, D)] {
        assert_eq!(self.orig.as_ptr(), c.get_bots() as *const _);
        let k = unsafe { &*(self.a.as_slice() as *const _ as *const [(&T, D)]) };
        k
    }
}
impl<'a, T: Send + Sync, D: Send + Sync> SingleCollisionList<'a, T, D> {
    pub fn for_every_mut_par<'b, A: Axis, N: Num>(
        &'b mut self,
        c: &'b mut CollectableDinoTree<'a, A, N, T>,
        func: impl Fn(&mut T, &mut D) + Send + Sync + Copy,
    ) {
        assert_eq!(self.orig.as_ptr(), c.get_bots() as *const _);
        use rayon::prelude::*;
        self.a
            .par_iter_mut()
            .for_each(|(a, d)| func(unsafe { &mut *a.as_mut() }, d));
    }
}

pub struct BotCollision<'a, T, D> {
    _p: PhantomData<&'a mut T>,
    cols: Vec<(MyPtr<T>, MyPtr<T>, D)>,
    orig: MyPtr<[T]>,
}
impl<'a, T, D> BotCollision<'a, T, D> {
    pub fn get<'b, A: Axis, N: Num>(
        &self,
        c: &'b CollectableDinoTree<'a, A, N, T>,
    ) -> &'b [(&T, &T, D)] {
        assert_eq!(self.orig.as_ptr(), c.get_bots() as *const _);
        unsafe { &*(&self.cols as &[_] as *const _ as *const _) }
    }

    pub fn for_every_pair_mut<'b, A: Axis, N: Num>(
        &'b mut self,
        c: &'b mut CollectableDinoTree<'a, A, N, T>,
        mut func: impl FnMut(&mut T, &mut T, &mut D),
    ) {
        assert_eq!(self.orig.as_ptr(), c.get_bots() as *const _);
        for (a, b, d) in self.cols.iter_mut() {
            let a = unsafe { a.as_mut() };
            let b = unsafe { b.as_mut() };
            func(a, b, d)
        }
    }
}

pub struct BotCollisionPar<'a, T, D> {
    _p: PhantomData<&'a mut T>,
    cols: Vec<Vec<(MyPtr<T>, MyPtr<T>, D)>>,
    orig: MyPtr<[T]>,
}

impl<'a, T, D> BotCollisionPar<'a, T, D> {
    pub fn get<'b, A: Axis, N: Num>(
        &self,
        c: &'b CollectableDinoTree<'a, A, N, T>,
    ) -> &'b [Vec<(&T, &T, D)>] {
        assert_eq!(self.orig.as_ptr(), c.get_bots() as *const _);
        unsafe { &*(&self.cols as &[_] as *const _ as *const _) }
    }
    pub fn for_every_pair_mut<'b, A: Axis, N: Num>(
        &'b mut self,
        c: &'b mut CollectableDinoTree<'a, A, N, T>,
        mut func: impl FnMut(&mut T, &mut T, &mut D),
    ) {
        assert_eq!(self.orig.as_ptr(), c.get_bots() as *const _);
        for a in self.cols.iter_mut() {
            for (a, b, d) in a.iter_mut() {
                let a = unsafe { a.as_mut() };
                let b = unsafe { b.as_mut() };
                func(a, b, d)
            }
        }
    }
}
impl<'a, T: Send + Sync, D: Send + Sync> BotCollisionPar<'a, T, D> {
    pub fn for_every_pair_mut_par<'b, A: Axis, N: Num>(
        &'b mut self,
        c: &'b mut CollectableDinoTree<'a, A, N, T>,
        func: impl Fn(&mut T, &mut T, &mut D) + Send + Sync + Copy,
    ) {
        assert_eq!(self.orig.as_ptr(), c.get_bots() as *const _);

        fn parallelize<T: Visitor + Send + Sync>(a: T, func: impl Fn(T::Item) + Sync + Send + Copy)
        where
            T::Item: Send + Sync,
        {
            let (n, l) = a.next();
            func(n);
            if let Some([left, right]) = l {
                rayon::join(|| parallelize(left, func), || parallelize(right, func));
            }
        }

        let mtree = compt::dfs_order::CompleteTree::from_preorder_mut(&mut self.cols).unwrap();

        parallelize(mtree.vistr_mut(), |a| {
            for (a, b, d) in a.iter_mut() {
                let a = unsafe { a.as_mut() };
                let b = unsafe { b.as_mut() };
                func(a, b, d)
            }
        });
    }
}

fn create_collision_list<'a, A: Axis, T: Aabb + HasInner, D>(
    tree: &mut DinoTree<A, T>,
    mut func: impl FnMut(&mut T::Inner, &mut T::Inner) -> Option<D>,
) -> Vec<D> {
    let mut nodes: Vec<_> = Vec::new();

    tree.find_intersections_mut(|a, b| {
        if let Some(d) = func(a, b) {
            nodes.push(d);
        }
    });

    nodes
}
fn create_collision_list_par<'a, A: Axis, T: Aabb + HasInner + Send + Sync, D: Send + Sync>(
    tree: &mut DinoTree<A, T>,
    func: impl Fn(&mut T::Inner, &mut T::Inner) -> Option<D> + Send + Sync + Copy,
) -> Vec<Vec<D>> {
    struct Foo<T: Visitor> {
        current: T::Item,
        next: Option<[T; 2]>,
    }
    impl<T: Visitor> Foo<T> {
        fn new(a: T) -> Foo<T> {
            let (n, f) = a.next();
            Foo {
                current: n,
                next: f,
            }
        }
    }

    //TODO might break if user uses custom height
    let height =
        1 + par::compute_level_switch_sequential(par::SWITCH_SEQUENTIAL_DEFAULT, tree.get_height())
            .get_depth_to_switch_at();
    //dbg!(tree.get_height(),height);
    let mut nodes: Vec<Vec<D>> = (0..compt::compute_num_nodes(height))
        .map(|_| Vec::new())
        .collect();
    let mtree = compt::dfs_order::CompleteTree::from_preorder_mut(&mut nodes).unwrap();

    tree.find_intersections_par_ext(
        move |a| {
            let next = a.next.take();
            if let Some([left, right]) = next {
                let l = Foo::new(left);
                let r = Foo::new(right);
                *a = l;
                r
            } else {
                unreachable!()
            }
        },
        move |_a, _b| {},
        move |c, a, b| {
            if let Some(d) = func(a, b) {
                c.current.push(d);
            }
        },
        Foo::new(mtree.vistr_mut()),
    );

    nodes
}
