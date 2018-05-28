//! An iterative mulithreaded hybrid kdtree/mark and sweep algorithm used for broadphase detection.
//! ## Goal
//! To provide a fast and simple to use broad-phase collision system.
//!
//! ## Notes                      
//!
//! Checkout included examples. 
//!
//! The mutable reference to each element in the callback functions do not point to elements
//! in the user supplied slice of elements. The elements are internally unsafely copied directly into a tree structure
//! and then unsafely copied back into the slice at the end. So do not try storing the mutable references as pointers
//! in the callback functions since they would point to unallocated memory once the tree is destroyed.
//!
//! ## Use of unsafety           
//!
//! The multirect api uses unsafety internally. We unsafely convert the refernces returned by the rect query
//! closure to have a longer lifetime.
//! This allows the user to store mutable references of non intersecting rectangles at the same time. 
//! The multirect api panics at run time if the user attemps to query
//! rectangles that intersect. This protects the invariant at runtime. So it this use unsafety can be hidden by the api.
//!
//! The bots are unsafely copied into a tree, and then usafely copied back out. The algorithm ensures
//! That even though the ordering is different, this is a bijection between the two sets.
//! So we can safely hide this unsafety from the user.
//! The bots are copied back in the trees drop() method. If the user panics inside of a callback function,
//! The changes to the bots up until that more during the traversal of the tree will take effect when the 
//! trees drop() occurrs.
//!
//! The sequential version of the pair intersection uses unsafe{} to re-use code from the parallel version.
//! That is protected at runtime. It will panic if the parallel version tries to copy the closure.
//!
//! Unsafety is used to construct the special variable node size tree structure that is populated with dsts.
//!

#![feature(iterator_step_by)]
#![feature(test)]


extern crate axgeom;
extern crate compt;
extern crate ordered_float;
extern crate pdqselect;
extern crate rayon;
extern crate unsafe_unwrap;
extern crate smallvec;
extern crate dinotree_inner;

#[cfg(test)]
extern crate num;
#[cfg(test)]
extern crate cgmath;
#[cfg(test)]
extern crate collision;
#[cfg(test)]
extern crate rand;
#[cfg(test)]
extern crate test;

mod inner_prelude {
    pub use dinotree_inner::prelude::*;
    pub use AABBox;
    pub use axgeom::Axis;
    pub use compt::LevelIter;
    pub use compt::Depth;
    pub use axgeom::Range;
    pub use ::*;
    pub use oned::Sweeper;
    pub use compt::CTreeIterator;
    //pub use par;
    pub use axgeom::AxisTrait;
    pub use std::marker::PhantomData;
    pub use NumTrait;
    pub use ::*;
}

///Provides functionality to draw the dividers of a dinotree.
pub mod graphics;

///Contains convenience structs.
pub mod support;

///Contains query code
mod colfind;

mod k_nearest;
pub use k_nearest::Knearest;

//pub use nbody::CenterOfMass;
pub use nbody::NodeMassTrait;
mod nbody;

mod raycast;
pub use raycast::RayTrait;
pub use raycast::ray::Ray;
//pub use raycast::Vec2;
//pub use raycast::RectInf;

mod rect;

///A collection of 1d functions that operate on lists of 2d objects.
mod oned;

///Contains misc tools
mod tools;

pub use rects::Rects;
pub use rects::RectIntersectError;
mod rects;

//use dinotree_inner::support::DefaultDepthLevel;
pub use dinotree_inner::AABBox;
pub use dinotree_inner::NumTrait;
pub use dinotree_inner::SweepTrait;
use dinotree_inner::TreeTimerTrait;
use dinotree_inner::par;
use axgeom::Rect;
use axgeom::XAXISS;
use axgeom::YAXISS;
use colfind::ColMulti;
use smallvec::SmallVec;
use dinotree_inner::TreeTimer2;
use dinotree_inner::TreeTimerEmpty;
use dinotree_inner::compute_tree_height;
use axgeom::AxisTrait;
///Represents a destructured SweepTrait into the immutable bounding box reference,
///and the mutable reference to the rest of the object.
pub struct ColSingle<'a, T: SweepTrait + 'a> {
    pub rect: &'a AABBox<T::Num>,
    pub inner: &'a mut T::Inner,
}



