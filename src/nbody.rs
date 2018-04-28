use inner_prelude::*;
use super::*;

//TODO somehow take advantage of sorted property?????

mod tools{
    pub fn for_every_pair<T,F:FnMut(&mut T,&mut T)>(arr:&mut [T],mut func:F){
        unsafe{
            for x in 0..arr.len(){
                let xx=arr.get_unchecked_mut(x) as *mut T;
                for j in (x+1)..arr.len(){
                    
                    let j=arr.get_unchecked_mut(j);
                    let xx=unsafe{&mut*xx};
                    func(xx,j);
                }
            }
        }
    }
    pub fn for_bijective_pair<T,F:FnMut(&mut T,&mut T)>(arr1:&mut [T],arr2:&mut [T],mut func:F){
        for x in arr1.iter_mut(){
            for j in arr2.iter_mut(){
                func(x,j);
            }
        }
    }
}


pub trait NodeMassTrait:Send{
    type T:SweepTrait;
    fn handle_with(&self,b:&mut Self);
    fn handle_bot(&mut Self::T,&mut Self::T);
    fn new(rect:Rect<<Self::T as SweepTrait>::Num>,b:&[Self::T])->Self;
    fn increase_mass(&mut self,b:&[Self::T]);
    fn apply(&mut self,b:&mut Self::T);
    fn is_far_enough(&self,b:&Rect<<Self::T as SweepTrait>::Num>)->bool;
    fn get_box(&self)->&Rect<<Self::T as SweepTrait>::Num>;
}




//pseudo code
//build up a tree where every nodemass has the mass of all the bots in that node and all the bots under it.
fn buildtree<'a,
    A:AxisTrait,
    T:SweepTrait+'a,
    N:NodeMassTrait<T=T>
    >
    (tree:&DynTree<A,T>,rect:Rect<T::Num>)->compt::dfs::GenTreeDfsOrder<N>{


    fn recc<'a,A:AxisTrait,T:SweepTrait+'a,N:NodeMassTrait<T=T>>
        (axis:A,stuff:NdIter<T>,rect:Rect<T::Num>,vec:&mut Vec<N>){

        let (nn,rest)=stuff.next();

        let mut nodeb=N::new(rect,&nn.range);

        match rest{
            Some((mut left,mut righ))=>{
                let div=match nn.div{
                    Some(div)=>{div},
                    None=>{return}
                };

                let (leftr,rightr)=rect.subdivide(div,A::get());
                
                {
                    let left=left.create_wrap();
                    let righ=righ.create_wrap();

                    recc2(&mut nodeb,left);
                    recc2(&mut nodeb,righ);
                }

                recc(axis.next(),left,leftr,vec);
                
                vec.push(nodeb);
                
                recc(axis.next(),righ,rightr,vec);
            },
            None=>{

            }
        }

        fn recc2<'a,T:SweepTrait+'a,N:NodeMassTrait<T=T>>(nodeb:&mut N,stuff:NdIter<T>){
            let (nn,rest)=stuff.next();

            nodeb.increase_mass(&nn.range);

            match rest{
                Some((left,right))=>{
                    recc2(nodeb,left);
                    recc2(nodeb,right);
                },
                None=>{

                }
            }
        }

    }


    let mut vec=Vec::new();
    let height=tree.get_height();
    let stuff=tree.get_iter();
    recc(A::new(),stuff,rect,&mut vec);

    compt::dfs::GenTreeDfsOrder::from_vec(vec,height).unwrap()

}



//Construct anchor from cont!!!
struct Anchor<'a,N:NodeMassTrait+'a>{
    mass:N,
    node:&'a mut NodeDyn<N::T>
}

fn handle_anchor_with_children<'a,
    T:SweepTrait+'a,
    N:NodeMassTrait<T=T>+'a,
    C:CTreeIterator<Item=(&'a mut N,&'a mut NodeDyn<T>)>>
(anchor:&mut Anchor<N>,left:C,right:C){
    {
        recc2(anchor,left);
        recc2(anchor,right);
    }

    fn recc2<'a,T:SweepTrait+'a,N:NodeMassTrait<T=T>+'a,C:CTreeIterator<Item=(&'a mut N,&'a mut NodeDyn<T>)>>(anchor:&mut Anchor<N>,stuff:C){
        let (mut nn,rest)=stuff.next();
        

        if anchor.mass.is_far_enough(nn.0.get_box()){
            anchor.mass.handle_with(nn.0);
            return;
        }

        match rest{
            Some((left,right))=>{
                recc2(anchor,left);
                recc2(anchor,right);
            },
            None=>{

                for b in anchor.node.range.iter_mut(){
                    for b2 in nn.1.range.iter_mut(){
                        N::handle_bot(b,b2);
                    }
                }
            }
        }
    }
}

