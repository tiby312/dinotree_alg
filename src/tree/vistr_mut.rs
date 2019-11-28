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