use dinotree_inner::DynTree;

pub use ba::DinoTree;
pub(crate) use ba::DynTreeEnum;

use std::marker::PhantomData;


///If Xaxis, then the first divider will be a line splitting the x axis.
///So it would be a vertical line.
///TODO test this
#[derive(Copy,Clone,Debug)]
pub enum StartAxis{
    Xaxis,
    Yaxis
}


mod ba {
    use super::*;
    use DynTree;

    mod closure_struct {
        use super::*;
        use ColSingle;
        use ColMulti;

        pub struct ColMultiStruct<
            'a,
            A:Send,
            T: SweepTrait<Inner = I>,
            I: Send + Sync,
            F: Fn(&mut A,ColSingle<T>, ColSingle<T>) + Send + Sync + 'a,
            F2:Fn(A)->(A,A)+Sync+'a,
            F3:Fn(A,A)->A+Sync+'a
        > {
            pub a: &'a F,
            pub f2: &'a F2,
            pub f3: &'a F3,
            pub aa:A,
            pub _p: PhantomData<(T)>,
        }


        impl<
            'a,
            A:Send+Sync,
            T: SweepTrait<Inner = I>,
            I: Send + Sync,
            F: Fn(&mut A,ColSingle<T>, ColSingle<T>) + Send + Sync,
            F2:Fn(A)->(A,A)+Sync+'a,
            F3:Fn(A,A)->A+Sync+'a
        > ColMulti for ColMultiStruct<'a, A,T, I, F,F2,F3>
        {
            type T = T;
        
            fn collide(&mut self,a: ColSingle<T>, b: ColSingle<T>) {
                (self.a)(&mut self.aa,a,b);
            }
            fn div(self)->(Self,Self){
                let (aa1,aa2)=(self.f2)(self.aa);
                
                let c1=ColMultiStruct{a:self.a,f2:self.f2,f3:self.f3,aa:aa1,_p:PhantomData};
                let c2=ColMultiStruct{a:self.a,f2:self.f2,f3:self.f3,aa:aa2,_p:PhantomData};
                (c1,c2)
            }
            fn add(self,b:Self)->Self{

                let aa_n=(self.f3)(self.aa,b.aa);
                
                ColMultiStruct{a:self.a,f2:self.f2,f3:self.f3,aa:aa_n,_p:PhantomData}
            }
        }
    }


    pub(crate) enum DynTreeEnum<'a, T: SweepTrait + 'a> {
        Xa(DynTree<'a, XAXISS,(), T>),
        Ya(DynTree<'a, YAXISS,(), T>),
    }
    fn make<'a,T:SweepTrait+Send,JJ:par::Joiner,K:TreeTimerTrait>(axis:StartAxis,rest:&'a mut [T])->(DinoTree<'a,T>,K::Bag){
        let height = self::compute_tree_height(rest.len());
        match axis{
            StartAxis::Xaxis=>{
                let k=DynTree::<XAXISS,(),T>::new::<JJ,K>(rest,height);
                (DinoTree(DynTreeEnum::Xa(k.0)),k.1)
            },
            StartAxis::Yaxis=>{
                let k=DynTree::<YAXISS,(),T>::new::<JJ,K>(rest,height);
                (DinoTree(DynTreeEnum::Ya(k.0)),k.1)
            }
        }
    }


