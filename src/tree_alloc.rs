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


/*
mod alloc{
    pub struct Alloc<'a>{
        v:Vec<u8>,
        counter:*mut u8,
        _p:PhantomData<&'a mut u8>
    }

    impl<'a> Alloc<'a>{
        pub fn new(alignment:usize, num_bytes:usize)->Alloc{
            let mut v=Vec::with_capacity(alignment+num_bytes);
            v.push(0);
            let k=&mut v[0] as *mut u8;
            Alloc{v:v,counter:k,_p:PhantomData}
        }

        pub fn alloc(&mut self, num_bytes:usize)->&'a mut [u8]{
            let k=unsafe{from_raw_parts(self.counter,num_bytes)};
            self.counter+=num_bytes;
            k
        }

    }
}
*/


pub struct TreeAllocDst<'a,T:SweepTrait+'a>{
    v:Vec<u8>,
    capacity:usize,
    counter:*mut u8,
    max_counter:*const u8,
    alignment:usize,
    node_size:usize,
    _p:PhantomData<(&'a mut NodeDstDyn<'a,T>)>
}

impl<'a,T:SweepTrait+'a> TreeAllocDst<'a,T>{   

    pub fn new(num_nodes:usize,num_bots:usize)->TreeAllocDst<'a,T>{
        let (alignment,node_size)=Self::compute_alignment_and_size();


        let c=node_size*num_nodes+std::mem::size_of::<T>()*num_bots;
        //TODO lower upper bound
        let cap=c*2;

        let mut v=Vec::with_capacity(cap);
        
        v.push(0);
        let counter=(&mut v[0]) as *mut u8;
        v.pop();
        let max_counter=unsafe{counter.offset(cap as isize)};

        //println!("node={:?}  bot={:?}",(alignment,node_size),(std::mem::align_of::<T>(),std::mem::size_of::<T>()));
        //nnnnxx--nnnnxxxxnnnnxxxxx---nnnn


        TreeAllocDst{v:v,capacity:cap,counter:counter,max_counter,_p:PhantomData,alignment,node_size}
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

        assert!((self.counter as usize) < (self.max_counter as usize),"{:?}",(self.counter,self.max_counter));
      
        let ll=self.counter;


        let dst={
            let dst:&mut NodeDstDyn<T>=unsafe{std::mem::transmute(ReprMut{ptr:ll,size:n.num_bots})};    
            dst.c=None;
            dst.n.divider=n.divider;
            dst.n.container_box=n.container_box;

            for (a,b) in dst.n.range.iter_mut().zip(n.i){
                *a=b;
            }
            dst
        };

        self.counter=unsafe{self.counter.offset(std::mem::size_of_val(dst) as isize)};

        //assert!(self.v.len()<=self.capacity);        
        dst
    }
    fn move_to_align_to(&mut self){
        for _ in 0..{
            self.counter=unsafe{self.counter.offset(1)};
            let k=self.counter as *const u8;
            if k as usize % self.alignment == 0{
                return;
            }
        }        
    }
}