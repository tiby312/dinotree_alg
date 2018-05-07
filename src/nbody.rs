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
}



pub trait NodeMassTrait:Send+Clone{
    type T:SweepTrait;
    type No:Send;

    //gravitate this nodemass with another node mass
    fn handle_node_with_node(&self,&mut Self::No,b:&mut Self::No);

    //gravitate a bot with a bot
    fn handle_bot_with_bot(&self,&mut Self::T,&mut Self::T);

    //gravitate a nodemass with a bot
    fn handle_node_with_bot(&self,&mut Self::No,b:&mut Self::T);

    fn is_far_enough(&self,a:<Self::T as SweepTrait>::Num,b:<Self::T as SweepTrait>::Num)->bool;

    fn is_far_enough_half(&self,a:<Self::T as SweepTrait>::Num,b:<Self::T as SweepTrait>::Num)->bool;

    fn undo<'a,I:Iterator<Item=&'a mut Self::T>> (&'a self,&'a Self::No,it:I,len:usize);

    fn new<'a,I:Iterator<Item=&'a Self::T>> (&'a self,it:I,len:usize)->Self::No;

    fn div(self)->(Self,Self);
}





//pseudo code
//build up a tree where every nodemass has the mass of all the bots in that node and all the bots under it.
fn buildtree<'a,
    A:AxisTrait,
    T:SweepTrait+'a,
    N:NodeMassTrait<T=T>
    >
    (tree:&DynTree<A,T>,ncontext:N)->compt::dfs::GenTreeDfsOrder<N::No>{


    fn recc<'a,A:AxisTrait,T:SweepTrait+'a,N:NodeMassTrait<T=T>>
        (axis:A,stuff:NdIter<T>,vec:&mut Vec<N::No>,ncontext:N){

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
                            let mut nodeb=ncontext.new(bots_to_add.drain(..),len);
                            nodeb
                        };

                        let (n1,n2)=ncontext.div();
                        recc(axis.next(),left,vec,n1);
                        
                        vec.push(nodeb);
                        
                        recc(axis.next(),righ,vec,n2);
                    },
                    None=>{
                        let mut nodeb=ncontext.new(nn.range.iter(),nn.range.len());
                        
                        let (n1,n2)=ncontext.div();
                        //recurse anyway even though there is no divider.
                        //we want to populate this tree entirely.
                        recc(axis.next(),left,vec,n1);
                        
                        vec.push(nodeb);
                        
                        recc(axis.next(),righ,vec,n2); 
                    }
                }
            },
            None=>{
                let mut nodeb=ncontext.new(nn.range.iter(),nn.range.len());
                vec.push(nodeb);
            }
        }

        fn recc2<'a,T:SweepTrait+'a>(nodeb:&mut Vec<&'a T>,stuff:NdIter<'a,T>){
            let (nn,rest)=stuff.next();

            for i in nn.range.iter(){
                nodeb.push(i);
            }
         
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
    recc(A::new(),stuff,&mut vec,ncontext);


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
    (tree:&mut DynTree<A,T>,tree2:compt::dfs::GenTreeDfsOrder<N::No>,ncontext:N){

    fn recc<'a,T:SweepTrait+'a,N:NodeMassTrait<T=T>>
        (stuff:NdIterMut<T>,stuff2:compt::dfs::DownT<N::No>,ncontext:N){

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
                    ncontext.undo(nodeb,bots_to_undo.drain(..),l);
                }
                let (n1,n2)=ncontext.div();
                recc(left,left2,n1);
                recc(righ,right2,n2);
            },
            None=>{
                let l=nn1.range.len();
                ncontext.undo(nodeb,nn1.range.iter_mut(),l);
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
    recc(stuff,stuff2,ncontext);


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
(thisa:A,anchor:&mut Anchor<B,N::T>,left:DIter<N>,right:DIter<N>,ncontext:&N){
    
    struct Bo<'a,B:AxisTrait,N:NodeMassTrait+'a>{
        _anchor_axis:B,
        _p:PhantomData<N::No>,
        ncontext:&'a N
    }
    
    impl<'a,B:AxisTrait,N:NodeMassTrait+'a> Bok for Bo<'a,B,N>{
        type N=N;
        type T=N::T;
        type B=B;

        fn handle_every_node<A:AxisTrait>(&mut self,b:&mut N::T,anchor:&mut Anchor<B,Self::T>){
            for i in anchor.range.iter_mut(){
                self.ncontext.handle_bot_with_bot(i,b);
            }
        }
        fn handle_far_enough<A:AxisTrait>(&mut self,a:&mut N::No,anchor:&mut Anchor<B,Self::T>){
            for i in anchor.range.iter_mut(){
                self.ncontext.handle_node_with_bot(a,i);
            }
        }
        fn is_far_enough(&mut self,a:<Self::T as SweepTrait>::Num,b:<Self::T as SweepTrait>::Num)->bool{
            self.ncontext.is_far_enough(a,b)
        }
    }
    let mut bo= Bo{_anchor_axis:B::new(),_p:PhantomData,ncontext:ncontext};
    generic_rec(Left,thisa,anchor,left,&mut bo);  
    generic_rec(Right,thisa,anchor,right,&mut bo);  
}


struct DIter<'a,N:NodeMassTrait+'a>{
    a:BothIter<'a,N>,
    depth:usize
}
impl<'a,N:NodeMassTrait+'a> DIter<'a,N>{
    fn create_wrap_mut<'b>(&'b mut self)->DIter<'b,N>{
        
        let a=self.a.create_wrap_mut();
        DIter{a,depth:self.depth}
    }
}

