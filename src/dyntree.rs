
use inner_prelude::*;
use tree_alloc::NodeDstDynCont;
use tree_alloc::NodeDyn;
use base_kdtree::Node2;
use compt::GenTree;
use base_kdtree::KdTree;


pub struct NdIterMut<'a:'b,'b,T:SweepTrait+'a>{
    c:&'b mut NodeDstDynCont<'a,T>
}

impl<'a:'b,'b,T:SweepTrait+'a> CTreeIterator for NdIterMut<'a,'b,T>{
    type Item=&'b mut NodeDyn<T>;
    fn next(self)->(Self::Item,Option<(Self,Self)>){
        let i=&mut self.c.0.n;
        let o=match self.c.0.c{
            Some((ref mut a,ref mut b))=>{
                Some((NdIterMut{c:a},NdIterMut{c:b}))
            },
            None=>{
                None
            }
        };
        (i,o)
    }
}

pub struct NdIter<'a:'b,'b,T:SweepTrait+'a>{
    c:&'b NodeDstDynCont<'a,T>
}

impl<'a:'b,'b,T:SweepTrait+'a> CTreeIterator for NdIter<'a,'b,T>{
    type Item=&'b NodeDyn<T>;
    fn next(self)->(Self::Item,Option<(Self,Self)>){
        let i=&self.c.0.n;
        let o=match self.c.0.c{
            Some((ref a,ref b))=>{
                Some((NdIter{c:a},NdIter{c:b}))
            },
            None=>{
                None
            }
        };
        (i,o)
    }
}
/*
///Allows to traverse down from a visitor twice by creating a new visitor that borrows the other.
pub struct Wrap<'a:'b,'b,T:SweepTrait+'a>{
    a:LevelIter<NdIterMut<'a,'b,T>>
}
impl<'a:'b,'b,T:SweepTrait+'a> Wrap<'a,'b,T>{
    #[inline(always)]
    pub fn new(a:&'a mut LevelIter<NdIterMut<'a,'b,T>>)->Wrap<'a,'b,T>{
        let ff=unsafe{
            let mut ff=std::mem::uninitialized();
            std::ptr::copy(a, &mut ff, 1);
            ff
        };
        Wrap{a:ff}
    }
}

impl<'a:'b,'b,T:SweepTrait+'a> CTreeIterator for Wrap<'a,'b,T>{
    type Item=(LevelDesc,&'b mut NodeDyn<T>);
    fn next(self)->(Self::Item,Option<(Self,Self)>){
        let Wrap{a}=self;

        let (item,mm)=a.next();

        match mm{
            Some((left,right))=>{
                let left=Wrap{a:left};
                let right=Wrap{a:right};
                return (item,Some((left,right)));
            },
            None=>{
                return (item,None);
            }
        }
    }
}
*/

pub struct Cont<'b,T:'b>{
    pub a:&'b mut T
}

impl<'b,T:'b+SweepTrait+Send> SweepTrait for Cont<'b,T>{
    type Inner=T::Inner;
    type Num=T::Num;

    ///Destructure into the bounding box and mutable parts.
    fn get_mut<'a>(&'a mut self)->(&'a AABBox<T::Num>,&'a mut Self::Inner){
        self.a.get_mut()
    }

    ///Destructue into the bounding box and inner part.
    fn get<'a>(&'a self)->(&'a AABBox<T::Num>,&'a Self::Inner){
        self.a.get()
    }
}


pub struct DynTree<'b,A:AxisTrait,T:SweepTrait+Send+'b>{
    orig:&'b mut [T],
    tree:DynTreeRaw<'b,T>,
    mover:Mover,
    _p:PhantomData<A>
}


impl<'a,A:AxisTrait,T:SweepTrait+'a> DynTree<'a,A,T>{

    ///Create the tree.
    ///Specify whether it is done in parallel or sequential.
    ///If parallel, also specify the depth at which to switch to sequential.
    ///Also specify the median finding strategy to use.
    ///Also specify whether to use collect timing dat.a
    pub fn new<JJ:par::Joiner,H:DepthLevel,Z:MedianStrat<Num=T::Num>,K:TreeTimerTrait>(
        rest:&'a mut [T],tc:&mut TreeCache<A,T::Num>,medianstrat:&Z) -> (DynTree<'a,A,T>,K::Bag) {

        let num_bots=rest.len();


        //Pointer to the bot. Used to calculate offsets
        let start_pointer=mover::get_start_pointer(rest);

        let (fb,mover,bag)={
            let mut pointers:Vec<Cont<T>>=Vec::with_capacity(rest.len());
            for k in rest.iter_mut(){
                pointers.push(Cont{a:k});
            }
            {
                let (mut tree2,bag)=KdTree::new::<JJ,H,_,K>(&mut pointers,tc,medianstrat);
                
                // 12345
                // 42531     //vector:41302
                //let mut move_vector=Vec::with_capacity(num_bots);    
                let mover={
                    let t=tree2.get_tree().create_down();

                    let k=t.dfs_preorder_iter().flat_map(|a:&Node2<Cont<T>>|{
                        a.range.iter()
                    });

                    Mover::new(num_bots,start_pointer,k)
                };
                
                let fb=DynTreeRaw::new(tree2.into_tree(),num_bots);
                
                (fb,mover,bag)
            }
        };

        (DynTree{orig:rest,tree:fb,mover,_p:PhantomData},bag)
    }
   
    pub fn get_height(&self)->usize{
        self.tree.get_height()
    }

    pub fn get_level_desc(&self)->LevelDesc{
        self.tree.get_level()
    }
    pub fn get_iter_mut<'b>(&'b mut self)->NdIterMut<'a,'b,T>{
        NdIterMut{c:self.tree.get_root_mut()}
    }
    pub fn get_iter<'b>(&'b self)->NdIter<'a,'b,T>{
        NdIter{c:self.tree.get_root()}
    }
}



