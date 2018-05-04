use inner_prelude::*;

//TODO somehow take advantage of sorted property?????

mod tools{
    pub fn for_every_pair<T,F:FnMut(&mut T,&mut T)>(arr:&mut [T],mut func:F){
        unsafe{
            for x in 0..arr.len(){
                let xx=arr.get_unchecked_mut(x) as *mut T;
                for j in (x+1)..arr.len(){
                    
                    let j=arr.get_unchecked_mut(j);
                    let xx=&mut*xx;
                    func(xx,j);
                }
            }
        }
    }
    /*
    pub fn for_bijective_pair<T,F:FnMut(&mut T,&mut T)>(arr1:&mut [T],arr2:&mut [T],mut func:F){
        for x in arr1.iter_mut(){
            for j in arr2.iter_mut(){
                func(x,j);
            }
        }
    }
    */
}



pub trait NodeMassTrait:Send{
    type T:SweepTrait;


    //gravitate this nodemass with another node mass
    fn handle_with(&mut self,b:&mut Self);

    //gravitate a bot with a bot
    fn handle_bot(&mut Self::T,&mut Self::T);

    //gravitate a nodemass with a bot
    fn apply(&mut self,b:&mut Self::T);

    fn is_far_enough(a:<Self::T as SweepTrait>::Num,b:<Self::T as SweepTrait>::Num)->bool;

    fn is_far_enough_half(a:<Self::T as SweepTrait>::Num,b:<Self::T as SweepTrait>::Num)->bool;

    fn undo<'a,I:Iterator<Item=&'a mut Self::T>> (&self,it:I,len:usize) where Self::T:'a;

    fn new<'a,I:Iterator<Item=&'a Self::T>> (it:I,len:usize)->Self where Self::T:'a;
}