fn handle_left_with_right<'a,
    T:SweepTrait+'a,
    N:NodeMassTrait<T=T>+'a,
    C:CTreeIterator<Item=(&'a mut N,&'a mut NodeDyn<T>)>>
    (left:C,right:C,left_rect:&Rect<T::Num>,right_rect:&Rect<T::Num>){
    
    let (mut left_mass,mut left_bots)={
    
        let mut left_mass=Vec::new();
        let mut left_bots=Vec::new();
        recc3(left,&mut left_mass,&mut left_bots,right_rect);
        (left_mass,left_bots)
    };

    let (mut right_mass,mut right_bots)={

        let mut right_mass=Vec::new();
        let mut right_bots=Vec::new();
        recc3(right,&mut right_mass,&mut right_bots,left_rect);
        (right_mass,right_bots)
    };

    //handle the mass pairs
    for i in left_mass.iter_mut(){
        for j in right_mass.iter_mut(){
            i.handle_with(j);
        }
    }
    //handle the bot pairs
    for i in left_bots.iter_mut(){
        for j in right_bots.iter_mut(){
            N::handle_bot(i,j);
        }
    }

    //handle the mass/bot pairs.
    for i in left_mass.iter_mut(){
        for j in right_bots.iter_mut(){
            i.apply(j);
        }
    }
    for i in right_mass.iter_mut(){
        for j in left_bots.iter_mut(){
            i.apply(j);
        }
    }








    fn recc3<'a:'b,'b,
        T:SweepTrait+'a,
        N:NodeMassTrait<T=T>+'a,
        C:CTreeIterator<Item=(&'a mut N,&'a mut NodeDyn<T>)>>
    (mut stuff:C,rects:&mut Vec<&'b mut N>,bots:&mut Vec<&'b mut T>,other:&Rect<T::Num>)
    {
        let (nn,rest)=stuff.next();
        
        if nn.0.is_far_enough(other){
            rects.push(nn.0);
            return;
        }

        match rest{
            Some((left,right))=>{
                recc3(left,rects,bots,other);
                recc3(right,rects,bots,other);
            },
            None=>{
                for i in nn.1.range.iter_mut(){
                    bots.push(i)
                }
            }
        }
        
    }
}



pub fn nbody_seq<A:AxisTrait,T:SweepTrait,N:NodeMassTrait<T=T>>(tree:&mut DynTree<A,T>,rect:&Rect<T::Num>){

   
    fn recc<A:AxisTrait,T:SweepTrait,N:NodeMassTrait<T=T>>(axis:A,it1:NdIterMut<T>,it2:compt::dfs::DownTMut<N>,rect:Rect<T::Num>){
        let (nn1,rest1)=it1.next();
        let (nn2,rest2)=it2.next();

        //handle bots in itself
        tools::for_every_pair(&mut nn1.range,|a,b|{N::handle_bot(a,b)});
        

        match rest1{
            Some((mut left,mut right))=>{
                let div=match nn1.div{
                    Some(div)=>{div},
                    None=>{return;}
                };

                let (mut left2,mut right2)=rest2.unwrap();

                {
                    let l1=left2.create_wrap_mut().zip(left.create_wrap_mut());
                    let l2=right2.create_wrap_mut().zip(right.create_wrap_mut());

                    let mut anchor={
                        //Create a new mass that is only the rect of that contains all the bots intersecting the divider.
                        //let rect=rect.constrain_by(nn1.cont); //TODO
                        let rect=rect;
                        let m=N::new(rect,&nn1.range);
                        Anchor{mass:m,node:nn1}
                    };

                    handle_anchor_with_children(&mut anchor,l1,l2);
                }
                //At this point, everything has been handled with the root.
                //before we can fully remove the root, and reduce this problem to two smaller trees,
                //we have to do one more thing.
                //we have to handle all the bots on the left of the root with all the bots on the right of the root.

                //from the left side,get a list of nodemases.
                //from the right side,get a list of nodemases.
                //collide the two.
                {
                    let (left_rect,right_rect)={
                        let l=*left2.create_wrap_mut().next().0.get_box();
                        let r=*right2.create_wrap_mut().next().0.get_box();
                        (l,r)
                    };

                    let l1=left2.create_wrap_mut().zip(left.create_wrap_mut());
                    let l2=right2.create_wrap_mut().zip(right.create_wrap_mut());
                    handle_left_with_right(l1,l2,&left_rect,&right_rect);
                }
                //at this point we have successfully broken up this problem
                //into two independant ones, and we can do this all over again for the two children.
                //potentially in parlalel.
                //TODO parallelize.
                let (rect1,rect2)=rect.subdivide(div,A::get());
                recc(axis.next(),left,left2,rect1);
                recc(axis.next(),right,right2,rect2);
            },
            None=>{

            }
        }
    }
     //tree containing the nodemass of each node (and decendants)
    let mut tree2=buildtree::<_,_,N>(tree,*rect);

    let it1=tree.get_iter_mut();
    let it2=tree2.create_down_mut();

    recc(A::new(),it1,it2,*rect);

}
