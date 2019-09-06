use dinotree::prelude::*;


pub fn for_every_pair<T:HasAabbMut>(mut arr:ElemSliceMut<T>,mut func:impl FnMut(BBoxRefMut<T::Num,T::Inner>,BBoxRefMut<T::Num,T::Inner>)){
    loop{
        let temp=arr;
        match temp.split_first_mut(){
            Some((mut b1,mut x))=>{
                for b2 in x.as_mut().iter_mut(){
                    func(b1.as_mut(),b2);
                }
                arr=x;
            },
            None=>break
        }
    }
}

