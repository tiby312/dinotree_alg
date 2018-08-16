//!
//! # User Guide
//!
//! A nbody problem approximate solver. The user can choose the distance at which to fallback on approximate solutions.
//! The algorithm works similar to a Barnes–Hut simulation, but uses a kdtree instead of a quad tree.
//! 
//! A sequential and parallel version are supplied, both with a similar api:
//! ```
//! pub fn nbody<A:AxisTrait,N:NodeMassTraitMut>(
//!           t1:&mut DynTree<A,N::No,N::T>,
//!           ncontext:&mut N,
//!           rect:Rect<<N::T as HasAabb>::Num>){
//! ```
//! The user defines some geometric functions and their ideal accuracy. The user also supplies
//! a rectangle within which the nbody simulation will take place. So the simulation is only designed to work
//! in a finite area.
//!
//! # Safety
//!
//! There is unsafe code to reuse code between sequential and parallel versions.
//!
use inner_prelude::*;

///A mutable version that the user can take advantage of for debugging purposes.
///For example, the user can count how many node/node gravitations happened version bot/bot gravitations.
pub trait NodeMassTraitMut{
    type T:HasAabb;
    type No:Copy;

    ///Returns the bounding rectangle for this node.
    ///The rectangle returned here should be the rectangle supplied on creation of this node.
    fn get_rect(no:&Self::No)->&Rect<<Self::T as HasAabb>::Num>;

    ///Gravitate this node mass with another node mass
    fn handle_node_with_node(&mut self,&mut Self::No,b:&mut Self::No);

    ///Gravitate a bot with a bot
    fn handle_bot_with_bot(&mut self,&mut Self::T,&mut Self::T);

    ///Gravitate a nodemass with a bot
    fn handle_node_with_bot(&mut self,&mut Self::No,b:&mut Self::T);

    ///Return true if this distance if far away enough to use the node mass as an approximation.
    fn is_far_enough(&self,b:[<Self::T as HasAabb>::Num;2])->bool;

    ///Return true if this distance if far away enough to use the node mass as an approximation.
    fn is_far_enough_half(&self,b:[<Self::T as HasAabb>::Num;2])->bool;

    ///This unloads the force accumulated by this node to the bots.
    fn apply_to_bots<'a,I:Iterator<Item=&'a mut Self::T>> (&'a mut self,&'a Self::No,it:I);

    ///Create a new node mass.
    fn new<'a,I:Iterator<Item=&'a Self::T>> (&'a mut self,it:I,rect:Rect<<Self::T as HasAabb>::Num>)->Self::No;
}

///Use defined geometric functions to support the nbody function.
pub trait NodeMassTraitConst{
    type T:HasAabb;

    ///The nodemass. Every node in the tree has one. It's mass is equal to the sum of the masses
    ///of all the bots in that node and the nodes under it.
    type No:Copy;

    ///Returns the bounding rectangle for this node.
    //The rectangle returned here should be the rectangle supplied on creation of this node.
    fn get_rect(no:&Self::No)->&Rect<<Self::T as HasAabb>::Num>;

    ///Gravitate this node mass with another node mass
    fn handle_node_with_node(&self,&mut Self::No,b:&mut Self::No);

    ///Gravitate a bot with a bot
    fn handle_bot_with_bot(&self,&mut Self::T,&mut Self::T);

    ///Gravitate a nodemass with a bot
    fn handle_node_with_bot(&self,&mut Self::No,b:&mut Self::T);

    ///Return true if this distance if far away enough to use the node mass as an approximation.
    fn is_far_enough(&self,b:[<Self::T as HasAabb>::Num;2])->bool;

    ///Return true if this distance if far away enough to use the node mass as an approximation.
    fn is_far_enough_half(&self,b:[<Self::T as HasAabb>::Num;2])->bool;

    //This unloads the force accumulated by this node to the bots.
    fn apply_to_bots<'a,I:Iterator<Item=&'a mut Self::T>> (&'a self,&'a Self::No,it:I);

    ///Create a new node mass.
    fn new<'a,I:Iterator<Item=&'a Self::T>> (&'a self,it:I,rect:Rect<<Self::T as HasAabb>::Num>)->Self::No;
}




trait NodeMassTrait:Clone{
    type T:HasAabb;
    type No:Copy;

    //Returns the bounding rectangle for this node.
    fn get_rect(no:&Self::No)->&Rect<<Self::T as HasAabb>::Num>;

