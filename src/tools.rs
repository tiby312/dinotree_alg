use std;
use std::marker::PhantomData;


///A phantom data type that unsafely implements send,sync.
pub struct PhantomSendSync<T>(pub PhantomData<T>);
unsafe impl<T> Send for PhantomSendSync<T>{}
unsafe impl<T> Sync for PhantomSendSync<T>{}
impl<T> Copy for PhantomSendSync<T>{}
impl<T> Clone for PhantomSendSync<T>{
    fn clone(&self) -> PhantomSendSync<T> {
        *self
    }
}


pub mod par{
    use rayon;
    pub trait Joiner{

        fn join<A:FnOnce() -> RA + Send,RA:Send,B:FnOnce() -> RB + Send,RB:Send>(oper_a: A, oper_b: B) -> (RA, RB);
        fn is_parallel()->bool;
    }

    pub struct Parallel;
    impl Joiner for Parallel{
        fn is_parallel()->bool{
            return true;
        }

        fn join<A:FnOnce() -> RA + Send,RA:Send,B:FnOnce() -> RB + Send,RB:Send>(oper_a: A, oper_b: B) -> (RA, RB)   {
          rayon::join(oper_a, oper_b)
        }
    }
    pub struct Sequential;
    impl Joiner for Sequential{
        fn is_parallel()->bool{
            return false;
        }
        fn join<A:FnOnce() -> RA + Send,RA:Send,B:FnOnce() -> RB + Send,RB:Send>(oper_a: A, oper_b: B) -> (RA, RB)   {
            let a = oper_a();
            let b = oper_b();
            (a, b)
        }
    }
}


unsafe impl<T:Send> std::marker::Send for PreVec<T>{}

///An vec api to avoid excessive dynamic allocation by reusing a Vec
pub struct PreVec<T>{
    vec:Vec<* mut T>
}
impl<T> PreVec<T>{
	#[inline(always)]
    pub fn new()->PreVec<T>{
        PreVec{vec:Vec::new()}
    }

    #[inline(always)]
    pub fn with_capacity(size:usize)->PreVec<T>{
        PreVec{vec:Vec::with_capacity(size)}
    }

    ///Clears the vec and returns a mutable reference to a vec.
    #[inline(always)]
    pub fn get_empty_vec_mut<'a>(&'a mut self)->&mut Vec<&'a mut T>{
        self.vec.clear();
        let v:&mut Vec<*mut T> = &mut self.vec;
        unsafe{std::mem::transmute(v)}
    }
}




///Returns a combined slice given two slices that are next to each other in memory.
///Panics if they are not next to each other.
pub fn join_mut<'a,T>(first: &'a mut [T],second:&'a mut [T])->&'a mut[T]{
    let f1=first.len();
    if first[f1..].as_mut_ptr() == second.as_mut_ptr(){
        unsafe{
            return std::slice::from_raw_parts_mut(first.as_mut_ptr(),f1+second.len());
        }
    }else{
        panic!("Slices are not next to each other in memory.");
    }

}

///Returns a combined slice given two slices that are next to each other in memory.
///Panics if they are not next to each other.
pub fn join<'a,T>(first: &'a [T],second:&'a [T])->&'a [T]{
    let f1=first.len();
    if first[f1..].as_ptr() == second.as_ptr(){
        unsafe{
            return std::slice::from_raw_parts(first.as_ptr(),f1+second.len());
        }
    }else{
        panic!("Slices are not next to each other in memory.");
    }
}




pub mod log{

    
    use std::fs::File;
    use std::io::Write;
    


    pub struct Logger{
        file:File,
        counter:usize
    }
    impl Logger{

        pub fn new(str:&'static str)->Logger{
            
            let file = File::create(str).unwrap();
            Logger{file:file,counter:0}
        }

        pub fn with_names(str:&'static str,names:&[&'static str])->Logger{
            
            let mut file = File::create(str).unwrap();
           
            write!(file,"Iteration,").unwrap();
            for k in names{
                write!(file,"{},",k).unwrap();    
            }
            writeln!(file,"").unwrap();
            Logger{file:file,counter:0}
        }

        pub fn write_str(&mut self,strf:&'static str,slice:&[String]){

            write!(self.file,"{},",strf).unwrap();
            for k in slice{
                write!(self.file,"{},",k).unwrap();    
            }
            writeln!(self.file,"").unwrap();
            
        }
        pub fn write_data(&mut self,slice:&[f64]){
            
            write!(self.file,"{},",self.counter).unwrap();
            for k in slice{
                write!(self.file,"{},",k).unwrap();    
            }
            writeln!(self.file,"").unwrap();
            self.counter+=1;
            
        }
    }
}





use std::time::Instant;

pub struct Timer2{
    a:std::time::Instant
}

impl Timer2{
    pub fn new()->Timer2{
        Timer2{a:Instant::now()}
    }

    ///Returns the time since this object was created in seconds.
    pub fn elapsed(&self)->f64{
        let elapsed = self.a.elapsed();
        let sec = (elapsed.as_secs() as f64) + (elapsed.subsec_nanos() as f64 / 1000_000_000.0);
        sec
    }
}
