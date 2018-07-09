use inner_prelude::*;

pub trait NodeMassTrait:Clone{
    type T:HasAabb;
    type No:Copy;

    //gravitate this node mass with another node mass
    fn handle_node_with_node(&self,&mut Self::No,b:&mut Self::No);

    //gravitate a bot with a bot
    fn handle_bot_with_bot(&self,&mut Self::T,&mut Self::T);

    //gravitate a nodemass with a bot
    fn handle_node_with_bot(&self,&mut Self::No,b:&mut Self::T);

    fn is_far_enough(&self,depth:usize,b:[<Self::T as HasAabb>::Num;2])->bool;

    fn is_far_enough_half(&self,depth:usize,b:[<Self::T as HasAabb>::Num;2])->bool;

    //This unloads the force accumulated by this node to the bots.
    fn apply_to_bots<'a,I:Iterator<Item=&'a mut Self::T>> (&'a self,&'a Self::No,it:I);

    fn new<'a,I:Iterator<Item=&'a Self::T>> (&'a self,it:I)->Self::No;
}



pub fn naive_mut<T:HasAabb>(bots:&mut [T],nm:impl NodeMassTrait<T=T>){
    tools::for_every_pair(bots,|a,b|{
        nm.handle_bot_with_bot(a,b);
    });
}



//pseudo code
//build up a tree where every nodemass has the mass of all the bots in that node and all the bots under it.
fn buildtree<'a,
    T:HasAabb+Send+'a,
    N:NodeMassTrait<T=T>
    >
    (axis:impl AxisTrait,node:NdIterMut<N::No,T>,ncontext:N){


    fn recc<'a,T:HasAabb+'a,N:NodeMassTrait<T=T>>
        (axis:impl AxisTrait,stuff:NdIterMut<N::No,T>,ncontext:N){
        
        let (nn,rest)=stuff.next();
        match rest{
            Some((extra,mut left,mut right))=>{
                match extra{
                    None=>{
                        let empty:&[T]=&[];
                        let mut nodeb=ncontext.new(empty.iter());
                        
                        nn.misc=nodeb;
                        let n2=ncontext.clone();
                        //recurse anyway even though there is no divider.
                        //we want to populate this tree entirely.
                        recc(axis.next(),left,ncontext);    
                        recc(axis.next(),right,n2);
                    },
                    Some((div,cont))=>{

                        let nodeb={
                            let left=left.create_wrap_mut();
                            let right=right.create_wrap_mut();

                            
                            let i1=left.dfs_preorder_iter().flat_map(|(node,extra)|{node.range.iter()});
                            let i2=right.dfs_preorder_iter().flat_map(|(node,extra)|{node.range.iter()});
                            let i3=nn.range.iter().chain(i1.chain(i2));
                            ncontext.new(i3)
                        };

                        nn.misc=nodeb;

                        let n2=ncontext.clone();
                        recc(axis.next(),left,ncontext);
                        recc(axis.next(),right,n2);
                    }
                }
            },
            None=>{
                let mut nodeb=ncontext.new(nn.range.iter());
                nn.misc=nodeb;
            }
        }
    }
    recc(axis,node,ncontext);
}

fn apply_tree<'a,   
    T:HasAabb+'a,
    N:NodeMassTrait<T=T>
    >
    (_axis:impl AxisTrait,node:NdIterMut<N::No,T>,ncontext:N){

    fn recc<'a,T:HasAabb+'a,N:NodeMassTrait<T=T>>
        (stuff:NdIterMut<N::No,T>,ncontext:N){
        
        let (nn,rest)=stuff.next();
        match rest{
            Some((extra,mut left,mut right))=>{
                let (div,cont)=match extra{
                    Some(b)=>b,
                    None=>return
                };

                {
                    let left=left.create_wrap_mut();
                    let right=right.create_wrap_mut();
                                            
                    let i1=left.dfs_preorder_iter().flat_map(|(node,extra)|{node.range.iter_mut()});
                    let i2=right.dfs_preorder_iter().flat_map(|(node,extra)|{node.range.iter_mut()});
                    let i3=nn.range.iter_mut().chain(i1.chain(i2));
                    
                    ncontext.apply_to_bots(&mut nn.misc,i3);
                }
                let n2=ncontext.clone();
                recc(left,ncontext);
                recc(right,n2);
            },
            None=>{
                ncontext.apply_to_bots(&nn.misc,nn.range.iter_mut());
            }
        }
    }

    recc(node,ncontext);
}


//Construct anchor from cont!!!
struct Anchor<'a,A:AxisTrait,T:HasAabb+'a>{
	axis:A,
    range:&'a mut [T],
    div:T::Num
}

fn handle_anchor_with_children<'a,
	A:AxisTrait,
	B:AxisTrait,
    N:NodeMassTrait+'a>
(thisa:A,anchor:&mut Anchor<B,N::T>,left:LevelIter<NdIterMut<N::No,N::T>>,right:LevelIter<NdIterMut<N::No,N::T>>,ncontext:&N){
    
    struct Bo<'a,B:AxisTrait,N:NodeMassTrait+'a>{
        _anchor_axis:B,
        _p:PhantomData<N::No>,
        ncontext:&'a N
    }
    
    impl<'a,B:AxisTrait,N:NodeMassTrait+'a> Bok for Bo<'a,B,N>{
        type N=N;
        type T=N::T;
        type B=B;

        fn handle_every_node<A:AxisTrait>(&mut self,_axis:A,b:&mut N::T,anchor:&mut Anchor<B,Self::T>){
            for i in anchor.range.iter_mut(){
                self.ncontext.handle_bot_with_bot(i,b);
            }
        }
        fn handle_far_enough<A:AxisTrait>(&mut self,_axis:A,a:&mut N::No,anchor:&mut Anchor<B,Self::T>){
            for i in anchor.range.iter_mut(){
                self.ncontext.handle_node_with_bot(a,i);
            }
        }
        fn is_far_enough(&mut self,depth:Depth,b:[<Self::T as HasAabb>::Num;2])->bool{
            self.ncontext.is_far_enough(depth.0,b)
        }
    }
    let mut bo= Bo{_anchor_axis:anchor.axis,_p:PhantomData,ncontext:ncontext};
    generic_rec(thisa,anchor,left,&mut bo);  
    generic_rec(thisa,anchor,right,&mut bo);  
}

