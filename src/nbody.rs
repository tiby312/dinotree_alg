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

    //gravitate this nodemass with another node mass
    fn handle_with(&mut self,b:&mut Self);

    //gravitate a bot with a bot
    fn handle_bot(&mut Self::T,&mut Self::T);

  
    //increase the mass of a node mass by adding the masses of all the bots in the list
    //fn increase_mass(&mut self,b:&[Self::T]);

    //gravitate a nodemass with a bot
    fn apply(&mut self,b:&mut Self::T);

    //check if the rect is far enough away from the nodemass.
    //if it is we will use this nodemass instead of gravitating all the bots
    //fn is_far_enough(a:(&Self,&Rect<<Self::T as SweepTrait>::Num>),b:(&Self,&Rect<<Self::T as SweepTrait>::Num>))->bool;
    fn is_far_enough(a:&CenterOfMass<<Self::T as SweepTrait>::Num>,b:&CenterOfMass<<Self::T as SweepTrait>::Num>)->bool;


    fn center_of_mass(&self)->[<Self::T as SweepTrait>::Num;2];
    //get the bounding box of this nodemass
    //TODO get rid of this one
    //fn get_box(&self)->Rect<<Self::T as SweepTrait>::Num>;

    //apply the forces that this node mass has to all of the bots in it.
    //fn undo(&self,b:&mut [Self::T]);
    //returns all bots inside of this nodemass.
    fn undo<'a,I:Iterator<Item=&'a mut Self::T>> (&self,it:I,len:usize) where Self::T:'a;

      //create a new node mass that has the combined mass of all the bots in the slice
    //fn new(rect:Rect<<Self::T as SweepTrait>::Num>,b:&[Self::T])->Self;
    fn new<'a,I:Iterator<Item=&'a Self::T>> (it:I,len:usize)->Self where Self::T:'a;

}



struct NodeMassWrapper<N:NodeMassTrait>{
    nm:N,
    rect:Rect<<N::T as SweepTrait>::Num>
}

impl<N:NodeMassTrait> NodeMassWrapper<N>{
    fn create_center(&self)->CenterOfMass<<N::T as SweepTrait>::Num>{
        CenterOfMass{rect:self.rect,center:self.nm.center_of_mass()}
    }
}

pub struct CenterOfMass<N:NumTrait>{
    pub rect:Rect<N>,
    pub center:[N;2]
}