    ///This is the struct that this crate revolves around.
    pub struct DinoTree<'a, T: SweepTrait +Send+ 'a>(pub(crate) DynTreeEnum<'a, T>);

    impl<'a, T: SweepTrait +Send+ 'a> DinoTree<'a, T> {
        ///Create a dinotree.
        ///Specify the starting axis along which the bots will be partitioned.
        ///So if you picked the x axis, the root divider will be a vertical line.
        ///True means xaxis.
        ///The length of the slice must be less than the max value of a u32.
        pub fn new(rest: &'a mut [T], axis: StartAxis) -> DinoTree<'a, T> {
            self::make::<_,par::Parallel,TreeTimerEmpty>(axis,rest).0
        }

        ///Create a dinotree that does not use any parallel algorithms.
        pub fn new_seq(rest: &'a mut [T], axis: StartAxis) -> DinoTree<'a, T> {
            self::make::<_,par::Sequential,TreeTimerEmpty>(axis,rest).0
        }

        ///Create a dinotree that does not use any parallel algorithms.
        ///Returns time each level took in seconds. First element is root time, last element is last level time.
        pub fn new_seq_debug(rest: &'a mut [T], axis: StartAxis) -> (DinoTree<'a, T>, Vec<f64>) {
            let (a,b)=self::make::<_,par::Sequential,TreeTimer2>(axis,rest);
            return (a,b.into_vec());
        }

        ///Create a rect finding session.
        ///From this returned argument, you can retrive references to all elements
        ///within non-intersecting rectangles.
        pub fn rects<'b>(&'b mut self) -> Rects<'a, 'b, T> {
            Rects::new(self)
        }

        ///Find all intersecting pairs between the elements in this dinotree, and the specified elements.
        ///No intersecting pairs within each group are looked for, only those between the two groups.
        ///For best performance the group that this tree is built around should be the bigger of the two groups.
        pub fn intersect_with_seq<X: SweepTrait<Num = T::Num>>(
            &mut self,
            b: &mut [X],
            mut func: impl FnMut(ColSingle<T>, ColSingle<X>),
        ) {
            //todo find better algorithn?
            //todo do this before axis specialization?
            //ideally you'd bin the new group using the existing dividers and query that.
            for i in b.iter_mut() {
                let jj = i.get_mut();

                self.for_all_intersect_rect(jj.0, |a: ColSingle<T>| {
                    let blag = ColSingle {
                        rect: jj.0,
                        inner: jj.1,
                    };
                    func(a, blag);
                });
            }
        }

        ///Unlike the rects session api, this function returns all elements within the specified
        ///rectangle AND all those that intersect with it. This more relaxed requirement means that
        ///we can no longer query non intersecting rectangles simulatiously and be assured that the two respective
        ///sets of bots are disjoint.
        pub fn for_all_intersect_rect(
            &mut self,
            rect: &AABBox<T::Num>,
            fu: impl FnMut(ColSingle<T>),
        ) {
            match &mut self.0 {
                DynTreeEnum::Xa(a) => {
                    rect::for_all_intersect_rect(a, &rect.0, fu);
                }
                DynTreeEnum::Ya(a) => {
                    rect::for_all_intersect_rect(a, &rect.0, fu);
                }
            }
        }

        
        pub fn raycast<'b,R:RayTrait<T=T,N=T::Num>> //called to test if this object touches the ray. if it does, return distance to start of ray
            (
                &'b mut self,
                ray:Ray<T::Num>,
                rtrait:R
                )->Option<(ColSingle<'b,T>,T::Num)>{
            
            match &mut self.0 {
                DynTreeEnum::Xa(a) => {
                    raycast::raycast(
                        a,
                        ray,
                        rtrait
                    )
                }
                DynTreeEnum::Ya(a) => {
                    raycast::raycast(
                        a,
                        ray,
                        rtrait
                    )
                }
            }
            
        }

        ///The dinotree's NumTrait does not inherit any kind of arithmetic traits.
        ///This showcases that the tree construction and pair finding collision algorithms
        ///do not involves any arithmetic. 
        ///However, when finding the nearest neighbor, we need to do some calculations to
        ///compute distance between points. So instead of giving the NumTrait arithmetic and thus
        ///add uneeded bounds for general use of this tree, the user must provide functions for arithmetic
        ///specifically for this function.
        ///The user can also specify what the minimum distance function is minizing based off of. For example
        ///minimizing based off the square distance will give you the same answer as minimizing based off 
        ///of the distant. 
        ///The callback function will be called on the closest object, then the second closest, and so on up 
        ///until k.
        ///User can also this way choose whether to use manhatan distance or not.
        //TODO pass trait instead? So that the user can mutably borrow something between the closures.
        pub fn k_nearest<'b,K:Knearest<T=T,N=T::Num>>(
            &'b mut self,
            point: [T::Num;2],
            num:usize,
            knear:K,
            func: impl FnMut(ColSingle<'b,K::T>,K::D)
        ) where K::N:'b{
            
            match &mut self.0 {
                DynTreeEnum::Xa(a) => {
                    k_nearest::k_nearest(
                        a,
                        point,
                        num,
                        knear,
                        func
                    )
                }
                DynTreeEnum::Ya(a) => {
                    k_nearest::k_nearest(
                        a,
                        point,
                        num,
                        knear,
                        func
                    )
                }
            };
            
            
        }

        ///Sequential version of the parallel n_body. 
        pub fn n_body_seq<N:NodeMassTrait<T=T>+Send>(&mut self,ncontext:N) where N::No:Send{
            match &mut self.0{
                DynTreeEnum::Xa(a)=>{
                    nbody::nbody_seq(a,ncontext);
                },
                DynTreeEnum::Ya(a)=>{
                    nbody::nbody_seq(a,ncontext);
                }
            }
        }


        ///The nbody algorithm takes a user defined implementation of this trait.
        ///The dinotree nbody algorithm works by first creating a tree of node masses.
        ///Every node mass has the total mass of all the bots in the corresponding dinotree node, as well as
        ///the mass of all the bots underneath that node. It also has a center of mass (calculated as weighted average of all the bot masses)
        ///Once this node mass tree is constructed, the dinotree is recursed. 
        ///For every node, do three things.
        ///1. gravitate bots with each other that belong to this node.
        ///2. handle all children bots with the bots in this node. (this is its own algorithm).
        ///3. handle all children bots on the left of the divider, with all children bots on the right of the divider.
        ///4. at this point the node can be removed from the tree, and we can handle its two children as two completely independant trees.
        ///
        ///step 2 is done in the following way.Recurse left until you find a node who's node mass is sufficiently far away, then
        ///gravitate that node mass with the anchor's node mass. Do the same for the right.
        ///
        ///step 3 is done in the following way. Recurse left.If a node mass is sufficiently far away from the anchor's divider,
        ///gravitate that node mass with all the nodemasses/bots found on the right side. Otherwise recurse.
        ///For this algorithm, we care about the distance between the nodemass and the anchor's divider. So it is half as long 
        ///as the distance we cared about in step 2.
        pub fn n_body<N:NodeMassTrait<T=T>+Send>(&mut self,ncontext:N) where N::No:Send{
            match &mut self.0{
                DynTreeEnum::Xa(a)=>{
                    nbody::nbody_par(a,ncontext);
                },
                DynTreeEnum::Ya(a)=>{
                    nbody::nbody_par(a,ncontext);
                }
            }
        }


        ///Find all intersecting pairs sequentially.
        ///Notice that in this case, a FnMut is supplied instead of a Fn.
        pub fn intersect_every_pair_seq(&mut self, clos: impl FnMut(ColSingle<T>, ColSingle<T>)) {
            match &mut self.0 {
                DynTreeEnum::Xa(a) => {
                    colfind::for_every_col_pair_seq::<_, _, _, TreeTimerEmpty>(
                        a,
                        clos,
                    )
                }
                DynTreeEnum::Ya(a) => {
                    colfind::for_every_col_pair_seq::<_, _, _, TreeTimerEmpty>(
                        a,
                        clos,
                    )
                }
            };
        }


        ///Gives user the option to store some internals to the bots
        ///in vecs. Its main purpose is so that you could cache the ids 
        ///of colliding pairs.
        pub fn intersect_every_pair_adv<A:Send+Sync>(
            &mut self,
            a:A,
            clos: impl Fn(&mut A,ColSingle<T>, ColSingle<T>) + Send + Sync,
            f2:impl Fn(A)->(A,A)+Sync,
            f3:impl Fn(A,A)->A+Sync,
        )->A {
            let clos = self::closure_struct::ColMultiStruct{aa:a,a:&clos,f2:&f2,f3:&f3,_p:PhantomData};

            let ans=match &mut self.0 {
                DynTreeEnum::Xa(a) => {
                    colfind::for_every_col_pair::<_, _, _, TreeTimerEmpty>(
                        a,
                        clos,
                    )
                }
                DynTreeEnum::Ya(a) => {
                    colfind::for_every_col_pair::<_, _, _, TreeTimerEmpty>(
                        a,
                        clos,
                    )
                }
            };
            ans.0.aa
        }


        ///Find all intersecting pairs.
        ///Optionally return time data of each level of the tree.
        ///if a and b collide only one ordering of the pair is returned. The ordering is undefined. 
        ///if a pair collides the callback function is called exactly once for that pair.
        ///if the pair does not collide the callback function is not called.
        pub fn intersect_every_pair(
            &mut self,
            clos: impl Fn(ColSingle<T>, ColSingle<T>) + Send + Sync,
        ) {
            let c1=|_:&mut (),a:ColSingle<T>,b:ColSingle<T>|{
                clos(a,b);
            };
            let c2=|_:()|((),());
            let c3=|_:(),_:()|();

            let clos = self::closure_struct::ColMultiStruct{aa
                :(),a:&c1,f2:&c2,f3:&c3,_p:PhantomData};

            match &mut self.0 {
                DynTreeEnum::Xa(a) => {
                    colfind::for_every_col_pair::<_, _, _, TreeTimerEmpty>(
                        a,
                        clos,
                    )
                }
                DynTreeEnum::Ya(a) => {
                    colfind::for_every_col_pair::<_, _, _, TreeTimerEmpty>(
                        a,
                        clos,
                    )
                }
            };
        }

        ///Returns time each level took in seconds.
        ///Returns time each level took in seconds. First element is root time, last element is last level time.
        pub fn intersect_every_pair_seq_debug(
            &mut self,
            clos: impl FnMut(ColSingle<T>, ColSingle<T>),
        ) -> Vec<f64> {
            match &mut self.0 {
                DynTreeEnum::Xa(a) => {
                    colfind::for_every_col_pair_seq::<_, _, _, TreeTimer2>(
                        a,
                        clos,
                    )
                }
                DynTreeEnum::Ya(a) => {
                    colfind::for_every_col_pair_seq::<_, _, _, TreeTimer2>(
                        a,
                        clos,
                    )
                }
            }.1.into_vec()
        }

        ///Height of the tree.
        pub fn get_height(&self)->usize{
            match &self.0 {
                DynTreeEnum::Xa(a) => {
                    a.get_height()
                }
                DynTreeEnum::Ya(a) => {
                    a.get_height()
                }
            }
        }

        ///Returns the depth and the trail to the element in the tree, if found.
        ///The trail will be a sequence of bools starting from the root. true means left.
        ///Useful for debugging.
        pub fn find_element(&self,func:impl FnMut(&T)->bool)->Option<(usize,Vec<bool>)>{
            match &self.0 {
                DynTreeEnum::Xa(a) => {
                    colfind::find_element(
                        a,
                        func,
                    )
                }
                DynTreeEnum::Ya(a) => {
                    colfind::find_element(
                        a,
                        func,
                    )
                }
            }
        }
    }
}