fn handle_left_with_right<'a,A:AxisTrait,B:AxisTrait,N:NodeMassTrait+'a>
    (axis:A,anchor:&mut Anchor<B,N::T>,left:LevelIter<NdIterMut<'a,N::No,N::T>>,mut right:LevelIter<NdIterMut<'a,N::No,N::T>>,ncontext:&N){

	struct Bo4<'a,B:AxisTrait,N:NodeMassTrait+'a,>{
        _anchor_axis:B,
        bot:&'a mut N::T,
        ncontext:&'a N
    }

    impl<'a,B:AxisTrait,N:NodeMassTrait+'a,> Bok for Bo4<'a,B,N>{
    	type N=N;
        type T=N::T;
        type B=B;
    	fn handle_every_node<A:AxisTrait>(&mut self,_axis:A,b:&mut Self::T,_anchor:&mut Anchor<B,Self::T>){
            self.ncontext.handle_bot_with_bot(self.bot,b);
    	}
    	fn handle_far_enough<A:AxisTrait>(&mut self,_axis:A,a:&mut N::No,_anchor:&mut Anchor<B,Self::T>){
    		self.ncontext.handle_node_with_bot(a,self.bot);
    	}
        fn is_far_enough(&mut self,depth:Depth,b:[<Self::T as HasAabb>::Num;2])->bool{
            self.ncontext.is_far_enough_half(depth.0,b)
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
        fn handle_every_node<A:AxisTrait>(&mut self,_axis:A,b:&mut N::T,_anchor:&mut Anchor<B,Self::T>){
            self.ncontext.handle_node_with_bot(self.node,b);
    	}
    	fn handle_far_enough<A:AxisTrait>(&mut self,_axis:A,a:&mut N::No,_anchor:&mut Anchor<B,Self::T>){
    		self.ncontext.handle_node_with_node(self.node,a);
    	}
        fn is_far_enough(&mut self,depth:Depth,b:[<Self::T as HasAabb>::Num;2])->bool{
            self.ncontext.is_far_enough_half(depth.0,b)
        }
    }

    struct Bo<'a:'b,'b,B:AxisTrait,N:NodeMassTrait+'a>{
        _anchor_axis:B,
        right:&'b mut LevelIter<NdIterMut<'a,N::No,N::T>>,
        ncontext:&'b N
    }
    
    impl<'a:'b,'b,B:AxisTrait,N:NodeMassTrait+'a> Bok for Bo<'a,'b,B,N>{
    	type N=N;
        type T=N::T;
        type B=B;
        fn handle_every_node<A:AxisTrait>(&mut self,axis:A,b:&mut N::T,anchor:&mut Anchor<B,Self::T>){
    		let d=self.right.depth;
            let r=self.right.inner.create_wrap_mut().with_depth(d);
            let anchor_axis=anchor.axis;
    		generic_rec(axis,anchor,r,&mut Bo4{_anchor_axis:anchor_axis,bot:b,ncontext:self.ncontext})
    	}
    	fn handle_far_enough<A:AxisTrait>(&mut self,axis:A,a:&mut N::No,anchor:&mut Anchor<B,Self::T>){
    		let d=self.right.depth;
            let r=self.right.inner.create_wrap_mut().with_depth(d);
            let anchor_axis=anchor.axis;
    		generic_rec(axis,anchor,r,&mut Bo2{_anchor_axis:anchor_axis,node:a,ncontext:self.ncontext})
    	}
        fn is_far_enough(&mut self,depth:Depth,b:[<Self::T as HasAabb>::Num;2])->bool{
            self.ncontext.is_far_enough_half(depth.0,b)
        }
    }
    let mut bo= Bo{_anchor_axis:anchor.axis,right:&mut right,ncontext};
    generic_rec(axis,anchor,left,&mut bo);  
}

