use super::*;
//use crate::inner_prelude::*;
    
///Equivalent to: `(Rect<N>,*mut T)` 
#[repr(C)]
pub struct BBoxPtr<N, T> {
    rect: axgeom::Rect<N>,
    inner: core::ptr::NonNull<T>,
}

impl<N, T> BBoxPtr<N, T> {
    #[inline(always)]
    pub fn new(rect: axgeom::Rect<N>, inner: core::ptr::NonNull<T>) -> BBoxPtr<N, T> {
        BBoxPtr { rect, inner}
    }
}

unsafe impl<N,T> Send for BBoxPtr<N,T>{}
unsafe impl<N,T> Sync for BBoxPtr<N,T>{}


unsafe impl<N: NumTrait, T> HasAabb for BBoxPtr<N, T> {
    type Num = N;
    #[inline(always)]
    fn get(&self) -> &Rect<Self::Num>{
        &self.rect
    }
}

impl<N:NumTrait,T> HasInner for BBoxPtr<N,T>{
    type Inner= T;

    #[inline(always)]
    fn get_inner(&self)->(&Rect<N>,&Self::Inner){
        (&self.rect,unsafe{self.inner.as_ref()})
    }

    #[inline(always)]
    fn get_inner_mut(&mut self)->(&Rect<N>,&mut Self::Inner){
        (&self.rect,unsafe{self.inner.as_mut()})
    }
}


///A Node in a dinotree.
pub struct NodePtr<T: HasAabb> {
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


impl<T:HasAabb> NodeTrait for NodePtr<T>{
    type T=T;
    type Num=T::Num;
    fn get(&self)->NodeRef<Self::T>{
        NodeRef{bots:unsafe{self.range.as_ref()},cont:&self.cont,div:&self.div}
    }
    fn get_mut(&mut self)->NodeRefMut<Self::T>{
        NodeRefMut{bots:ProtectedBBoxSlice::new(unsafe{self.range.as_mut()}),cont:&self.cont,div:&self.div}
    }
}


///An owned dinotree
pub struct DinoTreeOwned<A:AxisTrait,N:NumTrait,T>{
    inner:DinoTree<A,NodePtr<BBoxPtr<N,T>>>,
    bots_aabb:Vec<BBoxPtr<N,T>>,
    bots:Vec<T>
}



impl<N:NumTrait,T : Send + Sync> DinoTreeOwned<DefaultAxis,N,T>{
    pub fn new_par(
        bots:Vec<T>,
        aabb_create:impl FnMut(&T)->Rect<N>)->DinoTreeOwned<DefaultAxis,N,T>{
        Self::with_axis_par(default_axis(),bots,aabb_create)
    }
}

impl<A:AxisTrait,N:NumTrait,T : Send + Sync> DinoTreeOwned<A,N,T>{

    ///Create an owned dinotree in one thread.
    pub fn with_axis_par(
        axis:A,
        mut bots:Vec<T>,
        mut aabb_create:impl FnMut(&T)->Rect<N>)->DinoTreeOwned<A,N,T>{
        let mut bots_aabb:Vec<BBoxPtr<N,T>>=bots.iter_mut().map(|k|BBoxPtr::new(aabb_create(k),core::ptr::NonNull::new(k).unwrap())).collect();

        let inner = DinoTreeBuilder::new(axis,&mut bots_aabb).build_par();
        
        let inner:Vec<_>=inner.inner.into_nodes().drain(..).map(|node|NodePtr{range:core::ptr::NonNull::new(node.range).unwrap(),cont:node.cont,div:node.div}).collect(); 
        let inner=compt::dfs_order::CompleteTreeContainer::from_preorder(inner).unwrap();
        DinoTreeOwned{
            inner:DinoTree{
                axis,
                inner
            },
            bots_aabb,
            bots
        }
    }
}

impl<N:NumTrait,T> DinoTreeOwned<DefaultAxis,N,T>{
    pub fn new(
        bots:Vec<T>,
        aabb_create:impl FnMut(&T)->Rect<N>)->DinoTreeOwned<DefaultAxis,N,T>{
        Self::with_axis(default_axis(),bots,aabb_create)
    }
}
impl<A:AxisTrait,N:NumTrait,T> DinoTreeOwned<A,N,T>{
    
    ///Create an owned dinotree in parallel.
    pub fn with_axis(
        axis:A,
        mut bots:Vec<T>,
        mut aabb_create:impl FnMut(&T)->Rect<N>)->DinoTreeOwned<A,N,T>{
        let mut bots_aabb:Vec<BBoxPtr<N,T>>=bots.iter_mut().map(|k|BBoxPtr::new(aabb_create(k),core::ptr::NonNull::new(k).unwrap())).collect();

        let inner = DinoTreeBuilder::new(axis,&mut bots_aabb).build_seq();
        
        let inner:Vec<_>=inner.inner.into_nodes().drain(..).map(|node|NodePtr{range:core::ptr::NonNull::new(node.range).unwrap(),cont:node.cont,div:node.div}).collect(); 
        let inner=compt::dfs_order::CompleteTreeContainer::from_preorder(inner).unwrap();
        DinoTreeOwned{
            inner:DinoTree{
                axis,
                inner
            },
            bots_aabb,
            bots
        }
    }

    pub fn get(&self)->&DinoTree<A,NodePtr<BBoxPtr<N,T>>>{
        &self.inner
    }
    pub fn get_mut(&mut self)->&mut DinoTree<A,NodePtr<BBoxPtr<N,T>>>{
        &mut self.inner
    }

    pub fn get_aabb_bots_mut(&mut self)->ProtectedBBoxSlice<BBoxPtr<N,T>>{
        ProtectedBBoxSlice::new(&mut self.bots_aabb)
    }

    pub unsafe fn get_aabb_bots_mut_not_protected(&mut self)->&mut [BBoxPtr<N,T>]{
        &mut self.bots_aabb
    }


    pub fn get_aabb_bots(&self)->&[BBoxPtr<N,T>]{
        &self.bots_aabb
    }
    pub fn get_height(&self)->usize{
        self.inner.get_height()
    }

    pub fn axis(&self)->A{
        self.inner.axis()
    }

    pub fn into_inner(self)->Vec<T>{
        self.bots
    }
}    