use self::mover::Mover;
mod mover{
    use std;
    use super::Cont;

    pub struct Mover(
        Vec<usize>
    );

    pub fn get_start_pointer<T>(rest:&[T])->*const T{
        struct Repr<T>{
            ptr:*const T,
            _size:usize
        }
        let j:Repr<T>=unsafe{std::mem::transmute(rest)};
        j.ptr
    }
    impl Mover{
        pub fn new<'a:'b,'b,T:'a,I:Iterator<Item=&'b Cont<'a,T>>>(num_bots:usize,start_pointer:*const T,iter:I)->Mover{
            let mut move_vector=Vec::with_capacity(num_bots);    
                       
            #[inline]
            pub fn offset_to<T>(s: *const T, other: *const T) -> Option<isize> where T: Sized {
                 let size = std::mem::size_of::<T>();
                 if size == 0 {
                     None
                 } else {
                     let diff = (other as isize).wrapping_sub(s as isize);
                     Some(diff / size as isize)
                 }
            }

            for bot in iter {
                let target_ind:usize=offset_to(start_pointer,bot.a).unwrap() as usize;
                move_vector.push(target_ind);
            }

            Mover(move_vector)
        }

        pub fn finish<'a,T:'a,I:Iterator<Item=&'a T>>(&mut self,tree_bots:I,orig:&mut [T]){
            for (mov,b) in self.0.iter().zip(tree_bots){

                let cp=unsafe{orig.get_unchecked_mut(*mov)};

                unsafe{std::ptr::copy(b,cp,1)};
                    
                //*unsafe{orig.get_unchecked_mut(*mov)}=*b;
            }
        }
    }
}

impl<'a,A:AxisTrait,T:SweepTrait+Send+'a> Drop for DynTree<'a,A,T>{
    fn drop(&mut self){
        let orig=&mut self.orig;

        let i=NdIter{c:&self.tree.get_root()};

        let k=i.dfs_preorder_iter().flat_map(|a:&NodeDyn<T>|{
            a.range.iter()
        });

        self.mover.finish(k,orig);
    }
}


use self::alloc::DynTreeRaw;
mod alloc{
    use super::*;
    use std::mem::ManuallyDrop;
    use tree_alloc::TreeAllocDst;
    use tree_alloc::NodeDynBuilder; 

    pub struct DynTreeRaw<'a,T:SweepTrait+Send+'a>{
        height:usize,
        level:LevelDesc,
        alloc:ManuallyDrop<TreeAllocDst<'a,T>>,
        root:ManuallyDrop<NodeDstDynCont<'a,T>>
    }
    impl<'a,T:SweepTrait+'a+Send> Drop for DynTreeRaw<'a,T> {
        fn drop(&mut self) {
            unsafe {
                ManuallyDrop::drop(&mut self.root);
                ManuallyDrop::drop(&mut self.alloc);
            }
        }
    }

    impl<'a,T:SweepTrait+'a+Send> DynTreeRaw<'a,T>{
        pub fn new(tree:GenTree<Node2<Cont<T>>>,num_bots:usize)->DynTreeRaw<'a,T>{
            let height=tree.get_height();
            let level=tree.get_level_desc();
            let mut alloc=TreeAllocDst::new(tree.get_nodes().len(),num_bots);

            let root=Self::construct_flat_tree(&mut alloc,tree);    

            DynTreeRaw{height,level,alloc:ManuallyDrop::new(alloc),root:ManuallyDrop::new(root)}
        }
        pub fn get_level(&self)->LevelDesc{
            self.level
        }
        pub fn get_height(&self)->usize{
            self.height
        }
        pub fn get_root(&self)->&NodeDstDynCont<'a,T>{
            &self.root
        }
        pub fn get_root_mut(&mut self)->&mut NodeDstDynCont<'a,T>{
            &mut self.root
        }


        fn construct_flat_tree(
            alloc:&mut TreeAllocDst<'a,T>,
            tree:GenTree<Node2<Cont<T>>>
            )->NodeDstDynCont<'a,T>{

            let num_nodes=tree.get_nodes().len();
            let mut queue:Vec<NodeDstDynCont<'a,T>>=Vec::with_capacity(num_nodes);
            
            let mut v=tree.into_nodes();

            for node in v.drain(..){
                let Node2{divider,container_box,range}=node;
                let num_bots=range.len();
                let nn=NodeDynBuilder{divider,container_box,num_bots,range};
                let n=alloc.add(nn);
                queue.push(NodeDstDynCont(n));
            }

            assert!(alloc.is_full());

            assert_eq!(queue.len(),num_nodes);

            for i in (1..(num_nodes/2)+1).rev(){
                let c2=queue.pop().unwrap();
                let c1=queue.pop().unwrap();
                let j=2*i;
                let parent=(j-1)/2;
                queue[parent].0.c=Some((c1,c2)); 
            }

            assert_eq!(queue.len(),1);
            queue.pop().unwrap()
        }
    }
}