fn recc<J:par::Joiner,A:AxisTrait,N:NodeMassTrait+Send>(join:J,axis:A,it:LevelIter<NdIterMut<N::No,N::T>>,ncontext:N) where N::T:Send,N::No:Send{
    

    let ((depth,nn),rest)=it.next();
    match rest{
        Some((extra,mut left,mut right))=>{
            let (div,cont)=match extra{
                Some(b)=>b,
                None=>return
            };

            //handle bots in itself
            tools::for_every_pair(&mut nn.range,|a,b|{ncontext.handle_bot_with_bot(a,b)});
            {
                let depth=left.depth;
                let l1=left.inner.create_wrap_mut().with_depth(depth);
                let l2=right.inner.create_wrap_mut().with_depth(depth);
                let mut anchor=Anchor{axis:axis,range:&mut nn.range,div};

                handle_anchor_with_children(axis.next(),&mut anchor,l1,l2,&ncontext);
            }
            //At this point, everything has been handled with the root.
            //before we can fully remove the root, and reduce this problem to two smaller trees,
            //we have to do one more thing.
            //we have to handle all the bots on the left of the root with all the bots on the right of the root.

            //from the left side,get a list of nodemases.
            //from the right side,get a list of nodemases.
            //collide the two.


            {
                let depth=left.depth;
                    
                let l1=left.inner.create_wrap_mut().with_depth(depth);
                let l2=right.inner.create_wrap_mut().with_depth(depth);
                let mut anchor=Anchor{axis:axis,range:&mut nn.range,div};

                handle_left_with_right(axis.next(),&mut anchor,l1,l2,&ncontext);
            }
            //at this point we have successfully broken up this problem
            //into two independant ones, and we can do this all over again for the two children.
            //potentially in parlalel.
           
            let n2=ncontext.clone();
            if join.should_switch_to_sequential(depth){
                recc(join.into_seq(),axis.next(),left,ncontext);
                recc(join.into_seq(),axis.next(),right,n2);
            }else{
                rayon::join(
                ||recc(join,axis.next(),left,ncontext),
                ||recc(join,axis.next(),right,n2)
                );
            }
        },
        None=>{
            //handle bots in itself
            tools::for_every_pair(&mut nn.range,|a,b|{ncontext.handle_bot_with_bot(a,b)});
        }
    }

    /*
    match rest{
        Some((mut left,mut right))=>{
            let div=match nn1.div{
                Some(div)=>{div},
                None=>{return;}
            };

            match nn1.cont{
                Some(_cont)=>{
                    let depth=left.depth;
                    let l1=left.inner.create_wrap_mut().with_depth(depth);
                    let l2=right.inner.create_wrap_mut().with_depth(depth);
                    let mut anchor=Anchor{axis:axis,range:&mut nn1.range,div};

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
                let depth=left.depth;
                    
                let l1=left.inner.create_wrap_mut().with_depth(depth);
                let l2=right.inner.create_wrap_mut().with_depth(depth);
                let mut anchor=Anchor{axis:axis,range:&mut nn1.range,div};

                handle_left_with_right(axis.next(),&mut anchor,l1,l2,&ncontext);
            }
            //at this point we have successfully broken up this problem
            //into two independant ones, and we can do this all over again for the two children.
            //potentially in parlalel.
           
            let n2=ncontext.clone();
            if join.should_switch_to_sequential(depth){
                recc(join.into_seq(),axis.next(),left,ncontext);
                recc(join.into_seq(),axis.next(),right,n2);
            }else{
                rayon::join(
                ||recc(join,axis.next(),left,ncontext),
                ||recc(join,axis.next(),right,n2)
                );
            }
        },
        None=>{

        }
    }
    */
}



