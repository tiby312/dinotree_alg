
use std::cmp::Ordering;
use axgeom::Rect;

pub struct Counter(usize);

pub fn from_rect(counter:&mut Counter,rect:Rect<isize>)->Rect<DataNum>{
    let ((a,b),(c,d))=rect.get();
    Rect::new(counter.new_num(a),counter.new_num(b),counter.new_num(c),counter.new_num(d))
}


impl Counter{
    pub fn new()->Counter{
        Counter(0)
    }
    pub fn into_inner(self)->usize{
        self.0
    }
    pub fn get_inner(&self)->&usize{
        &self.0
    }
    pub fn reset(&mut self){
        self.0=0;
    }
    pub fn new_num(&mut self,a:isize)->DataNum{
        DataNum(a,self as *mut Counter)
    }
}

#[derive(Copy,Clone,Debug)]
pub struct DataNum(pub isize,*mut Counter);

//unsafe implement send and sync.
//we will be cause to only use sequential version of the tree algorithms
unsafe impl Send for DataNum{}
unsafe impl Sync for DataNum{}


impl PartialOrd for DataNum {
    fn partial_cmp(&self, other: &DataNum) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for DataNum {
    fn eq(&self, other: &DataNum) -> bool {
        self.0.cmp(&other.0)==Ordering::Equal
    }
}

impl Eq for DataNum {}
impl Ord for DataNum{
    fn cmp(&self, other: &DataNum) -> Ordering {

        unsafe{
            let p=self.1;
            (*p).0+=1;
        }
        self.0.cmp(&other.0)
    }
}