//pseudo code
//build up a tree where every nodemass has the mass of all the bots in that node and all the bots under it.
fn buildtree<'a,
    A:AxisTrait,
    T:SweepTrait+'a,
    N:NodeMassTrait<T=T>
    >
    (tree:&DynTree<A,T>)->compt::dfs::GenTreeDfsOrder<N>{


    fn recc<'a,A:AxisTrait,T:SweepTrait+'a,N:NodeMassTrait<T=T>>
        (axis:A,stuff:NdIter<T>,vec:&mut Vec<N>){

        let (nn,rest)=stuff.next();


        match rest{
            Some((mut left,mut righ))=>{
                

                match nn.div{
                    Some(_div)=>{
                        
                        
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
                            let mut nodeb=N::new(bots_to_add.drain(..),len);
                            nodeb
                        };

                        
                        recc(axis.next(),left,vec);
                        
                        vec.push(nodeb);
                        
                        recc(axis.next(),righ,vec);
                    },
                    None=>{
                        //recurse anyway even though there is no divider.
                        //we want to populate this tree entirely.
                       recc(axis.next(),left,vec);
                        
                        let mut nodeb=N::new(nn.range.iter(),nn.range.len());
                        vec.push(nodeb);
                        
                        recc(axis.next(),righ,vec); 
                    }
                }
            },
            None=>{
                let mut nodeb=N::new(nn.range.iter(),nn.range.len());
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
    recc(A::new(),stuff,&mut vec);


    let len=vec.len();
    match compt::dfs::GenTreeDfsOrder::from_vec(vec,height){
        Ok(a)=>a,
        Err(e)=>{
            panic!("vec size={:?} {:?}",len,e);
        }
    }

}

fn apply_tree<'a,
    A:AxisTrait,
    T:SweepTrait+'a,
    N:NodeMassTrait<T=T>
    >
    (tree:&mut DynTree<A,T>,tree2:compt::dfs::GenTreeDfsOrder<N>){

    fn recc<'a,T:SweepTrait+'a,N:NodeMassTrait<T=T>>
        (stuff:NdIterMut<T>,stuff2:compt::dfs::DownT<N>){

        let (nn1,rest)=stuff.next();
        let (nodeb,rest2)=stuff2.next();
        

        match rest{
            Some((mut left,mut righ))=>{
                let (left2,right2)=rest2.unwrap();

                let _div=match nn1.div{
                    Some(div)=>{div},
                    None=>{return;}
                };

                
                {
                    let mut bots_to_undo:Vec<&mut T>=Vec::with_capacity(nn1.range.len());
                    for b in nn1.range.iter_mut(){
                        bots_to_undo.push(b);
                    }
                    let left=left.create_wrap_mut();
                    let righ=righ.create_wrap_mut();

                    recc2(&mut bots_to_undo,left);
                    recc2(&mut bots_to_undo,righ);

                    let l=bots_to_undo.len();
                    nodeb.undo(bots_to_undo.drain(..),l);
                }

                recc(left,left2);
                recc(righ,right2);
            },
            None=>{
                let l=nn1.range.len();
                nodeb.undo(nn1.range.iter_mut(),l);
                //nodeb.undo()
            }
        }

        fn recc2<'a,T:SweepTrait+'a>(bots:&mut Vec<&'a mut T>,stuff:NdIterMut<'a,T>){
            let (nn,rest)=stuff.next();

            match rest{
                Some((left,right))=>{
                    match nn.div{
                        Some(_div)=>{
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


use self::ll::*;
mod ll{

    #[derive(Copy,Clone)]
    pub struct Left;
    impl LeftOrRight for Left{
        fn is_left(&self)->bool{true}
    }

    #[derive(Copy,Clone)]
    pub struct Right;
    impl LeftOrRight for Right{
        fn is_left(&self)->bool{false}
    }

    pub trait LeftOrRight:Copy+Clone{
        fn is_left(&self)->bool;
    }
}



//Construct anchor from cont!!!
struct Anchor<'a,A:AxisTrait,T:SweepTrait+'a>{
	_axis:A,
    range:&'a mut [T],
    div:T::Num
}

fn handle_anchor_with_children<'a,
	A:AxisTrait,
	B:AxisTrait,
    N:NodeMassTrait+'a>
(thisa:B,anchor:&mut Anchor<A,N::T>,left:BothIter<N>,right:BothIter<N>){
    
    struct Bo<A:AxisTrait,N:NodeMassTrait>{
        _axis:A,
        _p:PhantomData<N>
    }
    
    impl<A:AxisTrait,N:NodeMassTrait> Bok for Bo<A,N>{
        type N=N;
        type T=N::T;

        fn handle_every_node<B:AxisTrait>(&mut self,b:&mut N::T,anchor:&mut Anchor<B,Self::T>){
            for i in anchor.range.iter_mut(){
                N::handle_bot(i,b);
            }
        }
        fn handle_far_enough<B:AxisTrait>(&mut self,a:&mut N,anchor:&mut Anchor<B,Self::T>){
            for i in anchor.range.iter_mut(){
                a.apply(i);
            }
        }
    }
    let mut bo= Bo{_axis:A::new(),_p:PhantomData};
    generic_rec(Left,A::new(),anchor,left,&mut bo,&mut |a,b|N::is_far_enough(a,b));  
    generic_rec(Right,A::new(),anchor,right,&mut bo,&mut |a,b|N::is_far_enough(a,b));  
}


struct BothIter<'a,N:NodeMassTrait+'a>{
    it1:NdIterMut<'a,N::T>,
    it2:compt::dfs::DownTMut<'a,N>
}
impl<'a,N:NodeMassTrait+'a> BothIter<'a,N>{
    fn create_wrap_mut<'b>(&'b mut self)->BothIter<'b,N>{
        let it1=self.it1.create_wrap_mut();
        let it2=self.it2.create_wrap_mut();
        BothIter{it1,it2}
    }
}

impl<'a,N:NodeMassTrait+'a> CTreeIterator for BothIter<'a,N>{
    type Item=(&'a mut NodeDyn<N::T>,&'a mut N);
    fn next(self)->(Self::Item,Option<(Self,Self)>){
        let (n1,rest1)=self.it1.next();
        let (n2,rest2)=self.it2.next();
        
        match rest1{
            Some((left,right))=>{
                let (ll,rr)=rest2.unwrap();

                ((n1,n2),Some((BothIter{it1:left,it2:ll},BothIter{it1:right,it2:rr})))
            },
            None=>{
                ((n1,n2),None)  
            }
        }
    }
}



fn handle_left_with_right<A:AxisTrait,B:AxisTrait,N:NodeMassTrait>
    (_axis:A,anchor:&mut Anchor<B,N::T>,left:BothIter<N>,mut right:BothIter<N>){

	struct Bo4<'a,N:NodeMassTrait+'a>(&'a mut N::T);

    impl<'a,N:NodeMassTrait+'a> Bok for Bo4<'a,N>{
    	type N=N;
        type T=N::T;
    	fn handle_every_node<B:AxisTrait>(&mut self,b:&mut Self::T,_anchor:&mut Anchor<B,Self::T>){
    		N::handle_bot(self.0,b);
    	}
    	fn handle_far_enough<B:AxisTrait>(&mut self,a:&mut N,_anchor:&mut Anchor<B,Self::T>){
    		a.apply(self.0);
    	}
    }
    struct Bo2<'a,N:NodeMassTrait+'a>(&'a mut N);

    impl<'a,N:NodeMassTrait+'a> Bok for Bo2<'a,N>{
    	type N=N;
        type T=N::T;
        fn handle_every_node<B:AxisTrait>(&mut self,b:&mut N::T,_anchor:&mut Anchor<B,Self::T>){
    		self.0.apply(b);
    	}
    	fn handle_far_enough<B:AxisTrait>(&mut self,a:&mut N,_anchor:&mut Anchor<B,Self::T>){
    		a.handle_with(self.0);
    	}
    }

    struct Bo<'a:'b,'b,A:AxisTrait,N:NodeMassTrait+'a>{
    	_axis:A,
        right:&'b mut BothIter<'a,N>
    }
    
    impl<'a:'b,'b,A:AxisTrait,N:NodeMassTrait+'a> Bok for Bo<'a,'b,A,N>{
    	type N=N;
        type T=N::T;

        fn handle_every_node<B:AxisTrait>(&mut self,b:&mut N::T,anchor:&mut Anchor<B,Self::T>){
    		let r=self.right.create_wrap_mut();
    		generic_rec(Right,A::new(),anchor,r,&mut Bo4(b),&mut |a,b|N::is_far_enough_half(a,b))
    	}
    	fn handle_far_enough<B:AxisTrait>(&mut self,a:&mut N,anchor:&mut Anchor<B,Self::T>){
    		let r=self.right.create_wrap_mut();
    		generic_rec(Right,A::new(),anchor,r,&mut Bo2(a),&mut |a,b|N::is_far_enough_half(a,b))
    	}
    }
    let mut bo= Bo{_axis:A::new(),right:&mut right};
    generic_rec(Left,A::new(),anchor,left,&mut bo,&mut |a,b|N::is_far_enough_half(a,b));  
}

trait Bok{
	type N:NodeMassTrait<T=Self::T>;
	type T:SweepTrait;
	fn handle_every_node<A:AxisTrait>(&mut self,n:&mut Self::T,anchor:&mut Anchor<A,Self::T>);
	fn handle_far_enough<A:AxisTrait>(&mut self,a:&mut Self::N,anchor:&mut Anchor<A,Self::T>);
}


fn generic_rec<
    A:AxisTrait,
    AnchorAxis:AxisTrait,
    B:Bok<N=N,T=T>,
    N:NodeMassTrait<T=T>,
    T:SweepTrait,
    L:LeftOrRight,
    F:FnMut(T::Num,T::Num)->bool>(side:L,this_axis:A,anchor:&mut Anchor<AnchorAxis,T>,stuff:BothIter<B::N>,bok:&mut B,func:&mut F){

	    
    fn recc4<
        AnchorAxis:AxisTrait,
        B:Bok<N=N,T=T>,
        N:NodeMassTrait<T=T>,
        T:SweepTrait,
        >(bok:&mut B,stuff:BothIter<B::N>,anchor:&mut Anchor<AnchorAxis,T>){
        let ((nn1,_),rest)=stuff.next();
        
        for i in nn1.range.iter_mut(){
            bok.handle_every_node(i,anchor);
        }
        match rest{
            Some((left,right))=>{
                recc4(bok,left,anchor);
                recc4(bok,right,anchor);
            },
            None=>{

            }
        }
    }

	let ((nn1,_),rest)=stuff.next();
    
    let div=match nn1.div{
    	Some(div)=>div,
    	None=>return
    };

    for i in nn1.range.iter_mut(){
        bok.handle_every_node(i,anchor);    
    }
    
    match rest{
        Some((left,right))=>{
			if A::get()==AnchorAxis::get(){
	        	
                //B::N::is_far_enough_half(div,anchor.div)
	        	if func(div,anchor.div){
                    let (mut side_to_stop,side_to_continue)=if side.is_left(){
                        (left,right)
                    }else{
                        (right,left)
                    };
	    			//the left node is far enough away.
	    			//handle the left as a whole, and recurse the right only.
		        	let a=side_to_stop.create_wrap_mut().next().0;
		        	
		        	bok.handle_far_enough(a.1,anchor);//handle_node(a,&mut right_tree,div);

		        	recc4(bok,side_to_continue,anchor);

	            }else{

	                generic_rec(side,this_axis.next(),anchor,left,bok,func);
	                generic_rec(side,this_axis.next(),anchor,right,bok,func);
	            }
	        }else{
	        	generic_rec(side,this_axis.next(),anchor,left,bok,func);
	        	generic_rec(side,this_axis.next(),anchor,right,bok,func);
	        }
	    },
	    None=>{

	    }
	}   	
}

  




pub fn nbody_seq<A:AxisTrait,T:SweepTrait,N:NodeMassTrait<T=T>>(tree:&mut DynTree<A,T>){

   
    fn recc<A:AxisTrait,N:NodeMassTrait>(axis:A,it:BothIter<N>){
        let ((nn1,_),rest)=it.next();
        

        //handle bots in itself
        tools::for_every_pair(&mut nn1.range,|a,b|{N::handle_bot(a,b)});
        

        match rest{
            Some((mut left,mut right))=>{
                let div=match nn1.div{
                    Some(div)=>{div},
                    None=>{return;}
                };

                match nn1.cont{
                    Some(_cont)=>{
                        let l1=left.create_wrap_mut();
                        let l2=right.create_wrap_mut();
                        let mut anchor=Anchor{_axis:axis,range:&mut nn1.range,div};

                        handle_anchor_with_children(axis,&mut anchor,l1,l2);
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
                    let l1=left.create_wrap_mut();
                    let l2=right.create_wrap_mut();
                    let mut anchor=Anchor{_axis:axis,range:&mut nn1.range,div};

                    handle_left_with_right(A::new(),&mut anchor,l1,l2);
                }
                //at this point we have successfully broken up this problem
                //into two independant ones, and we can do this all over again for the two children.
                //potentially in parlalel.
               
                
                recc(axis.next(),left);
                recc(axis.next(),right);
                /*
                rayon::join(
                ||recc(axis.next(),left),
                ||recc(axis.next(),right)
                );
                */
                
            },
            None=>{

            }
        }
    }

    //use dinotree_inner::tools::Timer2;
    //let timer=Timer2::new();

    //tree containing the nodemass of each node (and decendants)
    //TODO add this to the existing tree isntead of making a new tree???
    let mut tree2=buildtree::<_,_,N>(tree);

    {
        let it1=tree.get_iter_mut();
        let it2=tree2.create_down_mut();
        recc(A::new(),BothIter{it1,it2});
    }
    

    apply_tree(tree,tree2);

}