//pseudo code
//build up a tree where every nodemass has the mass of all the bots in that node and all the bots under it.
fn buildtree<'a,
    A:AxisTrait,
    T:SweepTrait+'a,
    N:NodeMassTrait<T=T>
    >
    (tree:&DynTree<A,T>,rect:Rect<T::Num>)->compt::dfs::GenTreeDfsOrder<NodeMassWrapper<N>>{


    fn recc<'a,A:AxisTrait,T:SweepTrait+'a,N:NodeMassTrait<T=T>>
        (axis:A,stuff:NdIter<T>,rect:Rect<T::Num>,vec:&mut Vec<NodeMassWrapper<N>>){

        let (nn,rest)=stuff.next();


        match rest{
            Some((mut left,mut righ))=>{
                

                match nn.div{
                    Some(div)=>{
                        
                        
                        let nodeb={
                            //We know this vec will atleast have the size of the number of bots in this node.
                            let mut bots_to_add:Vec<&T>=Vec::with_capacity(nn.range.len());
                            for i in nn.range.iter(){
                                bots_to_add.push(i);
                            }
                            
                            let left=left.create_wrap();
                            let righ=righ.create_wrap();

                            recc2(&mut bots_to_add,left);
                            recc2(&mut bots_to_add,righ);
                            let len=bots_to_add.len();
                            let mut nodeb=NodeMassWrapper{nm:N::new(bots_to_add.drain(..),len),rect};
                            nodeb
                        };

                        let (leftr,rightr)=rect.subdivide(div,A::get());
                   
                        recc(axis.next(),left,leftr,vec);
                        
                        vec.push(nodeb);
                        
                        recc(axis.next(),righ,rightr,vec);
                    },
                    None=>{
                        //recurse anyway even though there is no divider.
                        //we want to populate this tree entirely.
                       recc(axis.next(),left,rect,vec);
                        
                        let mut nodeb=NodeMassWrapper{nm:N::new(nn.range.iter(),nn.range.len()),rect};
                        vec.push(nodeb);
                        
                        recc(axis.next(),righ,rect,vec); 
                    }
                }
            },
            None=>{
                let mut nodeb=NodeMassWrapper{nm:N::new(nn.range.iter(),nn.range.len()),rect};
                vec.push(nodeb);
            }
        }

        fn recc2<'a,T:SweepTrait+'a>(nodeb:&mut Vec<&'a T>,stuff:NdIter<'a,T>){
            let (nn,rest)=stuff.next();

            for i in nn.range.iter(){
                nodeb.push(i);
            }
            //nodeb.increase_mass(&nn.range);

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

    //TODO with capcaity
    let mut vec=Vec::new();
    let height=tree.get_height();
    let stuff=tree.get_iter();
    recc(A::new(),stuff,rect,&mut vec);


    let len=vec.len();
    match compt::dfs::GenTreeDfsOrder::from_vec(vec,height){
        Ok(a)=>a,
        Err(e)=>{
            panic!("vec size={:?}",len);
        }
    }

}

fn apply_tree<'a,
    A:AxisTrait,
    T:SweepTrait+'a,
    N:NodeMassTrait<T=T>
    >
    (tree:&mut DynTree<A,T>,tree2:compt::dfs::GenTreeDfsOrder<NodeMassWrapper<N>>){

    fn recc<'a,T:SweepTrait+'a,N:NodeMassTrait<T=T>>
        (mut stuff:NdIterMut<T>,stuff2:compt::dfs::DownT<NodeMassWrapper<N>>){

        let (nn1,rest)=stuff.next();
        let (nodeb,rest2)=stuff2.next();
        


        //nodeb.undo(&mut nn1.range);


        match rest{
            Some((mut left,mut righ))=>{
                let (left2,right2)=rest2.unwrap();

                let div=match nn1.div{
                    Some(div)=>{div},
                    None=>{return;}
                };

                
                {
                    let mut bots_to_undo:Vec<&mut T>=Vec::new();

                    let left=left.create_wrap_mut();
                    let righ=righ.create_wrap_mut();

                    recc2(&mut bots_to_undo,left);
                    recc2(&mut bots_to_undo,righ);

                    let l=bots_to_undo.len();
                    nodeb.nm.undo(bots_to_undo.drain(..),l);
                }

                recc(left,left2);
                recc(righ,right2);
            },
            None=>{
                let l=nn1.range.len();
                nodeb.nm.undo(nn1.range.iter_mut(),l);
                //nodeb.undo()
            }
        }

        fn recc2<'a,T:SweepTrait+'a>(bots:&mut Vec<&'a mut T>,stuff:NdIterMut<'a,T>){
            let (nn,rest)=stuff.next();

            match rest{
                Some((left,right))=>{
                    match nn.div{
                        Some(div)=>{
                            for i in nn.range.iter_mut(){
                                bots.push(i);
                            }
                            recc2(bots,left);
                            recc2(bots,right);
                        },
                        None=>{
                            return;
                        }
                    }
                    
                },
                None=>{
                    for i in nn.range.iter_mut(){
                        bots.push(i);
                    }
                }
            }
        }
    }


    let stuff=tree.get_iter_mut();
    let stuff2=tree2.create_down();
    recc(stuff,stuff2);


}


//Construct anchor from cont!!!
struct Anchor<'a,N:NodeMassTrait+'a>{
    mass:NodeMassWrapper<N>,
    node:&'a mut NodeDyn<N::T>
}

fn handle_anchor_with_children<'a,
    T:SweepTrait+'a,
    N:NodeMassTrait<T=T>+'a,
    C:CTreeIterator<Item=(&'a mut NodeMassWrapper<N>,&'a mut NodeDyn<T>)>>
(anchor:&mut Anchor<N>,left:C,right:C){
    {
        recc2(anchor,left);
        recc2(anchor,right);
    }

    fn recc2<'a,T:SweepTrait+'a,N:NodeMassTrait<T=T>+'a,C:CTreeIterator<Item=(&'a mut NodeMassWrapper<N>,&'a mut NodeDyn<T>)>>(anchor:&mut Anchor<N>,stuff:C){
        let (mut nn,rest)=stuff.next();
        

        {
            if N::is_far_enough(&anchor.mass.create_center(),&nn.0.create_center()){
                anchor.mass.nm.handle_with(&mut nn.0.nm);
                return;
            }
        }


        for b in anchor.node.range.iter_mut(){
            for b2 in nn.1.range.iter_mut(){
                N::handle_bot(b,b2);
            }
        }
        
        match rest{
            Some((left,right))=>{
                recc2(anchor,left);
                recc2(anchor,right);
            },
            None=>{

            }
        }
    }
}


fn handle_left_with_right<'a,
    T:SweepTrait+'a,
    N:NodeMassTrait<T=T>+'a,
    C:CTreeIterator<Item=(&'a mut NodeMassWrapper<N>,&'a mut NodeDyn<T>)>>
    (left:C,right:C,left_rect:&CenterOfMass<T::Num>,right_rect:&CenterOfMass<T::Num>){
    
        /*
    //let (left_anchor,left_rest)=left.next();
    //let (right_anchor,right_rest)=right.next();

    //handle_a_node(left_anchor,&mut left_mass,&mut left_bots,&right_anchor.0);
    //handle_a_node(right_anchor,&mut right_mass,&mut right_bots,&left_anchor.0);
    if N::is_far_enough((&left_anchor.0.nm,&left_anchor.0.rect),(&right_anchor.0.nm,&right_anchor.0.rect)){
        left_anchor.0.nm.handle_with(&mut right_anchor.0.nm);
    }else{
        for i in left_anchor.1.range.iter_mut(){
            for j in right_anchor.1.range.iter_mut(){
                N::handle_bot(i,j);
            }
        }
    }
    */

    
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
            i.nm.handle_with(&mut j.nm);
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
            i.nm.apply(j);
        }
    }
    for i in right_mass.iter_mut(){
        for j in left_bots.iter_mut(){
            i.nm.apply(j);
        }
    }

    

    /*
    fn handle_a_node<'a:'b,'b,
        T:SweepTrait+'a,
        N:NodeMassTrait<T=T>+'a>
        (nn:(&'a mut NodeMassWrapper<N>,&'a mut NodeDyn<T>),nms:&mut Vec<&'b mut NodeMassWrapper<N>>,bots:&mut Vec<&'b mut T>,other:&NodeMassWrapper<N>)
    {
        if N::is_far_enough((&nn.0.nm,&nn.0.rect),(&other.nm,&other.rect)){
            //anchor.mass.nm.handle_with(&mut nn.0.nm);
            nms.push(nn.0);
            return;
        }
    
        for i in nn.1.range.iter_mut(){
            bots.push(i)
        }

    
    }
    */
    fn recc3<'a:'b,'b,
        T:SweepTrait+'a,
        N:NodeMassTrait<T=T>+'a,
        C:CTreeIterator<Item=(&'a mut NodeMassWrapper<N>,&'a mut NodeDyn<T>)>>
    (mut stuff:C,nms:&mut Vec<&'b mut NodeMassWrapper<N>>,bots:&mut Vec<&'b mut T>,other:&CenterOfMass<T::Num>)
    {
        let (nn,rest)=stuff.next();
        
        //handle_a_node(nn,nms,bots,other);
        
        if N::is_far_enough(&nn.0.create_center(),other){
            //anchor.mass.nm.handle_with(&mut nn.0.nm);
            nms.push(nn.0);
            return;
        }
    
        for i in nn.1.range.iter_mut(){
            bots.push(i)
        }
        
    

        match rest{
            Some((left,right))=>{
                recc3(left,nms,bots,other);
                recc3(right,nms,bots,other);
            },
            None=>{
                
            }
        }
        
    }
}