    //gravitate this node mass with another node mass
    fn handle_node_with_node(&mut self,&mut Self::No,b:&mut Self::No);

    //gravitate a bot with a bot
    fn handle_bot_with_bot(&mut self,&mut Self::T,&mut Self::T);

    //gravitate a nodemass with a bot
    fn handle_node_with_bot(&mut self,&mut Self::No,b:&mut Self::T);

    fn is_far_enough(&self,b:[<Self::T as HasAabb>::Num;2])->bool;

    fn is_far_enough_half(&self,b:[<Self::T as HasAabb>::Num;2])->bool;

    //This unloads the force accumulated by this node to the bots.
    fn apply_to_bots<'a,I:Iterator<Item=&'a mut Self::T>> (&'a mut self,&'a Self::No,it:I);

    fn new<'a,I:Iterator<Item=&'a Self::T>> (&'a mut self,it:I,rect:Rect<<Self::T as HasAabb>::Num>)->Self::No;
}


///Naive version simply visits every pair.
pub fn naive_mut<T:HasAabb>(bots:&mut [T],func:impl FnMut(&mut T,&mut T)){
    tools::for_every_pair(bots,func);
}



//pseudo code
//build up a tree where every nodemass has the mass of all the bots in that node and all the bots under it.
fn buildtree<'a,
    T:HasAabb+Send+'a,
    N:NodeMassTrait<T=T>
    >
    (axis:impl AxisTrait,node:NdIterMut<N::No,T>,ncontext:&mut N,rect:Rect<T::Num>){


    fn recc<'a,T:HasAabb+'a,N:NodeMassTrait<T=T>>
        (axis:impl AxisTrait,stuff:NdIterMut<N::No,T>,ncontext:&mut N,rect:Rect<T::Num>){
        
        let (nn,rest)=stuff.next();
        match rest{
            Some((extra,mut left,mut right))=>{

                match extra{
                    None=>{
                        let empty:&[T]=&[];
                        let mut nodeb=ncontext.new(empty.iter(),rect);
                        
                        nn.misc=nodeb;
                        //let n2=ncontext.clone();
                        //recurse anyway even though there is no divider.
                        //we want to populate this tree entirely.
                        recc(axis.next(),left,ncontext,rect);    
                        recc(axis.next(),right,ncontext,rect);
                    },
                    Some(FullComp{div,cont:_})=>{
                        let (l,r)=rect.subdivide(axis,*div);

                        let nodeb={
                            let left=left.create_wrap_mut();
                            let right=right.create_wrap_mut();

                            
                            let i1=left.dfs_preorder_iter().flat_map(|(node,_extra)|{node.range.iter()});
                            let i2=right.dfs_preorder_iter().flat_map(|(node,_extra)|{node.range.iter()});
                            let i3=nn.range.iter().chain(i1.chain(i2));
                            ncontext.new(i3,rect)
                        };

                        nn.misc=nodeb;

                        //let n2=ncontext.clone();
                        recc(axis.next(),left,ncontext,l);
                        recc(axis.next(),right,ncontext,r);
                    }
                }
            },
            None=>{
                let mut nodeb=ncontext.new(nn.range.iter(),rect);
                nn.misc=nodeb;
            }
        }
    }
    recc(axis,node,ncontext,rect);
}

fn apply_tree<'a,   
    T:HasAabb+'a,
    N:NodeMassTrait<T=T>
    >
    (_axis:impl AxisTrait,node:NdIterMut<N::No,T>,ncontext:&mut N){

    fn recc<'a,T:HasAabb+'a,N:NodeMassTrait<T=T>>
        (stuff:NdIterMut<N::No,T>,ncontext:&mut N){
        
        let (nn,rest)=stuff.next();
        match rest{
            Some((extra,mut left,mut right))=>{
                match extra{
                    Some(_)=>{
                        {
                            let left=left.create_wrap_mut();
                            let right=right.create_wrap_mut();
                                                    
                            let i1=left.dfs_preorder_iter().flat_map(|(node,_extra)|{node.range.iter_mut()});
                            let i2=right.dfs_preorder_iter().flat_map(|(node,_extra)|{node.range.iter_mut()});
                            let i3=nn.range.iter_mut().chain(i1.chain(i2));
                            
                            ncontext.apply_to_bots(&mut nn.misc,i3);
                        }
                    },
                    None=>{}
                };

                recc(left,ncontext);
                recc(right,ncontext);
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
(thisa:A,anchor:&mut Anchor<B,N::T>,left:LevelIter<NdIterMut<N::No,N::T>>,right:LevelIter<NdIterMut<N::No,N::T>>,ncontext:&mut N){
    

    struct BoLeft<'a,B:AxisTrait,N:NodeMassTrait+'a>{
        _anchor_axis:B,
        _p:PhantomData<N::No>,
        ncontext:&'a mut N
    }
    
    impl<'a,B:AxisTrait,N:NodeMassTrait+'a> Bok2 for BoLeft<'a,B,N>{
        type No=N::No;
        type T=N::T;
        type AnchorAxis=B;

        fn handle_node<A:AxisTrait>(&mut self,_axis:A,b:&mut N::T,anchor:&mut Anchor<B,Self::T>){
            for i in anchor.range.iter_mut(){
                self.ncontext.handle_bot_with_bot(i,b);
            }
        }
        fn handle_node_far_enough<A:AxisTrait>(&mut self,_axis:A,a:&mut N::No,anchor:&mut Anchor<B,Self::T>){
            for i in anchor.range.iter_mut(){
                self.ncontext.handle_node_with_bot(a,i);
            }
        }

        fn is_far_enough<A:AxisTrait>(&mut self,axis:A,anchor:&mut Anchor<B,Self::T>,misc:&Self::No)->bool{
            let rect=N::get_rect(misc);
            let range=rect.get_range(axis);
            self.ncontext.is_far_enough([anchor.div,range.right])
        }
    }
    struct BoRight<'a,B:AxisTrait,N:NodeMassTrait+'a>{
        _anchor_axis:B,
        _p:PhantomData<N::No>,
        ncontext:&'a mut N
    }
    
    impl<'a,B:AxisTrait,N:NodeMassTrait+'a> Bok2 for BoRight<'a,B,N>{
        type No=N::No;
        type T=N::T;
        type AnchorAxis=B;

        fn handle_node<A:AxisTrait>(&mut self,_axis:A,b:&mut N::T,anchor:&mut Anchor<B,Self::T>){
            for i in anchor.range.iter_mut(){
                self.ncontext.handle_bot_with_bot(i,b);
            }
        }
        fn handle_node_far_enough<A:AxisTrait>(&mut self,_axis:A,a:&mut N::No,anchor:&mut Anchor<B,Self::T>){
            for i in anchor.range.iter_mut(){
                self.ncontext.handle_node_with_bot(a,i);
            }
        }

        fn is_far_enough<A:AxisTrait>(&mut self,axis:A,anchor:&mut Anchor<B,Self::T>,misc:&Self::No)->bool{
            let rect=N::get_rect(misc);
            let range=rect.get_range(axis);
            self.ncontext.is_far_enough([anchor.div,range.left])
        }
    }
    {
        let mut bo= BoLeft{_anchor_axis:anchor.axis,_p:PhantomData,ncontext:ncontext};
        bo.generic_rec2(thisa,anchor,left);  
    }
    {
        let mut bo= BoRight{_anchor_axis:anchor.axis,_p:PhantomData,ncontext:ncontext};
        bo.generic_rec2(thisa,anchor,right);  
    }
}

fn handle_left_with_right<'a,A:AxisTrait,B:AxisTrait,N:NodeMassTrait+'a>
    (axis:A,anchor:&mut Anchor<B,N::T>,left:LevelIter<NdIterMut<'a,N::No,N::T>>,mut right:LevelIter<NdIterMut<'a,N::No,N::T>>,ncontext:&mut N){


	struct Bo4<'a,B:AxisTrait,N:NodeMassTrait+'a,>{
        _anchor_axis:B,
        bot:&'a mut N::T,
        ncontext:&'a mut N,
        div:<N::T as HasAabb>::Num
    }

    impl<'a,B:AxisTrait,N:NodeMassTrait+'a,> Bok2 for Bo4<'a,B,N>{
    	type No=N::No;
        type T=N::T;
        type AnchorAxis=B;
    	fn handle_node<A:AxisTrait>(&mut self,_axis:A,b:&mut Self::T,_anchor:&mut Anchor<B,Self::T>){
            self.ncontext.handle_bot_with_bot(self.bot,b);
    	}
    	fn handle_node_far_enough<A:AxisTrait>(&mut self,_axis:A,a:&mut N::No,_anchor:&mut Anchor<B,Self::T>){
    		self.ncontext.handle_node_with_bot(a,self.bot);
    	}
        fn is_far_enough<A:AxisTrait>(&mut self,axis:A,anchor:&mut Anchor<B,Self::T>,misc:&Self::No)->bool{
            let rect=N::get_rect(misc);
            let range=rect.get_range(axis);
            self.ncontext.is_far_enough_half([self.div,range.left])
        }
    }
    struct Bo2<'a,B:AxisTrait,N:NodeMassTrait+'a>{
        _anchor_axis:B,
        node:&'a mut N::No,
        ncontext:&'a mut N,
        div:<N::T as HasAabb>::Num
    }

    impl<'a,B:AxisTrait,N:NodeMassTrait+'a> Bok2 for Bo2<'a,B,N>{
    	type No=N::No;
        type T=N::T;
        type AnchorAxis=B;
        fn handle_node<A:AxisTrait>(&mut self,_axis:A,b:&mut N::T,_anchor:&mut Anchor<B,Self::T>){
            self.ncontext.handle_node_with_bot(self.node,b);
    	}
    	fn handle_node_far_enough<A:AxisTrait>(&mut self,_axis:A,a:&mut N::No,_anchor:&mut Anchor<B,Self::T>){
    		self.ncontext.handle_node_with_node(self.node,a);
    	}
        fn is_far_enough<A:AxisTrait>(&mut self,axis:A,anchor:&mut Anchor<B,Self::T>,misc:&Self::No)->bool{
            let rect=N::get_rect(misc);
            let range=rect.get_range(axis);
            self.ncontext.is_far_enough_half([self.div,range.left])
        }
    }

    struct Bo<'a:'b,'b,B:AxisTrait,N:NodeMassTrait+'a>{
        _anchor_axis:B,
        right:&'b mut LevelIter<NdIterMut<'a,N::No,N::T>>,
        ncontext:&'b mut N
    }
    
    impl<'a:'b,'b,B:AxisTrait,N:NodeMassTrait+'a> Bok2 for Bo<'a,'b,B,N>{
    	type No=N::No;
        type T=N::T;
        type AnchorAxis=B;
        fn handle_node<A:AxisTrait>(&mut self,axis:A,b:&mut N::T,anchor:&mut Anchor<B,Self::T>){
    		let d=self.right.depth;
            let r=self.right.inner.create_wrap_mut().with_depth(d);
            let anchor_axis=anchor.axis;

            let right_most=b.get().get_range(axis).right;

            let mut bok=Bo4{_anchor_axis:anchor_axis,bot:b,ncontext:self.ncontext,div:anchor.div};
            bok.generic_rec2(axis,anchor,r);
    	}
    	fn handle_node_far_enough<A:AxisTrait>(&mut self,axis:A,a:&mut N::No,anchor:&mut Anchor<B,Self::T>){
    		let d=self.right.depth;
            let r=self.right.inner.create_wrap_mut().with_depth(d);
            let anchor_axis=anchor.axis;

            let right_most=N::get_rect(a).get_range(axis).right;
            let mut bok=Bo2{_anchor_axis:anchor_axis,node:a,ncontext:self.ncontext,div:anchor.div};
            bok.generic_rec2(axis,anchor,r);
    	}
        fn is_far_enough<A:AxisTrait>(&mut self,axis:A,anchor:&mut Anchor<B,Self::T>,misc:&Self::No)->bool{
            let rect=N::get_rect(misc);
            let range=rect.get_range(axis);
            self.ncontext.is_far_enough_half([range.right,anchor.div])
        }
    }
    let mut bo= Bo{_anchor_axis:anchor.axis,right:&mut right,ncontext};
    bo.generic_rec2(axis,anchor,left); 
    
}

fn recc<J:par::Joiner,A:AxisTrait,N:NodeMassTrait+Send>(join:J,axis:A,it:LevelIter<NdIterMut<N::No,N::T>>,ncontext:&mut N) where N::T:Send,N::No:Send{
    

    let ((depth,nn),rest)=it.next();
    match rest{
        Some((extra,mut left,mut right))=>{
            let &FullComp{div,cont:_}=match extra{
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

                handle_anchor_with_children(axis.next(),&mut anchor,l1,l2,ncontext);
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

                handle_left_with_right(axis.next(),&mut anchor,l1,l2,ncontext);
            }
            //at this point we have successfully broken up this problem
            //into two independant ones, and we can do this all over again for the two children.
            //potentially in parlalel.
           
            if join.should_switch_to_sequential(depth){
                recc(join.into_seq(),axis.next(),left,ncontext);
                recc(join.into_seq(),axis.next(),right,ncontext);
            }else{
                let mut n2=ncontext.clone();
                rayon::join(
                ||recc(join,axis.next(),left,ncontext),
                ||recc(join,axis.next(),right,&mut n2)
                );
            }
        },
        None=>{
            //handle bots in itself
            tools::for_every_pair(&mut nn.range,|a,b|{ncontext.handle_bot_with_bot(a,b)});
        }
    }
}





trait Bok2{
    type No:Copy;
    type T:HasAabb;
    type AnchorAxis:AxisTrait;
    fn is_far_enough<A:AxisTrait>(&mut self,axis:A,anchor:&mut Anchor<Self::AnchorAxis,Self::T>,misc:&Self::No)->bool;
    fn handle_node<A:AxisTrait>(&mut self,axis:A,n:&mut Self::T,anchor:&mut Anchor<Self::AnchorAxis,Self::T>);
    fn handle_node_far_enough<A:AxisTrait>(&mut self,axis:A,a:&mut Self::No,anchor:&mut Anchor<Self::AnchorAxis,Self::T>);