impl<'a,N:NodeMassTrait+'a> CTreeIterator for DIter<'a,N>{
    type Item=(Depth,(&'a mut NodeDyn<N::T>,&'a mut N::No));
    fn next(self)->(Self::Item,Option<(Self,Self)>){
        let (n1,rest1)=self.a.next();
        
        let n1=(Depth(self.depth),n1);
        match rest1{
            Some((left,right))=>{
                let depth=self.depth+1;
                (n1,Some((DIter{a:left,depth},DIter{a:right,depth})))
            },
            None=>{
                (n1,None)  
            }
        }
    }
}

struct BothIter<'a,N:NodeMassTrait+'a>{
    it1:NdIterMut<'a,N::T>,
    it2:compt::dfs::DownTMut<'a,N::No>
}
impl<'a,N:NodeMassTrait+'a> BothIter<'a,N>{
    fn create_wrap_mut<'b>(&'b mut self)->BothIter<'b,N>{
        let it1=self.it1.create_wrap_mut();
        let it2=self.it2.create_wrap_mut();
        BothIter{it1,it2}
    }
}

impl<'a,N:NodeMassTrait+'a> CTreeIterator for BothIter<'a,N>{
    type Item=(&'a mut NodeDyn<N::T>,&'a mut N::No);
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
    (_axis:A,anchor:&mut Anchor<B,N::T>,left:DIter<N>,mut right:DIter<N>,ncontext:&N){

	struct Bo4<'a,B:AxisTrait,N:NodeMassTrait+'a,>{
        _anchor_axis:B,
        bot:&'a mut N::T,
        ncontext:&'a N
    }

    impl<'a,B:AxisTrait,N:NodeMassTrait+'a,> Bok for Bo4<'a,B,N>{
    	type N=N;
        type T=N::T;
        type B=B;
    	fn handle_every_node<A:AxisTrait>(&mut self,b:&mut Self::T,_anchor:&mut Anchor<B,Self::T>){
            self.ncontext.handle_bot_with_bot(self.bot,b);
    	}
    	fn handle_far_enough<A:AxisTrait>(&mut self,a:&mut N::No,_anchor:&mut Anchor<B,Self::T>){
    		self.ncontext.handle_node_with_bot(a,self.bot);
    	}
        fn is_far_enough(&mut self,a:<Self::T as SweepTrait>::Num,b:<Self::T as SweepTrait>::Num)->bool{
            self.ncontext.is_far_enough_half(a,b)
        }
    }
    struct Bo2<'a,B:AxisTrait,N:NodeMassTrait+'a>{
        _anchor_axis:B,
        node:&'a mut N::No,
        ncontext:&'a N
    }

    impl<'a,B:AxisTrait,N:NodeMassTrait+'a> Bok for Bo2<'a,B,N>{
    	type N=N;
        type T=N::T;
        type B=B;
        fn handle_every_node<A:AxisTrait>(&mut self,b:&mut N::T,_anchor:&mut Anchor<B,Self::T>){
            self.ncontext.handle_node_with_bot(self.node,b);
    	}
    	fn handle_far_enough<A:AxisTrait>(&mut self,a:&mut N::No,_anchor:&mut Anchor<B,Self::T>){
    		self.ncontext.handle_node_with_node(self.node,a);
    	}
        fn is_far_enough(&mut self,a:<Self::T as SweepTrait>::Num,b:<Self::T as SweepTrait>::Num)->bool{
            self.ncontext.is_far_enough_half(a,b)
        }
    }

    struct Bo<'a:'b,'b,B:AxisTrait,N:NodeMassTrait+'a>{
        _anchor_axis:B,
        right:&'b mut DIter<'a,N>,
        ncontext:&'b N
    }
    
    impl<'a:'b,'b,B:AxisTrait,N:NodeMassTrait+'a> Bok for Bo<'a,'b,B,N>{
    	type N=N;
        type T=N::T;
        type B=B;
        fn handle_every_node<A:AxisTrait>(&mut self,b:&mut N::T,anchor:&mut Anchor<B,Self::T>){
    		let r=self.right.create_wrap_mut();
    		generic_rec(Right,A::new(),anchor,r,&mut Bo4{_anchor_axis:B::new(),bot:b,ncontext:self.ncontext})
    	}
    	fn handle_far_enough<A:AxisTrait>(&mut self,a:&mut N::No,anchor:&mut Anchor<B,Self::T>){
    		let r=self.right.create_wrap_mut();
    		generic_rec(Right,A::new(),anchor,r,&mut Bo2{_anchor_axis:B::new(),node:a,ncontext:self.ncontext})
    	}
        fn is_far_enough(&mut self,a:<Self::T as SweepTrait>::Num,b:<Self::T as SweepTrait>::Num)->bool{
            self.ncontext.is_far_enough_half(a,b)
        }
    }
    let mut bo= Bo{_anchor_axis:B::new(),right:&mut right,ncontext};
    generic_rec(Left,A::new(),anchor,left,&mut bo);  
}

