use dinotree::prelude::*;


pub fn for_every_pair<T:HasAabb>(mut arr:ProtectedBBoxSlice<T>,mut func:impl FnMut(ProtectedBBox<T>,ProtectedBBox<T>)){
    loop{
        let temp=arr;
        match temp.split_first_mut(){
            Some((mut b1,mut x))=>{
                for mut b2 in x.as_mut().iter_mut(){
                    func(b1.as_mut(),b2.as_mut());
                }
                arr=x;
            },
            None=>break
        }
    }
}

