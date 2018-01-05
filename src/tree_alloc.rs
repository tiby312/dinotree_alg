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
        
        TreeAllocDst{v:Vec::new(),capacity:0,_p:PhantomData,alignment,node_size}
    }
    pub fn allocate(&mut self,numt1:usize,numt2:usize){
        let c=self.node_size*numt1+std::mem::size_of::<T>()*numt2;

        //TODO lower upper bound
        let cap=c*2;
        //let v=Vec::with_capacity(cap);
        self.v.reserve(cap);
        self.capacity=cap;
    }

    fn compute_alignment_and_size()->(usize,usize){
         //TODO fix this
        let k:&NodeDstDyn<T>=unsafe{
            let mut vec:Vec<u8>=Vec::with_capacity(500);
            vec.push(0);
            let x:&[u8]= std::slice::from_raw_parts(&vec[0], 200+std::mem::size_of::<T>()); 
            //let fake = x as *const NodeDyn<T>;
            std::mem::transmute(Repr{ptr:&x[0],size:0})
        };
        //8
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
        //let align=std::mem::align_of_val(v);

        for _ in 0..{
            self.v.push(0);
            let k=self.v.last().unwrap() as *const u8;
            if k as usize % self.alignment == 0{
                self.v.pop();
                //println!("moved up ={}",counter);
                return;
            }
        }        
    }
}


/*
pub struct TreeAlloc<'a,T1,T2:Copy>{
    v:Vec<u8>,
    capacity:usize,
    _p:PhantomData<(&'a mut u8,T1,T2)>
}

impl<'a,T1,T2:Copy> TreeAlloc<'a,T1,T2>{
    pub fn len(&self)->usize{
        self.v.len()
    }

    pub fn compute_packed_len(num1:usize,num2:usize)->usize{
        num1*std::mem::size_of::<T1>()+num2*std::mem::size_of::<T2>()
    }       

    pub fn new()->TreeAlloc<'a,T1,T2>{
        TreeAlloc{v:Vec::new(),capacity:0,_p:PhantomData}
    }
    pub fn allocate(&mut self,numt1:usize,numt2:usize){
        let c=Self::compute_packed_len(numt1,numt2);
        //TODO lower upper bound
        let cap=c+100;
        self.v.reserve(cap);
        self.capacity=cap;
    }
    pub fn add(&mut self,a:T1)->&'a mut T1{
        self.move_to_align_to::<T1>();

        let start=self.v.len();
        self.v.extend_from_slice(any_as_u8_slice(&a));

        //println!("moved up node");
        assert!(self.v.len()<=self.capacity);
        let zz=&mut self.v[start];
        unsafe{std::mem::transmute(zz)}
    }

    pub fn add_iter<I:Iterator<Item=T2>>(&mut self,a:I,num_bots:usize)->&'a mut [T2]{
        if num_bots==0{
            return &mut []
        }

        self.move_to_align_to::<T2>();

        let start=self.v.len();
        

        for i in a{
            self.v.extend_from_slice(any_as_u8_slice(&i));
        }
        //println!("moved up={} bots",num_bots);
        assert!(self.v.len()<=self.capacity);
        
        let pp:&mut T2=unsafe{std::mem::transmute(&mut self.v[start])};

        unsafe{
            std::slice::from_raw_parts_mut(
                ( pp as *mut  T2),num_bots,
            )
        }
    
        
    }

    fn move_to_align_to<T>(&mut self){
        let align=std::mem::align_of::<T>();

        for _ in 0..{
            self.v.push(0);
            let k=self.v.last().unwrap() as *const u8;
            if k as usize % align == 0{
                self.v.pop();
                //println!("moved up ={}",counter);
                return;
            }
        }        
    }

    
}
*/

/*
fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    unsafe{
        std::slice::from_raw_parts(
            (p as *const T) as *const u8,
            std::mem::size_of::<T>(),
        )
    }
}
*/