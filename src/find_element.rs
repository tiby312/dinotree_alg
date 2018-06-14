
/*
pub fn find_element<A:AxisTrait,T:SweepTrait,F:FnMut(&T)->bool>(tree:&DynTreeMut<A,(),T>,mut func:F)->Option<(usize,Vec<bool>)>{
       fn recc<'a,A:AxisTrait,T:SweepTrait+'a,F:FnMut(&T)->bool,C:CTreeIterator<Item=(Depth,&'a NodeDyn<(),T>)>>(axis:A,func:&mut F,stuff:C,trail:Vec<bool>)->Option<(usize,Vec<bool>)>{
            let ((depth,nn),rest)=stuff.next();

            for b in nn.range.iter(){
                if func(b){
                    return Some((depth.0,trail));
                }
            }

            match rest{
                Some((left,right))=>{
                    let mut tl=trail.clone();
                    let mut tr=trail.clone();
                    tl.push(true);
                    tr.push(false);
                    
                    match recc(axis.next(),func,left,tl){
                        Some(ans)=>return Some(ans),
                        None=>{}
                    }
                    match recc(axis.next(),func,right,tr){
                        Some(ans)=>return Some(ans),
                        None=>return None
                    }
                    
                },
                None=>{
                    return None
                }
            }


       }

    recc(A::new(),&mut func,tree.get_iter().with_depth(Depth(0)),Vec::new())
}
*/