trait Bok{
	type N:NodeMassTrait<T=Self::T>;
	type T:SweepTrait;
    type B:AxisTrait;
    fn is_far_enough(&mut self,a:<Self::T as SweepTrait>::Num,b:<Self::T as SweepTrait>::Num)->bool;
	fn handle_every_node<A:AxisTrait>(&mut self,n:&mut Self::T,anchor:&mut Anchor<Self::B,Self::T>);
	fn handle_far_enough<A:AxisTrait>(&mut self,a:&mut <Self::N as NodeMassTrait>::No,anchor:&mut Anchor<Self::B,Self::T>);
}


fn generic_rec<
    A:AxisTrait,
    AnchorAxis:AxisTrait,
    B:Bok<N=N,T=T,B=AnchorAxis>,
    N:NodeMassTrait<T=T>,
    T:SweepTrait,
    L:LeftOrRight,
    >(side:L,this_axis:A,anchor:&mut Anchor<AnchorAxis,T>,stuff:DIter<N>,bok:&mut B){

	    
    fn recc4<
        A:AxisTrait,
        AnchorAxis:AxisTrait,
        B:Bok<N=N,T=T,B=AnchorAxis>,
        N:NodeMassTrait<T=T>,
        T:SweepTrait,
        >(axis:A,bok:&mut B,stuff:DIter<N>,anchor:&mut Anchor<AnchorAxis,T>){
        let ((_depth,(nn1,_)),rest)=stuff.next();
        
        for i in nn1.range.iter_mut(){
            bok.handle_every_node::<A>(i,anchor);
        }
        match rest{
            Some((left,right))=>{
                recc4(axis.next(),bok,left,anchor);
                recc4(axis.next(),bok,right,anchor);
            },
            None=>{

            }
        }
    }

	let ((_depth,(nn1,_)),rest)=stuff.next();
    
    

    for i in nn1.range.iter_mut(){
        bok.handle_every_node::<A>(i,anchor);    
    }

    
    match rest{
        Some((left,right))=>{
            let div=match nn1.div{
                Some(div)=>div,
                None=>{
                    return;
                }
            };
            
            
			if A::get()==AnchorAxis::get(){
	        	
                if bok.is_far_enough(div,anchor.div){
                    let (mut side_to_stop,side_to_continue)=if side.is_left(){
                        (left,right)
                    }else{
                        (right,left)
                    };
	    			//the left node is far enough away.
	    			//handle the left as a whole, and recurse the right only.
		        	let (_dd1,(_nn1,nn2))=side_to_stop.create_wrap_mut().next().0;
		        	
		        	bok.handle_far_enough::<A>(nn2,anchor);//handle_node(a,&mut right_tree,div);

		        	recc4(this_axis.next(),bok,side_to_continue,anchor);

	            }else{

	                generic_rec(side,this_axis.next(),anchor,left,bok);
	                generic_rec(side,this_axis.next(),anchor,right,bok);
	            }
	        }else{
	        	generic_rec(side,this_axis.next(),anchor,left,bok);
	        	generic_rec(side,this_axis.next(),anchor,right,bok);
	        }   
	    },
	    None=>{

	    }
	}   	
}

  