    fn generic_rec2<
        A:AxisTrait,
        >(&mut self,this_axis:A,anchor:&mut Anchor<Self::AnchorAxis,Self::T>,stuff:LevelIter<NdIterMut<Self::No,Self::T>>){

        let ((depth,nn),rest)=stuff.next();
        
        if this_axis.is_equal_to(anchor.axis){
            if self.is_far_enough(this_axis,anchor,&nn.misc){
                self.handle_node_far_enough(this_axis,&mut nn.misc,anchor);
                return;
            }        
        }

        match rest{
            Some((extra,left,right))=>{
                let &FullComp{div,cont}=match extra{
                    Some(b)=>b,
                    None=>return
                };
                
                for i in nn.range.iter_mut(){
                    self.handle_node(this_axis,i,anchor);    
                }

                self.generic_rec2(this_axis.next(),anchor,left);
                self.generic_rec2(this_axis.next(),anchor,right);
            },
            None=>{
                for i in nn.range.iter_mut(){
                    self.handle_node(this_axis,i,anchor);    
                }
            }
        }
    }

}


///Parallel version.
pub fn nbody_par<A:AxisTrait,T:HasAabb+Send,N:NodeMassTraitConst<T=T>+Sync>(t1:&mut DynTree<A,N::No,T>,ncontext:&N,rect:Rect<<N::T as HasAabb>::Num>) where N::No:Send{
    let axis=t1.get_axis();
    let height=t1.get_height();
 
    struct Wrapper<'a,N:NodeMassTraitConst+'a>(&'a N);
    impl<'a,N:NodeMassTraitConst+'a> Clone for Wrapper<'a,N>{
        fn clone(&self)->Wrapper<'a,N>{
            Wrapper(self.0)
        }
    }
    impl<'a,N:NodeMassTraitConst+'a> NodeMassTrait for Wrapper<'a,N>{
        type T=N::T;
        type No=N::No;

