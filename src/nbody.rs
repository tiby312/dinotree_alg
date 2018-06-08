use inner_prelude::*;
use dinotree_inner::*;
//TODO somehow take advantage of sorted property?????




pub trait NBodyTrait:Clone{
    type N:NumTrait;
    type T;
    type No:Copy;

    fn create_empty(&self)->Self::No;

    //gravitate this node mass with another node mass
    fn handle_node_with_node(&self,&mut Self::No,b:&mut Self::No);

    //gravitate a bot with a bot
    fn handle_bot_with_bot(&self,BBoxDet<Self::N,Self::T>,BBoxDet<Self::N,Self::T>);

    //gravitate a nodemass with a bot
    fn handle_node_with_bot(&self,&mut Self::No,b:BBoxDet<Self::N,Self::T>);

    fn is_far_enough(&self,depth:usize,b:[Self::N;2])->bool;

    fn is_far_enough_half(&self,depth:usize,b:[Self::N;2])->bool;

    //This unloads the force accumulated by this node to the bots.
    fn apply_to_bots<'a,I:Iterator<Item=BBoxDet<'a,Self::N,Self::T>>> (&'a self,&'a Self::No,it:I);

    fn new<'a,I:Iterator<Item=&'a Self::T>> (&'a self,it:I)->Self::No;
}


struct Wrapper<N:NumTrait,T,K:NBodyTrait>{
    a:K,
    _p:PhantomData<(N,*const T)>
}

impl<N:NumTrait,T,K:NBodyTrait> Clone for Wrapper<N,T,K>{
    fn clone(&self)->Self{
        Wrapper{a:self.a.clone(),_p:PhantomData}
    }
}

impl<N:NumTrait,T,K:NBodyTrait<N=N,T=T>> NodeMassTrait for Wrapper<N,T,K>{
    type T=BBox<N,T>;
    type No=K::No;

    fn create_empty(&self)->Self::No{
        self.a.create_empty()
    }

    //gravitate this node mass with another node mass
    fn handle_node_with_node(&self,a:&mut Self::No,b:&mut Self::No){
        self.a.handle_node_with_node(a,b);
    }

    //gravitate a bot with a bot
    fn handle_bot_with_bot(&self,a:&mut Self::T,b:&mut Self::T){
        self.a.handle_bot_with_bot(a.destruct(),b.destruct())
    }

    //gravitate a nodemass with a bot
    fn handle_node_with_bot(&self,a:&mut Self::No,b:&mut Self::T){
        self.a.handle_node_with_bot(a,b.destruct())
    }

    fn is_far_enough(&self,depth:usize,b:[N;2])->bool{
        self.a.is_far_enough(depth,b)
    }

    fn is_far_enough_half(&self,depth:usize,b:[N;2])->bool{
        self.a.is_far_enough_half(depth,b)
    }

    //This unloads the force accumulated by this node to the bots.
    fn apply_to_bots<'a,I:Iterator<Item=&'a mut Self::T>> (&'a self,a:&'a Self::No,it:I){
        self.a.apply_to_bots(a,it.map(|a|a.destruct()))
    }