pub fn nbody_seq<A:AxisTrait,T:SweepTrait,N:NodeMassTrait<T=T>>(tree:&mut DynTree<A,T>,rect:&Rect<T::Num>){

   
    fn recc<A:AxisTrait,T:SweepTrait,N:NodeMassTrait<T=T>>(axis:A,it1:NdIterMut<T>,it2:compt::dfs::DownTMut<NodeMassWrapper<N>>,rect:Rect<T::Num>){
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

                match nn1.cont{
                    Some(cont)=>{
                        let l1=left2.create_wrap_mut().zip(left.create_wrap_mut());
                        let l2=right2.create_wrap_mut().zip(right.create_wrap_mut());

                        let mut anchor={
                            //Create a new mass that is only the rect of that contains all the bots intersecting the divider.
                            //let rect=rect.constrain_by(nn1.cont); //TODO

                            let mut rect=rect;
                            *rect.get_range2_mut::<A>()=cont;
                            let m=NodeMassWrapper{nm:N::new(nn1.range.iter(),nn1.range.len()),rect};
                            Anchor{mass:m,node:nn1}
                        };

                        handle_anchor_with_children(&mut anchor,l1,l2);
                        let l=anchor.node.range.len();
                        anchor.mass.nm.undo(anchor.node.range.iter_mut(),l);
                    },
                    None=>{

                    }
                }
                //At this point, everything has been handled with the root.
                //before we can fully remove the root, and reduce this problem to two smaller trees,
                //we have to do one more thing.
                //we have to handle all the bots on the left of the root with all the bots on the right of the root.

                //from the left side,get a list of nodemases.
                //from the right side,get a list of nodemases.
                //collide the two.


                {
                    let (lcenter,rcenter)={
                        let l=left2.create_wrap_mut().next().0;
                        let r=right2.create_wrap_mut().next().0;
                        let lcenter=l.create_center();//CenterOfMass{center:l.nm.center_of_mass(),rect:l.nm.rect};
                        let rcenter=r.create_center();//CenterOfMass{center:r.nm.center_of_mass(),rect:r.nm.rect};
                        (lcenter,rcenter)
                    };
                    let l1=left2.create_wrap_mut().zip(left.create_wrap_mut());
                    let l2=right2.create_wrap_mut().zip(right.create_wrap_mut());
                    handle_left_with_right(l1,l2,&lcenter,&rcenter);
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

    use dinotree_inner::tools::Timer2;
    let timer=Timer2::new();

    //tree containing the nodemass of each node (and decendants)
    let mut tree2=buildtree::<_,_,N>(tree,*rect);

    println!("build timer={:?}",timer.elapsed());
    let timer=Timer2::new();
    {
        let it1=tree.get_iter_mut();
        let it2=tree2.create_down_mut();
        recc(A::new(),it1,it2,*rect);
    }
    println!("main alg={:?}",timer.elapsed());
    let timer=Timer2::new();


    apply_tree(tree,tree2);

    println!("application={:?}",timer.elapsed());
    //Now lets go through and apply the results of the node masses
}