fn recc<J:par::Joiner,A:AxisTrait,N:NodeMassTrait>(join:J,axis:A,it:DIter<N>,ncontext:N){
    let ((depth,(nn1,_)),rest)=it.next();
    

    //handle bots in itself
    tools::for_every_pair(&mut nn1.range,|a,b|{ncontext.handle_bot_with_bot(a,b)});
    

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

                    handle_anchor_with_children(axis.next(),&mut anchor,l1,l2,&ncontext);
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

                handle_left_with_right(axis.next(),&mut anchor,l1,l2,&ncontext);
            }
            //at this point we have successfully broken up this problem
            //into two independant ones, and we can do this all over again for the two children.
            //potentially in parlalel.
           
            let (n1,n2)=ncontext.div();
            if join.should_switch_to_sequential(depth){
                recc(join.into_seq(),axis.next(),left,n1);
                recc(join.into_seq(),axis.next(),right,n2);
            }else{
                rayon::join(
                ||recc(join,axis.next(),left,n1),
                ||recc(join,axis.next(),right,n2)
                );
            }
            
            
            
        },
        None=>{

        }
    }
}

pub fn nbody_par<A:AxisTrait,T:SweepTrait,N:NodeMassTrait<T=T>>(tree:&mut DynTree<A,T>,ncontext:N){



    //use dinotree_inner::tools::Timer2;
    //let timer=Timer2::new();

    //tree containing the nodemass of each node (and decendants)
    //TODO add this to the existing tree isntead of making a new tree???
    let mut tree2=buildtree::<_,_,N>(tree,ncontext.clone());

    {
        let height=tree.get_height();
        let it1=tree.get_iter_mut();
        let it2=tree2.create_down_mut();
        let b=BothIter{it1,it2};
        let d=DIter{a:b,depth:0};

        let kk=if height<3{
            0
        }else{
            height-3
        };


        recc(par::Parallel(Depth(kk)),A::new(),d,ncontext.clone());
    }
    

    apply_tree(tree,tree2,ncontext);

}

pub fn nbody_seq<A:AxisTrait,T:SweepTrait,N:NodeMassTrait<T=T>>(tree:&mut DynTree<A,T>,ncontext:N){



    //use dinotree_inner::tools::Timer2;
    //let timer=Timer2::new();

    //tree containing the nodemass of each node (and decendants)
    //TODO add this to the existing tree isntead of making a new tree???
    let mut tree2=buildtree::<_,_,N>(tree,ncontext.clone());

    {
        let it1=tree.get_iter_mut();
        let it2=tree2.create_down_mut();
        let b=BothIter{it1,it2};
        let d=DIter{a:b,depth:0};
        recc(par::Sequential,A::new(),d,ncontext.clone());
    }
    

    apply_tree(tree,tree2,ncontext);

}