trait Bok{
    type N:NodeMassTrait<T=Self::T>;
    type T:HasAabb;
    type B:AxisTrait;
    fn is_far_enough(&mut self,depth:Depth,b:[<Self::T as HasAabb>::Num;2])->bool;
    fn handle_every_node<A:AxisTrait>(&mut self,axis:A,n:&mut Self::T,anchor:&mut Anchor<Self::B,Self::T>);
    fn handle_far_enough<A:AxisTrait>(&mut self,axis:A,a:&mut <Self::N as NodeMassTrait>::No,anchor:&mut Anchor<Self::B,Self::T>);
}


fn generic_rec<
    A:AxisTrait,
    AnchorAxis:AxisTrait,
    B:Bok<N=N,T=T,B=AnchorAxis>,
    N:NodeMassTrait<T=T>,
    T:HasAabb,
    >(this_axis:A,anchor:&mut Anchor<AnchorAxis,T>,stuff:LevelIter<NdIterMut<N::No,T>>,bok:&mut B){

    let ((depth,nn),rest)=stuff.next();
    match rest{
        Some((extra,left,right))=>{
            let (div,cont)=match extra{
                Some(b)=>b,
                None=>return
            };
            for i in nn.range.iter_mut(){
                bok.handle_every_node(this_axis,i,anchor);    
            }

            if this_axis.is_equal_to(anchor.axis){
                if bok.is_far_enough(depth,[div,anchor.div]){
                    bok.handle_far_enough(this_axis,&mut nn.misc,anchor);
                    return;
                }        
            }

            generic_rec(this_axis.next(),anchor,left,bok);
            generic_rec(this_axis.next(),anchor,right,bok);
        },
        None=>{
            for i in nn.range.iter_mut(){
                bok.handle_every_node(this_axis,i,anchor);    
            }
        }
    }
    
    /*
    let ((_depth,nn1),rest)=stuff.next();
    
    for i in nn1.range.iter_mut(){
        bok.handle_every_node(this_axis,i,anchor);    
    }

    match rest{
        Some((left,right))=>{
            let div=match nn1.div{
                Some(div)=>div,
                None=>{
                    return;
                }
            };
            
            if this_axis.is_equal_to(anchor.axis){
                if bok.is_far_enough(_depth,[div,anchor.div]){
                    bok.handle_far_enough(this_axis,&mut nn1.misc,anchor);
                    return;
                }        
            }

            generic_rec(this_axis.next(),anchor,left,bok);
            generic_rec(this_axis.next(),anchor,right,bok);
        },
        None=>{

        }
    }  
    */     
}


