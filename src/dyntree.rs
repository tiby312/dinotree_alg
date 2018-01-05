use compt::CTreeIterator;
use median::MedianStrat;
use compt::LevelIter;
use axgeom::Range;
use compt::GenTree;
use compt::LevelDesc;
use base_kdtree::new_tree;
use tools::par;
use tree_alloc::NodeDstDynCont;
use tree_alloc::NodeDyn;
use axgeom::AxisTrait;
use std::marker::PhantomData;
use base_kdtree::Node2;
use TreeCache;
use super::*;

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






struct Cont<'b,T:'b+SweepTrait+Send>{
    a:&'b mut T
}

impl<'b,T:'b+SweepTrait+Send> SweepTrait for Cont<'b,T>{
    type Inner=T::Inner;
    type Num=T::Num;
    fn get_mut<'c>(&'c mut self)->(&'c axgeom::Rect<T::Num>,&'c mut Self::Inner){
        self.a.get_mut()
    }
    fn get<'c>(&'c self)->(&'c axgeom::Rect<T::Num>,&'c Self::Inner){
        self.a.get()
    }
}



pub struct DynTree<'b,A:AxisTrait,T:SweepTrait+Copy+Send+'b>{
    orig:&'b mut [T],
    tree:DynTreeRaw<'b,T>,
    //vector to where the bots should be put back to
    vec:Vec<usize>, //TODO use this
    _p:PhantomData<A>
}




use super::DynTreeTrait;
impl<'a,A:AxisTrait,T:SweepTrait+Copy+Send+'a> DynTreeTrait for DynTree<'a,A,T>{
   type T=T;
   type Num=T::Num;
    
   fn for_all_in_rect<F:FnMut(ColSingle<Self::T>)>(&mut self,rect:&axgeom::Rect<Self::Num>,fu:&mut F){
        colfind::for_all_in_rect(self,rect,fu);
   }
   
   fn for_every_col_pair_seq<H:DepthLevel,F:Bleek<T=Self::T>,K:TreeTimerTrait>
        (&mut self,clos:&mut F)->K::Bag{
       colfind::for_every_col_pair_seq::<A,T,H,F,K>(self,clos)
    
   }
   fn for_every_col_pair<H:DepthLevel,F:BleekSync<T=Self::T>,K:TreeTimerTrait>
        (&mut self,clos:&F)->K::Bag{
        colfind::for_every_col_pair::<A,T,H,F,K>(self,clos)
    }
}

impl<'a,A:AxisTrait,T:SweepTrait+Copy+Send+'a> DynTree<'a,A,T>{


    pub fn new<JJ:par::Joiner,H:DepthLevel,Z:MedianStrat<Num=T::Num>,K:TreeTimerTrait>(
        rest:&'a mut [T],tc:&mut TreeCache<A,T::Num>) -> (DynTree<'a,A,T>,K::Bag) {

        //let height=tc.get_tree().get_height()+1;

        let num_bots=rest.len();

        let bb=(&rest as &[T]) as *const [T];
        let bbr=&unsafe{&*bb}[0] as *const T;
               
        let (fb,move_vector,bag)={
            let mut pointers:Vec<Cont<T>>=Vec::with_capacity(rest.len());
            for (_,k) in (0..rest.len()).zip(rest.iter_mut()){
                pointers.push(Cont{a:k});
            }
            
            {
                let (mut tree2,bag)=self::new_tree::<A,JJ,_,H,Z,K>(&mut pointers,tc);

                // 12345
                // 42531     //vector:41302
                let mut move_vector=Vec::with_capacity(num_bots);    
                {
                    let t=tree2.get_tree().create_down();
                    t.dfs_preorder(|a:&Node2<Cont<T>>|{
                        for bot in a.range.iter(){
                            let target_ind:usize=bbr.offset_to(bot.a).unwrap() as usize;
                            move_vector.push(target_ind);

                        }
                    });
                }
                //let level=tree2.get_tree().get_level_desc();
                let fb=DynTreeRaw::new(tree2.into_tree(),num_bots);
                
                //let KdTree{total_slice,tree,tc}=tree2;
                (fb,move_vector,bag)
            }
        };
        //TODO PLUS ONE IMPORTANT!
        (DynTree{orig:rest,tree:fb,vec:move_vector,_p:PhantomData},bag)
    }
   
    pub fn get_height(&self)->usize{
        self.tree.get_height()
    }

    pub fn get_iter<'b>(&'b self)->NdIter<'a,'b,T>{
        NdIter{c:&self.tree.get_root()}
    }
    pub fn get_level_desc(&self)->LevelDesc{
        self.tree.get_level()
    }
    pub fn get_iter_mut<'b>(&'b mut self)->NdIterMut<'a,'b,T>{
        NdIterMut{c:self.tree.get_root_mut()}
    }

}


