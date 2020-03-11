//!
//! ## An owned `(Rect<N>,T)` example
//!
//! ```rust
//! use dinotree_alg::{prelude::*,dinotree_owned::*};
//! use axgeom::*;
//!
//! fn not_lifetimed()->DinoTreeOwned<DefaultA,BBox<i32,f32>>
//! {
//!     let a=vec![bbox(rect(0,10,0,10),0.0)];
//!     DinoTreeOwned::new(a)
//! }
//!
//! not_lifetimed();
//!
//! ```
//!
//! ## An owned `(Rect<N>,*mut T)` example
//!
//! ```rust
//! use dinotree_alg::{prelude::*,dinotree_owned::*};
//! use axgeom::*;
//!
//! fn not_lifetimed()->DinoTreeOwnedBBoxPtr<DefaultA,i32,Vec2<i32>>
//! {
//!     let rect=vec![vec2(0,10),vec2(3,30)];
//!     DinoTreeOwnedBBoxPtr::new(rect,|&p|{
//!         let radius=vec2(10,10);
//!         Rect::from_point(p,radius)
//!     })
//! }
//!
//! not_lifetimed();
//!
//! ```

use super::*;
use core::ptr::NonNull;


#[repr(transparent)]
pub struct MyPtr<T:?Sized>(NonNull<T>);
impl<T:?Sized> Clone for MyPtr<T>{
    fn clone(&self)->Self{
        MyPtr(self.0)
    }
}
impl<T:?Sized> Copy for MyPtr<T>{}

impl<T:?Sized> MyPtr<T>{
    pub unsafe fn as_mut(&mut self)->&mut T{
        self.0.as_mut()
    }
}
unsafe impl<T:?Sized> Send for MyPtr<T>{}
unsafe impl<T:?Sized> Sync for MyPtr<T>{}

pub fn myptr<T>(a:&mut T)->MyPtr<T>{
    MyPtr(unsafe{NonNull::new_unchecked(a as *mut _)})
}

/*
///Equivalent to: `(Rect<N>,*mut T)`
#[repr(C)]
pub struct BBoxPtr<N, T> {
    rect: axgeom::Rect<N>,
    inner: core::ptr::NonNull<T>,
}

impl<N, T> BBoxPtr<N, T> {
    #[inline(always)]
    pub fn new(rect: axgeom::Rect<N>, inner: core::ptr::NonNull<T>) -> BBoxPtr<N, T> {
        BBoxPtr { rect, inner }
    }
}

unsafe impl<N, T> Send for BBoxPtr<N, T> {}
unsafe impl<N, T> Sync for BBoxPtr<N, T> {}

unsafe impl<N: Num, T> Aabb for BBoxPtr<N, T> {
    type Num = N;
    #[inline(always)]
    fn get(&self) -> &Rect<Self::Num> {
        &self.rect
    }
}

impl<N: Num, T> HasInner for BBoxPtr<N, T> {
    type Inner = T;

    #[inline(always)]
    fn get_inner(&self) -> (&Rect<N>, &Self::Inner) {
        (&self.rect, unsafe { self.inner.as_ref() })
    }

    #[inline(always)]
    fn get_inner_mut(&mut self) -> (&Rect<N>, &mut Self::Inner) {
        (&self.rect, unsafe { self.inner.as_mut() })
    }
}
*/

unsafe impl<T:Aabb> Send for NodePtr<T> {}
unsafe impl<T:Aabb> Sync for NodePtr<T> {}

///A Node in a dinotree.
pub struct NodePtr<T: Aabb> {
    range: core::ptr::NonNull<[T]>,

    //range is empty iff cont is none.
    cont: Option<axgeom::Range<T::Num>>,
    //for non leafs:
    //  div is some iff mid is nonempty.
    //  div is none iff mid is empty.
    //for leafs:
    //  div is none
    div: Option<T::Num>,
}

impl<T: Aabb> Node for NodePtr<T> {
    type T = T;
    type Num = T::Num;
    fn get(&self) -> NodeRef<Self::T> {
        NodeRef {
            bots: unsafe { self.range.as_ref() },
            cont: &self.cont,
            div: &self.div,
        }
    }
    fn get_mut(&mut self) -> NodeRefMut<Self::T> {
        NodeRefMut {
            bots: PMut::new(unsafe { self.range.as_mut() }),
            cont: &self.cont,
            div: &self.div,
        }
    }
}


fn make_owned<A: Axis, T: Aabb>(axis: A, bots: &mut [T]) -> DinoTree<A, NodePtr<T>> {
    let inner = DinoTree::with_axis(axis, bots);
    let inner: Vec<_> = inner
        .inner
        .into_nodes()
        .drain(..)
        .map(|node| NodePtr {
            range: core::ptr::NonNull::new(node.range).unwrap(),
            cont: node.cont,
            div: node.div,
        })
        .collect();
    let inner = compt::dfs_order::CompleteTreeContainer::from_preorder(inner).unwrap();
    DinoTree { axis, inner,bot_ptr:bots as *const _ }
}

