use super::*;
use std::marker::PhantomData;


#[repr(C)]
struct ReprMut<T>{
    ptr:*mut T,
    size:usize,
}

#[repr(C)]
struct Repr<T>{
    ptr:*const T,
    size:usize,
}



pub struct NodeDyn<T:SweepTrait>{ 

    pub divider:T::Num,

    //only valid if the node has bots in it.
    pub container_box:axgeom::Range<T::Num>,

    pub range:[T]
}



pub struct NodeDstDynCont<'a,T:SweepTrait+'a>(
    pub &'a mut NodeDstDyn<'a,T> 
    );

pub struct NodeDstDyn<'a,T:SweepTrait+'a>{
    pub c:Option<(NodeDstDynCont<'a,T>,NodeDstDynCont<'a,T>)>,
    pub n:NodeDyn<T>
}

impl<T:SweepTrait> NodeDyn<T>{
 
    pub fn divider(&self)->&T::Num{
        &self.divider
    }


    pub fn get_container(&self)->&axgeom::Range<T::Num>{
        &self.container_box
    }
    pub fn get_bots(&mut self)->&mut [T]{
        &mut self.range
    }
}

pub struct NodeDynBuilder<T:SweepTrait,I:Iterator<Item=T>>{
    pub divider:T::Num,
    pub container_box:axgeom::Range<T::Num>,
    pub num_bots:usize,
    pub i:I
}

pub struct TreeAllocDst<'a,T:SweepTrait+'a>{
    v:Vec<u8>,
    capacity:usize,
    alignment:usize,
    node_size:usize,
    _p:PhantomData<(&'a mut NodeDstDyn<'a,T>)>
}

impl<'a,T:SweepTrait+'a> TreeAllocDst<'a,T>{   

    pub fn new()->TreeAllocDst<'a,T>{
        let (alignment,node_size)=Self::compute_alignment_and_size();


        //println!("node={:?}  bot={:?}",(alignment,node_size),(std::mem::align_of::<T>(),std::mem::size_of::<T>()));
        //nnnnxx--nnnnxxxxnnnnxxxxx---nnnn


        TreeAllocDst{v:Vec::new(),capacity:0,_p:PhantomData,alignment,node_size}
    }
    pub fn allocate(&mut self,numt1:usize,numt2:usize){
        

        let c=self.node_size*numt1+std::mem::size_of::<T>()*numt2;

        //TODO lower upper bound
        let cap=c*2;
        self.v.reserve(cap);
        self.capacity=cap;
    }

    fn compute_alignment_and_size()->(usize,usize){
         //TODO fix this
        let k:&NodeDstDyn<T>=unsafe{
            let mut vec:Vec<u8>=Vec::with_capacity(500);
            vec.push(0);
            let x:&[u8]= std::slice::from_raw_parts(&vec[0], 200+std::mem::size_of::<T>()); 
            std::mem::transmute(Repr{ptr:&x[0],size:0})
        };
        (std::mem::align_of_val(k),std::mem::size_of_val(k))
    }

    pub fn add<I:Iterator<Item=T>>(&mut self,n:NodeDynBuilder<T,I>)->&'a mut NodeDstDyn<'a,T>{
        self.move_to_align_to();

        let start=self.v.len();
        self.v.push(0);
        let ll=&mut self.v[start] as *mut u8;
        self.v.pop();


        let dst:&mut NodeDstDyn<T>=unsafe{std::mem::transmute(ReprMut{ptr:ll,size:n.num_bots})};
        
        for _ in 0..std::mem::size_of_val(dst){
            self.v.push(0)
        }
        
        dst.c=None;
        dst.n.divider=n.divider;
        dst.n.container_box=n.container_box;

        assert!(dst.n.range.len()==n.num_bots);

        for (a,b) in dst.n.range.iter_mut().zip(n.i){
            *a=b;
        }

        assert!(self.v.len()<=self.capacity);        
        dst
    }
    fn move_to_align_to(&mut self){
        
        for _ in 0..{
            self.v.push(0);
            let k=self.v.last().unwrap() as *const u8;
            if k as usize % self.alignment == 0{
                self.v.pop();
                return;
            }
        }        
    }
}