        //Returns the bounding rectangle for this node.
        fn get_rect(no:&Self::No)->&Rect<<Self::T as HasAabb>::Num>{
            N::get_rect(no)            
        }

        //gravitate this node mass with another node mass
        fn handle_node_with_node(&mut self,a:&mut Self::No,b:&mut Self::No){
            self.0.handle_node_with_node(a,b);
        }

        //gravitate a bot with a bot
        fn handle_bot_with_bot(&mut self,a:&mut Self::T,b:&mut Self::T){
            self.0.handle_bot_with_bot(a,b);
        }

        //gravitate a nodemass with a bot
        fn handle_node_with_bot(&mut self,a:&mut Self::No,b:&mut Self::T){
            self.0.handle_node_with_bot(a,b);
        }

        fn is_far_enough(&self,b:[<Self::T as HasAabb>::Num;2])->bool{
            self.0.is_far_enough(b)
        }

        fn is_far_enough_half(&self,b:[<Self::T as HasAabb>::Num;2])->bool{
            self.0.is_far_enough_half(b)
        }

        //This unloads the force accumulated by this node to the bots.
        fn apply_to_bots<'b,I:Iterator<Item=&'b mut Self::T>> (&'b mut self,a:&'b Self::No,it:I){
            self.0.apply_to_bots(a,it)
        }