fn make_owned_par<A: Axis, T: Aabb + Send + Sync>(
    axis: A,
    bots: &mut [T],
) -> DinoTree<A, NodePtr<T>> {
    let inner = DinoTree::with_axis_par(axis, bots);
    let inner: Vec<_> = inner
        .inner
        .into_nodes()
        .drain(..)
        .map(|node| NodePtr {
            range: core::ptr::NonNull::new(node.range).unwrap(),
            cont: node.cont,
            div: node.div,
        })
        .collect();
    let inner = compt::dfs_order::CompleteTreeContainer::from_preorder(inner).unwrap();
    DinoTree { axis, inner,bot_ptr:bots as *const _ }
}

///An owned dinotree componsed of `(Rect<N>,*mut T)`
pub struct DinoTreeOwnedBBoxPtr<A: Axis, N: Num, T> {
    tree: DinoTreeOwned<A, BBox<N, MyPtr<T>>>,
    bots: Vec<T>,
}

impl<N: Num, T: Send + Sync> DinoTreeOwnedBBoxPtr<DefaultA, N, T> {
    pub fn new_par(bots: Vec<T>, func: impl FnMut(&T) -> Rect<N>) -> Self {
        Self::with_axis_par(default_axis(), bots, func)
    }
}

impl<N: Num, T> DinoTreeOwnedBBoxPtr<DefaultA, N, T> {
    pub fn new(bots: Vec<T>, func: impl FnMut(&T) -> Rect<N>) -> Self {
        Self::with_axis(default_axis(), bots, func)
    }
}
impl<A: Axis, N: Num, T: Send + Sync> DinoTreeOwnedBBoxPtr<A, N, T> {
    pub fn with_axis_par(axis: A, mut bots: Vec<T>, mut func: impl FnMut(&T) -> Rect<N>) -> Self {
        let bbox = bots
            .iter_mut()
            .map(|b| BBox::new(func(b), myptr(b)))
            .collect();

        let tree = DinoTreeOwned::with_axis_par(axis, bbox);
        DinoTreeOwnedBBoxPtr { bots, tree }
    }
}

impl<A: Axis, N: Num, T> DinoTreeOwnedBBoxPtr<A, N, T> {
    pub fn with_axis(axis: A, mut bots: Vec<T>, mut func: impl FnMut(&T) -> Rect<N>) -> Self {
        let bbox = bots
            .iter_mut()
            .map(|b| BBox::new(func(b), myptr(b)))
            .collect();

        let tree = DinoTreeOwned::with_axis(axis, bbox);
        DinoTreeOwnedBBoxPtr { bots, tree }
    }
}

impl<A: Axis, N: Num, T> DinoTreeOwnedBBoxPtr<A, N, T> {
    pub fn as_owned(&self) -> &DinoTreeOwned<A, BBox<N, &T>> {
        let a=&self.tree as *const _;
        let b=a as *const DinoTreeOwned<A,BBox<N,&T>>;
        unsafe{&*b}
    }
    pub fn as_owned_mut(&mut self) -> &mut DinoTreeOwned<A, BBox<N, &mut T>> {
        let a=&mut self.tree as *mut _;
        let b=a as *mut DinoTreeOwned<A,BBox<N,&mut T>>;
        unsafe{&mut *b}
    }
    pub fn get_bots(&self) -> &[T] {
        &self.bots
    }
    pub fn get_bots_mut(&mut self) -> &mut [T] {
        &mut self.bots
    }
}

///An owned dinotree componsed of `T:Aabb`
pub struct DinoTreeOwned<A: Axis, T: Aabb> {
    tree: Option<DinoTree<A, NodePtr<T>>>,
    bots: Vec<T>,
}

impl<T: Aabb> DinoTreeOwned<DefaultA, T> {
    pub fn new(bots: Vec<T>) -> DinoTreeOwned<DefaultA, T> {
        Self::with_axis(default_axis(), bots)
    }
}
impl<T: Aabb + Send + Sync> DinoTreeOwned<DefaultA, T> {
    pub fn new_par(bots: Vec<T>) -> DinoTreeOwned<DefaultA, T> {
        Self::with_axis_par(default_axis(), bots)
    }
}

impl<A: Axis, T: Aabb + Send + Sync> DinoTreeOwned<A, T> {
    ///Create an owned dinotree in one thread.
    pub fn with_axis_par(axis: A, mut bots: Vec<T>) -> DinoTreeOwned<A, T> {
        DinoTreeOwned {
            tree: Some(make_owned_par(axis, &mut bots)),
            bots,
        }
    }
}
impl<A: Axis, T: Aabb> DinoTreeOwned<A, T> {
    ///Create an owned dinotree in one thread.
    pub fn with_axis(axis: A, mut bots: Vec<T>) -> DinoTreeOwned<A, T> {
        DinoTreeOwned {
            tree: Some(make_owned(axis, &mut bots)),
            bots,
        }
    }

    pub fn as_tree(&self) -> &DinoTree<A, NodePtr<T>> {
        self.tree.as_ref().unwrap()
    }

    pub fn as_tree_mut(&mut self) -> &mut DinoTree<A, NodePtr<T>> {
        self.tree.as_mut().unwrap()
    }
    pub fn get_bots(&self) -> &[T] {
        &self.bots
    }

    pub fn get_bots_mut(&mut self, mut func: impl FnMut(&mut [T])) {
        func(&mut self.bots);

        let axis = {
            let tree = self.tree.take().unwrap();
            tree.axis()
        };
        self.tree = Some(make_owned(axis, &mut self.bots));
    }
}