pub fn nbody_par<A:AxisTrait,T:HasAabb+Send,N:NodeMassTrait<T=T>+Send>(t1:&mut DynTree<A,N::No,T>,ncontext:N) where N::No:Send{
    let axis=t1.get_axis();
    let height=t1.get_height();
 
    buildtree(axis,t1.tree.get_iter_mut(),ncontext.clone());

    {
        let kk=if height<3{
            0
        }else{
            height-3
        };
        let d=t1.tree.get_iter_mut().with_depth(Depth(0));
        recc(par::Parallel(Depth(kk)),axis,d,ncontext.clone());    
    }

    apply_tree(axis,t1.tree.get_iter_mut(),ncontext);
}


pub fn nbody<A:AxisTrait,N:NodeMassTrait>(t1:&mut DynTree<A,N::No,N::T>,ncontext:N){
    #[derive(Copy,Clone)]
    #[repr(transparent)]
    struct Wrap<T>(T);
    unsafe impl<T> Send for Wrap<T>{}
    impl<T:HasAabb> HasAabb for Wrap<T>{
        type Num=T::Num;
        fn get(&self)->&Rect<Self::Num>{
            self.0.get()
        }
    }

    #[derive(Copy,Clone)]
    #[repr(transparent)]
    struct Wrapper<N:NodeMassTrait>(N);
    
    unsafe impl<N:NodeMassTrait> Send for Wrapper<N>{}

    impl<N:NodeMassTrait> NodeMassTrait for Wrapper<N>{
        type T=Wrap<N::T>;
        type No=Wrap<N::No>;

        //gravitate this node mass with another node mass
        fn handle_node_with_node(&self,a:&mut Self::No,b:&mut Self::No){
            self.0.handle_node_with_node(&mut a.0,&mut b.0);
        }

        //gravitate a bot with a bot
        fn handle_bot_with_bot(&self,a:&mut Self::T,b:&mut Self::T){
            self.0.handle_bot_with_bot(&mut a.0,&mut b.0);
        }

        //gravitate a nodemass with a bot
        fn handle_node_with_bot(&self,a:&mut Self::No,b:&mut Self::T){
            self.0.handle_node_with_bot(&mut a.0,&mut b.0);
        }

        fn is_far_enough(&self,depth:usize,b:[<Self::T as HasAabb>::Num;2])->bool{
            self.0.is_far_enough(depth,b)
        }

        fn is_far_enough_half(&self,depth:usize,b:[<Self::T as HasAabb>::Num;2])->bool{
            self.0.is_far_enough_half(depth,b)
        }

        //This unloads the force accumulated by this node to the bots.
        fn apply_to_bots<'a,I:Iterator<Item=&'a mut Self::T>> (&'a self,a:&'a Self::No,it:I){
            self.0.apply_to_bots(&a.0,it.map(|a|&mut a.0))
        }

        fn new<'a,I:Iterator<Item=&'a Self::T>> (&'a self,it:I)->Self::No{
            Wrap(self.0.new(it.map(|b|&b.0)))
        }
        
    }


    let axis=t1.get_axis();//A::new();
    let ncontext=Wrapper(ncontext);
    let t1:&mut DynTree<A,Wrap<N::No>,Wrap<N::T>>=unsafe{std::mem::transmute(t1)};


    buildtree(axis,t1.tree.get_iter_mut(),ncontext.clone());

    {
        let d=t1.tree.get_iter_mut().with_depth(Depth(0));
        recc(par::Sequential,axis,d,ncontext.clone());    
    }

    apply_tree(axis,t1.tree.get_iter_mut(),ncontext);

}