        fn new<'b,I:Iterator<Item=&'b Self::T>> (&'b mut self,it:I,rect:Rect<<Self::T as HasAabb>::Num>)->Self::No{
            self.0.new(it,rect)
        }
        
    }


    let mut ncontext=Wrapper(ncontext);
    buildtree(axis,t1.get_iter_mut(),&mut ncontext,rect);

    {
        let kk=if height<3{
            0
        }else{
            height-3
        };
        let d=t1.get_iter_mut().with_depth(Depth(0));
        recc(par::Parallel(Depth(kk)),axis,d,&mut ncontext);    
    }

    apply_tree(axis,t1.get_iter_mut(),&mut ncontext);
}


///Sequential version.
pub fn nbody<A:AxisTrait,N:NodeMassTraitMut>(t1:&mut DynTree<A,N::No,N::T>,ncontext:&mut N,rect:Rect<<N::T as HasAabb>::Num>){
    
    #[derive(Copy,Clone)]
    #[repr(transparent)]
    struct Wrap<T>(T);
    unsafe impl<T> Send for Wrap<T>{}
    unsafe impl<T:HasAabb> HasAabb for Wrap<T>{
        type Num=T::Num;
        fn get(&self)->&Rect<Self::Num>{
            self.0.get()
        }
    }

    #[repr(transparent)]
    struct Wrapper<'a,N:NodeMassTraitMut+'a>(&'a mut N);
    impl<'a,N:NodeMassTraitMut+'a> Clone for Wrapper<'a,N>{
        fn clone(&self)->Wrapper<'a,N>{
            unreachable!()
        }
    }