impl<'a,A:AxisTrait,T:SweepTrait+Copy+Send+'a> Drop for DynTree<'a,A,T>{
    fn drop(&mut self){
        let mut move_iter=self.vec.iter();
        let orig=&mut self.orig;

        let i=NdIter{c:&self.tree.get_root()};
        i.dfs_preorder(|a:&NodeDyn<T>|{
            for b in a.range.iter(){
                let i=move_iter.next().unwrap();

                //TODO do in unsafe block hid by a module
                orig[*i]=*b;
            }
        });
    }
}


use self::alloc::DynTreeRaw;
mod alloc{
    use super::*;
    use std::mem::ManuallyDrop;
    use tree_alloc::TreeAllocDst;
    use tree_alloc::NodeDynBuilder; 
    use tree_alloc::NodeDstDyn;


    pub struct DynTreeRaw<'a,T:SweepTrait+Send+Copy+'a>{
        height:usize,
        level:LevelDesc,
        alloc:ManuallyDrop<TreeAllocDst<'a,T>>,
        root:ManuallyDrop<NodeDstDynCont<'a,T>>
    }
    impl<'a,T:SweepTrait+'a+Send+Copy> Drop for DynTreeRaw<'a,T> {
        fn drop(&mut self) {
            unsafe {
                ManuallyDrop::drop(&mut self.root);
                ManuallyDrop::drop(&mut self.alloc);
            }
        }
    }

    impl<'a,T:SweepTrait+'a+Send+Copy> DynTreeRaw<'a,T>{
        pub(super) fn new(tree:GenTree<Node2<Cont<T>>>,num_bots:usize)->DynTreeRaw<'a,T>{
            let height=tree.get_height();
            let level=tree.get_level_desc();
            let mut alloc=TreeAllocDst::new();

            alloc.allocate(tree.get_nodes().len(),num_bots);

            let root=Self::construct_flat_tree(&mut alloc,tree);    

            DynTreeRaw{height,level,alloc:ManuallyDrop::new(alloc),root:ManuallyDrop::new(root)}
        }
        pub fn get_level(&self)->LevelDesc{
            self.level
        }
        pub fn get_height(&self)->usize{
            self.height
        }
        pub(super) fn get_root(&self)->&NodeDstDynCont<'a,T>{
            &self.root
        }
        pub(super) fn get_root_mut(&mut self)->&mut NodeDstDynCont<'a,T>{
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
                //let n=Self::add_node(alloc,divider,container_box,range.iter().map(|c:&Cont<T>|{*c.a}),num_bots);
                let nn=NodeDynBuilder{divider,container_box,num_bots,i:range.iter().map(|c:&Cont<T>|{*c.a})};
                let n=alloc.add(nn);
                queue.push(NodeDstDynCont(n));
            }

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


#[test]
fn testy(){
    use kdtree::median::MedianStrict;
    use compt::LevelDesc;
    use ordered_float::NotNaN;
    use test_support;
    use super::*;
    use std::sync::Mutex;
    use test_support::Bot;
    use support::BBox;
    use test::Bencher;
    use test::black_box;
    use extensions::Rects;
    use kdtree::base_kdtree;
    use tools::par;
    use support::Numf32;
    let world=test_support::create_word();
    let mut vecc=black_box(test_support::create_bots(&world,300,&[5,1,3,6,1,8]));

    let copy=vecc.clone();
    {
        let mut treecache=TreeCache::new::<par::Parallel>(axgeom::XAXIS,4);

        {

            let mut fl=dyntree::DynTreeRaw::new();
            let mut dyntree=fl.new_tree::<par::Parallel,DefaultDepthLevel,MedianStrict<Numf32>>
                        (&mut vecc,&mut treecache);

            use oned::sup::BleekBF;
            //let mut bb=BleekBF::new(&clos);
                    
            //colfind::for_every_col_pair::<_,DefaultDepthLevel,_>(&mut dyntree,&bb,&mut t);
        }   

        for (a,i) in copy.iter().zip(vecc.iter()){
            assert_eq!(a.get().1.id,i.get().1.id);
        }               
        //println!("align x={}",std::mem::align_of::<NodeDst<BBox<Numf32,Bot>>>());
        //println!("align y={}",std::mem::align_of::<BBox<Numf32,Bot>>());
        //println!("len={}  withoutpad={}",DynTreeRaw.alloc.len(),packed_len);        
        
    }

    let _v=black_box(&mut vecc);
}