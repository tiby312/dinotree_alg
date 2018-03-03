use super::*;
use std::marker::PhantomData;
use dyntree::Cont;


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

pub struct NodeDynBuilder<'a,'b:'a,T:SweepTrait+'b>{
    pub divider:T::Num,
    pub container_box:axgeom::Range<T::Num>,
    pub num_bots:usize,
    pub range:&'a [Cont<'b,T>]
}


pub struct TreeAllocDst<'a,T:SweepTrait+'a>{
    _vec:Vec<u8>,
    counter:*mut u8,
    max_counter:*const u8,
    _p:PhantomData<(&'a mut NodeDstDyn<'a,T>)>
}

impl<'a,T:SweepTrait+'a> TreeAllocDst<'a,T>{   

    pub fn new(num_nodes:usize,num_bots:usize)->TreeAllocDst<'a,T>{

        let (alignment,node_size)=Self::compute_alignment_and_size();

        let cap=node_size*num_nodes+std::mem::size_of::<T>()*num_bots;
        
        let (start_addr,vec)={

            let mut v=Vec::with_capacity(alignment+cap);
        
            v.push(0);
            let mut counter=(&mut v[0]) as *mut u8;
            v.pop();
            

            for _ in 0..alignment{
                let k=counter as *const u8;
                if k as usize % alignment == 0{
                    break;
                }
                counter=unsafe{counter.offset(1)};
            } 
            (counter,v)
        };

        let max_counter=unsafe{start_addr.offset(cap as isize)};

        TreeAllocDst{_vec:vec,counter:start_addr,max_counter,_p:PhantomData,}
    }


    fn compute_alignment_and_size()->(usize,usize){
        
        let (alignment,siz)={
            let k:&NodeDstDyn<T>=unsafe{
            //let mut vec:Vec<u8>=Vec::with_capacity(500);
            //vec.push(0);
            //let x:&[u8]= std::slice::from_raw_parts(&vec[0], 200+std::mem::size_of::<T>()); 
            //TODO safe to do this??????????????
            let k:*const u8=std::mem::transmute(0x10 as usize);//std::ptr::null::<T>();
            std::mem::transmute(Repr{ptr:k,size:0})
            };
            (std::mem::align_of_val(k),std::mem::size_of_val(k))
        };

        assert!(std::mem::size_of::<T>() % alignment==0);

        (alignment,siz)
    }
    pub fn is_full(&self)->bool{
        self.counter as *const u8== self.max_counter
    }
    pub fn add<'b,'c:'b>(&mut self,n:NodeDynBuilder<'b,'c,T>)->&'a mut NodeDstDyn<'a,T>{
        
        assert!((self.counter as *const u8) < self.max_counter);
    
        let ll=self.counter;


        let dst={
            let dst:&mut NodeDstDyn<T>=unsafe{std::mem::transmute(ReprMut{ptr:ll,size:n.num_bots})};    
            dst.c=None;
            dst.n.divider=n.divider;
            dst.n.container_box=n.container_box;

            for (a,b) in dst.n.range.iter_mut().zip(n.range){

                //we cant just move it into here.
                //then rust will try and call the destructor of the uninitialized object
                let k=unsafe{std::ptr::copy(b.a,a,1)};
                   
                //*a=b;
            }
            dst
        };

        self.counter=unsafe{self.counter.offset(std::mem::size_of_val(dst) as isize)};
       
        dst
    }
}