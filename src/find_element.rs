
//!
//! # Safety
//! 
//! There is no unsafe code in this module.
//!
use crate::inner_prelude::*;


///Represents the path taken at a node.
#[derive(Copy,Clone)]
pub enum Dir{
    Left,
    Right
}

///Used for debugging.
///Returns the debugging information of the first bot found that satisfies the predicate.
///Returns the depth, as well as the path down the tree taken to get to the node.
pub fn find_element<A:AxisTrait,T:HasAabb,F:FnMut(&T)->bool>(tree:DinoTreeRef<A,(),T>,mut func:F) -> Option<(usize,Vec<Dir>)>{
   fn recc<A:AxisTrait,T:HasAabb,F:FnMut(&T)->bool>(axis:A,func:&mut F,stuff:LevelIter<Vistr<(),T>>,trail:Vec<Dir>)->Option<(usize,Vec<Dir>)>{
        let ((depth,nn),rest)=stuff.next();

        for b in nn.range.iter(){
            if func(b){
                return Some((depth.0,trail));
            }
        }

        match rest{
            Some((_extra,left,right))=>{
                let mut tl=trail.clone();
                let mut tr=trail.clone();
                tl.push(Dir::Left);
                tr.push(Dir::Right);
                
                if let Some(ans)= recc(axis.next(),func,left,tl){
                    return Some(ans);
                }
                if let Some(ans) = recc(axis.next(),func,right,tr){
                    Some(ans)
                }else{
                    None
                }
                
            },
            None=>{
                None
            }
        }
   }
   recc(tree.axis(),&mut func,tree.vistr().with_depth(Depth(0)),Vec::new())
}