    fn new<'a,I:Iterator<Item=&'a Self::T>> (&'a self,it:I)->Self::No{
        self.new(it)
    }
}




mod tools{
    pub fn for_every_pair<T>(arr:&mut [T],mut func:impl FnMut(&mut T,&mut T)){
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



pub trait NodeMassTrait:Clone{
    type T:HasAabb;
    type No:Copy;

    fn create_empty(&self)->Self::No;

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
            Some((mut left,mut righ))=>{
                

                match nn.div{
                    Some(_div)=>{
                        
                        
                        let nodeb={
       
                            let left=left.create_wrap_mut();
                            let righ=righ.create_wrap_mut();

                                                    
                            let i1=left.dfs_preorder_iter().flat_map(|node|{node.range.iter()});
                            let i2=righ.dfs_preorder_iter().flat_map(|node|{node.range.iter()});
                            let i3=nn.range.iter().chain(i1.chain(i2));
                            
                            let mut nodeb=ncontext.new(i3);
                            

                            nodeb
                        };

                        nn.misc=nodeb;

                        let n2=ncontext.clone();
                        recc(axis.next(),left,ncontext);
                        recc(axis.next(),righ,n2);
                    },
                    None=>{
                        let mut nodeb=ncontext.new(nn.range.iter());
                        
                        nn.misc=nodeb;
                        let n2=ncontext.clone();
                        //recurse anyway even though there is no divider.
                        //we want to populate this tree entirely.
                        recc(axis.next(),left,ncontext);    
                        recc(axis.next(),righ,n2); 
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
    (axis:impl AxisTrait,node:NdIterMut<N::No,T>,ncontext:N){

    fn recc<'a,T:HasAabb+'a,N:NodeMassTrait<T=T>>
        (stuff:NdIterMut<N::No,T>,ncontext:N){

        let (nn1,rest)=stuff.next();
        
        let nodeb=&mut nn1.misc;
        match rest{
            Some((mut left,mut righ))=>{
                
                let _div=match nn1.div{
                    Some(div)=>{div},
                    None=>{return;}
                };

                
                {
                    let left=left.create_wrap_mut();
                    let righ=righ.create_wrap_mut();

                                            
                    let i1=left.dfs_preorder_iter().flat_map(|node|{node.range.iter_mut()});
                    let i2=righ.dfs_preorder_iter().flat_map(|node|{node.range.iter_mut()});
                    let i3=nn1.range.iter_mut().chain(i1.chain(i2));
                    
                    ncontext.apply_to_bots(nodeb,i3);
                }
                let n2=ncontext.clone();
                recc(left,ncontext);
                recc(righ,n2);
            },
            None=>{
                let l=nn1.range.len();
                ncontext.apply_to_bots(nodeb,nn1.range.iter_mut());
            }
        }
    }

    recc(node,ncontext);
}


//Construct anchor from cont!!!
struct Anchor<'a,A:AxisTrait,T:HasAabb+'a>{
	_axis:A,
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
        fn is_far_enough(&mut self,depth:Depth,b:[<Self::T as HasAabb>::Num;2])->bool{
            self.ncontext.is_far_enough(depth.0,b)
        }
    }
    let mut bo= Bo{_anchor_axis:B::new(),_p:PhantomData,ncontext:ncontext};
    generic_rec(thisa,anchor,left,&mut bo);  
    generic_rec(thisa,anchor,right,&mut bo);  
}

fn handle_left_with_right<'a,A:AxisTrait,B:AxisTrait,N:NodeMassTrait+'a>
    (_axis:A,anchor:&mut Anchor<B,N::T>,left:LevelIter<NdIterMut<'a,N::No,N::T>>,mut right:LevelIter<NdIterMut<'a,N::No,N::T>>,ncontext:&N){

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
        fn handle_every_node<A:AxisTrait>(&mut self,b:&mut N::T,_anchor:&mut Anchor<B,Self::T>){
            self.ncontext.handle_node_with_bot(self.node,b);
    	}
    	fn handle_far_enough<A:AxisTrait>(&mut self,a:&mut N::No,_anchor:&mut Anchor<B,Self::T>){
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
        fn handle_every_node<A:AxisTrait>(&mut self,b:&mut N::T,anchor:&mut Anchor<B,Self::T>){
    		let d=self.right.depth;
            let r=self.right.inner.create_wrap_mut().with_depth(d);
    		generic_rec(A::new(),anchor,r,&mut Bo4{_anchor_axis:B::new(),bot:b,ncontext:self.ncontext})
    	}
    	fn handle_far_enough<A:AxisTrait>(&mut self,a:&mut N::No,anchor:&mut Anchor<B,Self::T>){
    		let d=self.right.depth;
            let r=self.right.inner.create_wrap_mut().with_depth(d);
    		generic_rec(A::new(),anchor,r,&mut Bo2{_anchor_axis:B::new(),node:a,ncontext:self.ncontext})
    	}
        fn is_far_enough(&mut self,depth:Depth,b:[<Self::T as HasAabb>::Num;2])->bool{
            self.ncontext.is_far_enough_half(depth.0,b)
        }
    }
    let mut bo= Bo{_anchor_axis:B::new(),right:&mut right,ncontext};
    generic_rec(A::new(),anchor,left,&mut bo);  
}

fn recc<J:par::Joiner,A:AxisTrait,N:NodeMassTrait+Send>(join:J,axis:A,it:LevelIter<NdIterMut<N::No,N::T>>,ncontext:N) where N::T:Send,N::No:Send{
    let ((depth,nn1),rest)=it.next();
    

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
                    let depth=left.depth;
                    let l1=left.inner.create_wrap_mut().with_depth(depth);
                    let l2=right.inner.create_wrap_mut().with_depth(depth);
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
                let depth=left.depth;
                    
                let l1=left.inner.create_wrap_mut().with_depth(depth);
                let l2=right.inner.create_wrap_mut().with_depth(depth);
                let mut anchor=Anchor{_axis:axis,range:&mut nn1.range,div};

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
}



trait Bok{
    type N:NodeMassTrait<T=Self::T>;
    type T:HasAabb;
    type B:AxisTrait;
    fn is_far_enough(&mut self,depth:Depth,b:[<Self::T as HasAabb>::Num;2])->bool;
    fn handle_every_node<A:AxisTrait>(&mut self,n:&mut Self::T,anchor:&mut Anchor<Self::B,Self::T>);
    fn handle_far_enough<A:AxisTrait>(&mut self,a:&mut <Self::N as NodeMassTrait>::No,anchor:&mut Anchor<Self::B,Self::T>);
}


fn generic_rec<
    A:AxisTrait,
    AnchorAxis:AxisTrait,
    B:Bok<N=N,T=T,B=AnchorAxis>,
    N:NodeMassTrait<T=T>,
    T:HasAabb,
    >(this_axis:A,anchor:&mut Anchor<AnchorAxis,T>,stuff:LevelIter<NdIterMut<N::No,T>>,bok:&mut B){

        
    fn recc4<
        A:AxisTrait,
        AnchorAxis:AxisTrait,
        B:Bok<N=N,T=T,B=AnchorAxis>,
        N:NodeMassTrait<T=T>,
        T:HasAabb,
        >(axis:A,bok:&mut B,stuff:LevelIter<NdIterMut<N::No,T>>,anchor:&mut Anchor<AnchorAxis,T>){
        let ((depth,nn1),rest)=stuff.next();
        
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

    let ((depth,nn1),rest)=stuff.next();
    
    

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
                if bok.is_far_enough(depth,[div,anchor.div]){
                    bok.handle_far_enough::<A>(&mut nn1.misc,anchor);
                    return;
                }        
            }

            generic_rec(this_axis.next(),anchor,left,bok);
            generic_rec(this_axis.next(),anchor,right,bok);
        },
        None=>{

        }
    }       
}


pub fn nbody_par<A:AxisTrait,T:HasAabb+Send,N:NodeMassTrait<T=T>>(tree:DynTree<A,(),T>,ncontext:N)->DynTree<A,(),T> where N:Send,N::No:Send{
    let axis=A::new();
    let height=tree.get_height();
    
    //let mut t1=tree.create_copy(ncontext.create_empty());
    let mut t1=tree.with_extra(ncontext.create_empty());
    
    
    //let mut tree2=buildtree(tree,ncontext.clone());
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
    t1.with_extra(())
}


pub fn nbody_seq<A:AxisTrait,T:HasAabb+Send,N:NodeMassTrait<T=T>+Send>(tree:DynTree<A,(),T>,ncontext:N)->DynTree<A,(),T> where N::No:Send{
    let axis=A::new();

    let height=tree.get_height();
    
    use std::time::Instant;

    let timer=Instant::now();

    //let mut t1=tree.create_copy(ncontext.create_empty());
    let mut t1=tree.with_extra(ncontext.create_empty());
    println!("a={:?}",timer.elapsed());



    let timer=Instant::now();
    
    //let mut tree2=buildtree(tree,ncontext.clone());
    buildtree(axis,t1.tree.get_iter_mut(),ncontext.clone());

    println!("b={:?}",timer.elapsed());


    let timer=Instant::now();

    {
        let kk=if height<3{
            0
        }else{
            height-3
        };
        let d=t1.tree.get_iter_mut().with_depth(Depth(0));
        recc(par::Sequential,axis,d,ncontext.clone());    
    }
    println!("c={:?}",timer.elapsed());
    let timer=Instant::now();

    apply_tree(axis,t1.tree.get_iter_mut(),ncontext);
    println!("d={:?}",timer.elapsed());

    t1.with_extra(())

}