    unsafe impl<'a,N:NodeMassTraitMut> Send for Wrapper<'a,N>{}

    impl<'a,N:NodeMassTraitMut+'a> NodeMassTrait for Wrapper<'a,N>{
        type T=Wrap<N::T>;
        type No=Wrap<N::No>;


        //Returns the bounding rectangle for this node.
        fn get_rect(no:&Self::No)->&Rect<<Self::T as HasAabb>::Num>{
            N::get_rect(&no.0)            
        }


        //gravitate this node mass with another node mass
        fn handle_node_with_node(&mut self,a:&mut Self::No,b:&mut Self::No){
            self.0.handle_node_with_node(&mut a.0,&mut b.0);
        }

        //gravitate a bot with a bot
        fn handle_bot_with_bot(&mut self,a:&mut Self::T,b:&mut Self::T){
            self.0.handle_bot_with_bot(&mut a.0,&mut b.0);
        }

        //gravitate a nodemass with a bot
        fn handle_node_with_bot(&mut self,a:&mut Self::No,b:&mut Self::T){
            self.0.handle_node_with_bot(&mut a.0,&mut b.0);
        }

        fn is_far_enough(&self,b:[<Self::T as HasAabb>::Num;2])->bool{
            self.0.is_far_enough(b)
        }

        fn is_far_enough_half(&self,b:[<Self::T as HasAabb>::Num;2])->bool{
            self.0.is_far_enough_half(b)
        }

        //This unloads the force accumulated by this node to the bots.
        fn apply_to_bots<'b,I:Iterator<Item=&'b mut Self::T>> (&'b mut self,a:&'b Self::No,it:I){
            self.0.apply_to_bots(&a.0,it.map(|a|&mut a.0))
        }

        fn new<'b,I:Iterator<Item=&'b Self::T>> (&'b mut self,it:I,rect:Rect<<Self::T as HasAabb>::Num>)->Self::No{
            Wrap(self.0.new(it.map(|b|&b.0),rect))
        }
        
    }


    let axis=t1.get_axis();//A::new();
    let mut ncontext=Wrapper(ncontext);
    let t1:&mut DynTree<A,Wrap<N::No>,Wrap<N::T>>=unsafe{std::mem::transmute(t1)};


    buildtree(axis,t1.get_iter_mut(),&mut ncontext,rect);

    {
        let d=t1.get_iter_mut().with_depth(Depth(0));
        recc(par::Sequential,axis,d,&mut ncontext);    
    }

    apply_tree(axis,t1.get_iter_mut(),&mut ncontext);

}

