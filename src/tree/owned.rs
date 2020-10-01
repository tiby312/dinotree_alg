//!
//! ## An owned `(Rect<N>,T)` example
//!
//! ```rust
//! use dinotree_alg::{*,owned::*};
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
//! use dinotree_alg::{*,owned::*};
//! use axgeom::*;
//!
//! fn not_lifetimed()->DinoTreeOwnedInd<DefaultA,i32,Vec2<i32>>
//! {
//!     let rect=vec![vec2(0,10),vec2(3,30)];
//!     DinoTreeOwnedInd::new(rect,|&p|{
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
pub(crate) struct MyPtr<T: ?Sized>(NonNull<T>);
impl<T: ?Sized> Clone for MyPtr<T> {
    fn clone(&self) -> Self {
        MyPtr(self.0)
    }
}
impl<T: ?Sized> Copy for MyPtr<T> {}

unsafe impl<T: ?Sized> Send for MyPtr<T> {}
unsafe impl<T: ?Sized> Sync for MyPtr<T> {}

pub(crate) fn myptr<T: ?Sized>(a: &mut T) -> MyPtr<T> {
    MyPtr(unsafe { NonNull::new_unchecked(a as *mut _) })
}

unsafe impl<T: Aabb> Send for NodePtr<T> {}
unsafe impl<T: Aabb> Sync for NodePtr<T> {}

///A Node in a dinotree.
pub(crate) struct NodePtr<T: Aabb> {
    _range: PMutPtr<[T]>,

    //range is empty iff cont is none.
    _cont: Option<axgeom::Range<T::Num>>,
    //for non leafs:
    //  div is some iff mid is nonempty.
    //  div is none iff mid is empty.
    //for leafs:
    //  div is none
    _div: Option<T::Num>,
}

pub(crate) fn make_owned<A: Axis, T: Aabb>(axis: A, bots: &mut [T]) -> DinoTreeInner<A, NodePtr<T>> {
    
    let inner = DinoTree::with_axis(axis, bots);
    let inner: Vec<_> = inner
        .inner
        .inner
        .into_nodes()
        .drain(..)
        .map(|mut node| NodePtr {
            _range: node.range.as_ptr(),
            _cont: node.cont,
            _div: node.div,
        })
        .collect();
    let inner = compt::dfs_order::CompleteTreeContainer::from_preorder(inner).unwrap();
    DinoTreeInner {
        axis,
        inner
    }
}

fn make_owned_par<A: Axis, T: Aabb + Send + Sync>(axis: A, bots: &mut [T]) -> DinoTreeInner<A, NodePtr<T>> {
    let inner = DinoTree::with_axis_par(axis, bots);
    let inner: Vec<_> = inner
        .inner
        .inner
        .into_nodes()
        .drain(..)
        .map(|mut node| NodePtr {
            _range: node.range.as_ptr(),
            _cont: node.cont,
            _div: node.div,
        })
        .collect();
    let inner = compt::dfs_order::CompleteTreeContainer::from_preorder(inner).unwrap();
    DinoTreeInner {
        axis,
        inner
    }
}



pub struct DinoTreeOwnedInd<A: Axis,N:Num, T> {
    inner:DinoTreeOwned<A,BBox<N,*mut T>>,
    bots:Vec<T>
}


impl<N:Num,T> DinoTreeOwnedInd<DefaultA,N, T> {
    pub fn new(bots: Vec<T>,mut func:impl FnMut(&T)->Rect<N>) -> DinoTreeOwnedInd<DefaultA, N,T> {
        Self::with_axis(default_axis(), bots,func)
    }    
}

impl<A:Axis,N:Num,T> DinoTreeOwnedInd<A,N,T>{
    ///Create an owned dinotree in one thread.
    pub fn with_axis(axis: A, mut bots: Vec<T>,mut func:impl FnMut(&T)->Rect<N>) -> DinoTreeOwnedInd<A,N, T> {
        let bbox = bots
            .iter_mut()
            .map(|b| BBox::new(func(b), b as *mut _))
            .collect();
        
        let inner= DinoTreeOwned::with_axis(axis,bbox); 
        DinoTreeOwnedInd {
            inner,
            bots,
        }
        
    }

    ///Cant use Deref because of lifetime
    pub fn as_tree(&self)->&DinoTree<A,BBox<N,&mut T>>{
        unsafe{&*(self.inner.as_tree() as *const _ as *const _)}
    }

    ///Cant use Deref because of lifetime
    pub fn as_tree_mut(&mut self)->&mut DinoTree<A,BBox<N,&mut T>>{
        unsafe{&mut *(self.inner.as_tree_mut() as *mut _ as *mut _)}
    }

    
    pub fn get_elements(&self) -> &[T] {
        &self.bots
    }
    pub fn get_elements_mut(&mut self) -> &mut [T] {
        &mut self.bots
    }
}



///An owned dinotree componsed of `T:Aabb`
pub struct DinoTreeOwned<A: Axis, T: Aabb> {
    tree: DinoTreeInner<A, NodePtr<T>>,
    bots: Vec<T>,
}

impl<T: Aabb> DinoTreeOwned<DefaultA, T> {
    pub fn new(bots: Vec<T>) -> DinoTreeOwned<DefaultA, T> {
        Self::with_axis(default_axis(), bots)
    }
    
}

impl<A: Axis, T: Aabb> DinoTreeOwned<A, T> {
    ///Create an owned dinotree in one thread.
    pub fn with_axis(axis: A, mut bots: Vec<T>) -> DinoTreeOwned<A, T> {
        DinoTreeOwned {
            tree: make_owned(axis, &mut bots),
            bots,
        }
    }
    
    ///Cant use Deref because of lifetime
    pub fn as_tree(&self)->&DinoTree<A,T>{
        unsafe{&*(&self.tree as *const _ as *const _)}
    }

    ///Cant use Deref because of lifetime
    pub fn as_tree_mut(&mut self)->&mut DinoTree<A,T>{
        unsafe{&mut *(&mut self.tree as *mut _ as *mut _)}
    }

    pub fn get_elements(&self) -> &[T] {
        &self.bots
    }
    pub fn get_elements_mut(&mut self) -> PMut<[T]> {
        PMut::new(&mut self.bots)
    }
    pub fn rebuild(&mut self, mut func: impl FnMut(&mut [T])) {
        func(&mut self.bots);

        let axis = self.tree.axis;
        self.tree = make_owned(axis, &mut self.bots);
    }

}
