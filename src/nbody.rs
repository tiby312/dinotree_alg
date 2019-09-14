//!
//! # User Guide
//!
//! A nbody problem approximate solver. The user can choose the distance at which to fallback on approximate solutions.
//! The algorithm works similar to a Barnes–Hut simulation, but uses a kdtree instead of a quad tree.
//! 
//! A sequential and parallel version are supplied, both with a similar api.
//!
//! The user defines some geometric functions and their ideal accuracy. The user also supplies
//! a rectangle within which the nbody simulation will take place. So the simulation is only designed to work
//! in a finite area.
//!
use crate::inner_prelude::*;


pub trait NodeMassTrait:Clone{
    type No:Copy+Send;
    type Num:NumTrait;
    type Item:HasAabb<Num=Self::Num>;

    //Returns the bounding rectangle for this node.
    fn get_rect(no:&Self::No)->&Rect<Self::Num>;

    //gravitate this node mass with another node mass
    fn handle_node_with_node(&self,a:&mut Self::No,b:&mut Self::No);

    //gravitate a bot with a bot
    fn handle_bot_with_bot(&self,a:ProtectedBBox<Self::Item>,b:ProtectedBBox<Self::Item>);

    //gravitate a nodemass with a bot
    fn handle_node_with_bot(&self,a:&mut Self::No,b:ProtectedBBox<Self::Item>);

    fn is_far_enough(&self,b:[Self::Num;2])->bool;

    fn is_far_enough_half(&self,b:[Self::Num;2])->bool;

    //This unloads the force accumulated by this node to the bots.
    fn apply_to_bots<'a,I:Iterator<Item=ProtectedBBox<'a,Self::Item>>> (&'a self,a:&'a Self::No,it:I);

    fn new<'a,I:Iterator<Item=&'a Self::Item>> (&'a self,it:I,rect:Rect<Self::Num>)->Self::No;
}


///Naive version simply visits every pair.
pub fn naive_mut<T:HasAabb>(bots:&mut [T],func:impl FnMut(ProtectedBBox<T>,ProtectedBBox<T>)){
    let bots=ProtectedBBoxSlice::new(bots);
    tools::for_every_pair(bots,func);
}


use compt::dfs_order;
type CombinedVistr<'a,N,J> = compt::LevelIter<compt::Zip<dfs_order::Vistr<'a,N,dfs_order::PreOrder>,VistrMut<'a,J>>>;
type CombinedVistrMut<'a,N,J> = compt::LevelIter<compt::Zip<dfs_order::VistrMut<'a,N,dfs_order::PreOrder>,VistrMut<'a,J>>>;


fn wrap_mut<'a:'b,'b,N,J:NodeTrait>(bla:&'b mut CombinedVistrMut<'a,N,J>)->CombinedVistrMut<'b,N,J>{
    let depth=bla.depth();

    let (a,b)=bla.as_inner_mut().as_inner_mut();

    let a=a.create_wrap_mut();
    let b=b.create_wrap_mut();

    a.zip(b).with_depth(Depth(depth))
}

//pseudo code
//build up a tree where every nodemass has the mass of all the bots in that node and all the bots under it.
fn buildtree<
    J:NodeTrait,
    N:NodeMassTrait<Num=J::Num,Item=J::T>
    >
    (axis:impl AxisTrait,node:VistrMut<J>,misc_nodes:&mut Vec<N::No>,ncontext:&N,rect:Rect<J::Num>){


    fn recc<J:NodeTrait,N:NodeMassTrait<Num=J::Num,Item=J::T>>
        (axis:impl AxisTrait,stuff:VistrMut<J>,misc_nodes:&mut Vec<N::No>,ncontext:&N,rect:Rect<J::Num>){
        
        let (nn,rest)=stuff.next();
        let nn=nn.get_mut();
        match rest{
            Some([left,right])=>{

                match nn.div{
                    None=>{
                        //let empty=&[];
                        //misc_nodes.push(ncontext.new(empty.iter(),rect));
                        
                        //recurse anyway even though there is no divider.
                        //we want to populate this tree entirely.
                        recc(axis.next(),left,misc_nodes,ncontext,rect);    
                        recc(axis.next(),right,misc_nodes,ncontext,rect);
                    },
                    Some(div)=>{
                        let (l,r)=rect.subdivide(axis,*div);

                        let nodeb={
                            let i1=left.create_wrap().dfs_preorder_iter().flat_map(|a|a.get().bots.iter());
                            let i2=right.create_wrap().dfs_preorder_iter().flat_map(|a|a.get().bots.iter());
                            let i3=nn.bots.iter().chain(i1.chain(i2));
                            ncontext.new(i3,rect)
                        };

                        misc_nodes.push(nodeb);
                        
                        recc(axis.next(),left,misc_nodes,ncontext,l);
                        recc(axis.next(),right,misc_nodes,ncontext,r);
                    }
                }
            },
            None=>{
                misc_nodes.push(ncontext.new(nn.bots.iter(),rect));
            }
        }
    }
    recc(axis,node,misc_nodes,ncontext,rect);
}

fn apply_tree<   
    N:NodeMassTrait<Num=J::Num,Item=J::T>,
    J:NodeTrait,
    >
    (_axis:impl AxisTrait,node:CombinedVistr<N::No,J>,ncontext:&N){

    fn recc<N:NodeMassTrait<Num=J::Num,Item=J::T>,J:NodeTrait>
        (stuff:CombinedVistr<N::No,J>,ncontext:&N){
        
        let ((_,(misc,nn)),rest)=stuff.next();
        let nn=nn.get_mut();
        match rest{
            Some([mut left,mut right])=>{

                let i1=left.as_inner_mut().as_inner_mut().1.create_wrap_mut().dfs_preorder_iter().flat_map(|a|a.get_mut().bots.iter_mut());
                let i2=right.as_inner_mut().as_inner_mut().1.create_wrap_mut().dfs_preorder_iter().flat_map(|a|a.get_mut().bots.iter_mut());
                let i3=nn.bots.iter_mut().chain(i1.chain(i2));
                

                ncontext.apply_to_bots(misc,i3);

                recc(left,ncontext);
                recc(right,ncontext);
            },
            None=>{
                ncontext.apply_to_bots(misc,nn.bots.iter_mut());
            }
        }
    }

    recc(node,ncontext);
}


//Construct anchor from cont!!!
struct Anchor<'a,A:AxisTrait,N:NodeTrait>{
	axis:A,
    range:ProtectedBBoxSlice<'a,N::T>,
    div:N::Num
}

fn handle_anchor_with_children<'a,
	A:AxisTrait,
	B:AxisTrait,
    N:NodeMassTrait<Num=J::Num,Item=J::T>,
    J:NodeTrait>
(thisa:A,anchor:&mut Anchor<B,J>,left:CombinedVistrMut<N::No,J>,right:CombinedVistrMut<N::No,J>,ncontext:&N){
    

    struct BoLeft<'a,B:AxisTrait,N:NodeMassTrait,J:NodeTrait>{
        _anchor_axis:B,
        _p:PhantomData<(N::No,J)>,
        ncontext:&'a N,
    }
    
    impl<'a,B:AxisTrait,N:NodeMassTrait<Num=J::Num,Item=J::T>,J:NodeTrait> Bok2 for BoLeft<'a,B,N,J>{
        type No=N::No;
        type T=J::T;
        type J=J;
        type AnchorAxis=B;

        fn handle_node<A:AxisTrait>(&mut self,_axis:A,mut b:ProtectedBBox<J::T>,anchor:&mut Anchor<B,J>){
            for i in anchor.range.as_mut().iter_mut(){
                self.ncontext.handle_bot_with_bot(i,b.as_mut());
            }
        }
        fn handle_node_far_enough<A:AxisTrait>(&mut self,_axis:A,a:&mut N::No,anchor:&mut Anchor<B,J>){
            for i in anchor.range.as_mut().iter_mut(){
                self.ncontext.handle_node_with_bot(a,i);
            }
        }

        fn is_far_enough<A:AxisTrait>(&mut self,axis:A,anchor:&mut Anchor<B,J>,misc:&Self::No)->bool{
            let rect=N::get_rect(misc);
            let range=rect.get_range(axis);
            self.ncontext.is_far_enough([anchor.div,range.right])
        }
    }

    struct BoRight<'a,B:AxisTrait,N:NodeMassTrait,J:NodeTrait>{
        _anchor_axis:B,
        _p:PhantomData<(N::No,J)>,
        ncontext:&'a N
    }
    
    impl<'a,B:AxisTrait,N:NodeMassTrait<Num=J::Num,Item=J::T>,J:NodeTrait> Bok2 for BoRight<'a,B,N,J>{
        type No=N::No;
        type T=J::T;
        type J=J;
        type AnchorAxis=B;

        fn handle_node<A:AxisTrait>(&mut self,_axis:A,mut b:ProtectedBBox<J::T>,anchor:&mut Anchor<B,J>){
            for i in anchor.range.as_mut().iter_mut(){
                self.ncontext.handle_bot_with_bot(i,b.as_mut());
            }
        }
        fn handle_node_far_enough<A:AxisTrait>(&mut self,_axis:A,a:&mut N::No,anchor:&mut Anchor<B,J>){
            for i in anchor.range.as_mut().iter_mut(){
                self.ncontext.handle_node_with_bot(a,i);
            }
        }

        fn is_far_enough<A:AxisTrait>(&mut self,axis:A,anchor:&mut Anchor<B,J>,misc:&Self::No)->bool{
            let rect=N::get_rect(misc);
            let range=rect.get_range(axis);
            self.ncontext.is_far_enough([anchor.div,range.left])
        }
    }
    {
        let mut bo= BoLeft{_anchor_axis:anchor.axis,_p:PhantomData,ncontext};
        bo.generic_rec2(thisa,anchor,left);  
    }
    {
        let mut bo= BoRight{_anchor_axis:anchor.axis,_p:PhantomData,ncontext};
        bo.generic_rec2(thisa,anchor,right);  
    }
}

fn handle_left_with_right<'a,A:AxisTrait,B:AxisTrait,N:NodeMassTrait<Num=J::Num,Item=J::T>,J:NodeTrait>
    (axis:A,anchor:&mut Anchor<B,J>,left:CombinedVistrMut<'a,N::No,J>,mut right:CombinedVistrMut<'a,N::No,J>,ncontext:&N){


	struct Bo4<'a,B:AxisTrait,N:NodeMassTrait,J:NodeTrait>{
        _anchor_axis:B,
        bot:ProtectedBBox<'a,J::T>,
        ncontext:&'a N,
        div:N::Num,
        _p:PhantomData<J>
    }

    impl<'a,B:AxisTrait,N:NodeMassTrait<Num=J::Num,Item=J::T>,J:NodeTrait> Bok2 for Bo4<'a,B,N,J>{
    	type No=N::No;
        type T=J::T;
        type J=J;
        type AnchorAxis=B;
    	fn handle_node<A:AxisTrait>(&mut self,_axis:A,b:ProtectedBBox<J::T>,_anchor:&mut Anchor<B,J>){
            self.ncontext.handle_bot_with_bot(self.bot.as_mut(),b);
    	}
    	fn handle_node_far_enough<A:AxisTrait>(&mut self,_axis:A,a:&mut N::No,_anchor:&mut Anchor<B,J>){
    		self.ncontext.handle_node_with_bot(a,self.bot.as_mut());
    	}
        fn is_far_enough<A:AxisTrait>(&mut self,axis:A,_anchor:&mut Anchor<B,Self::J>,misc:&Self::No)->bool{
            let rect=N::get_rect(misc);
            let range=rect.get_range(axis);
            self.ncontext.is_far_enough_half([self.div,range.left])
        }
    }
    struct Bo2<'a,B:AxisTrait,N:NodeMassTrait,J:NodeTrait>{
        _anchor_axis:B,
        node:&'a mut N::No,
        ncontext:&'a N,
        div:N::Num,
        _p:PhantomData<J>
    }

    impl<'a,B:AxisTrait,N:NodeMassTrait<Num=J::Num,Item=J::T>,J:NodeTrait> Bok2 for Bo2<'a,B,N,J>{
    	type No=N::No;
        type T=J::T;
        type J=J;
        type AnchorAxis=B;

        fn handle_node<A:AxisTrait>(&mut self,_axis:A,b:ProtectedBBox<J::T>,_anchor:&mut Anchor<B,J>){
            self.ncontext.handle_node_with_bot(self.node,b);
    	}
    	fn handle_node_far_enough<A:AxisTrait>(&mut self,_axis:A,a:&mut N::No,_anchor:&mut Anchor<B,J>){
    		self.ncontext.handle_node_with_node(self.node,a);
    	}
        fn is_far_enough<A:AxisTrait>(&mut self,axis:A,_anchor:&mut Anchor<B,J>,misc:&Self::No)->bool{
            let rect=N::get_rect(misc);
            let range=rect.get_range(axis);
            self.ncontext.is_far_enough_half([self.div,range.left])
        }
    }

    struct Bo<'a:'b,'b,B:AxisTrait,N:NodeMassTrait,J:NodeTrait>{
        _anchor_axis:B,
        right:&'b mut CombinedVistrMut<'a,N::No,J>,
        ncontext:&'b N
    }
    
    impl<'a:'b,'b,B:AxisTrait,N:NodeMassTrait<Num=J::Num,Item=J::T>,J:NodeTrait> Bok2 for Bo<'a,'b,B,N,J>{
    	type No=N::No;
        type T=J::T;
        type J=J;
        type AnchorAxis=B;
        fn handle_node<A:AxisTrait>(&mut self,axis:A,b:ProtectedBBox<J::T>,anchor:&mut Anchor<B,J>){
            let r=wrap_mut(&mut self.right);
            let anchor_axis=anchor.axis;

            let mut bok=Bo4{_anchor_axis:anchor_axis,bot:b,ncontext:self.ncontext,div:anchor.div,_p:PhantomData};
            bok.generic_rec2(axis,anchor,r);
    	}
    	fn handle_node_far_enough<A:AxisTrait>(&mut self,axis:A,a:&mut N::No,anchor:&mut Anchor<B,J>){
            let r=wrap_mut(&mut self.right);
            let anchor_axis=anchor.axis;

            let mut bok=Bo2{_anchor_axis:anchor_axis,node:a,ncontext:self.ncontext,div:anchor.div,_p:PhantomData};
            bok.generic_rec2(axis,anchor,r);
    	}
        fn is_far_enough<A:AxisTrait>(&mut self,axis:A,anchor:&mut Anchor<B,J>,misc:&Self::No)->bool{
            let rect=N::get_rect(misc);
            let range=rect.get_range(axis);
            self.ncontext.is_far_enough_half([range.right,anchor.div])
        }
    }
    let mut bo= Bo{_anchor_axis:anchor.axis,right:&mut right,ncontext};
    bo.generic_rec2(axis,anchor,left); 
    
}


fn recc<J:par::Joiner,A:AxisTrait,N:NodeMassTrait<Num=F::Num,Item=F::T>+Sync+Send,F:NodeTrait+Send+Sync>(join:J,axis:A,it:CombinedVistrMut<N::No,F>,ncontext:&N) where F::T:Send,N::No:Send{
    

    let ((depth,(_,mut nn)),rest)=it.next();
    let mut nn=nn.get_mut();
    match rest{
        Some([mut left,mut right])=>{
            let div=match nn.div{
                Some(b)=>b,
                None=>return
            };

            //handle bots in itself
            tools::for_every_pair(nn.bots.as_mut(),|a,b|{ncontext.handle_bot_with_bot(a,b)});
            {
                let l1=wrap_mut(&mut left);
                let l2=wrap_mut(&mut right);
                let mut anchor=Anchor{axis,range:nn.bots.as_mut(),div:*div};

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
                let l1=wrap_mut(&mut left);
                let l2=wrap_mut(&mut right);
                let mut anchor=Anchor{axis,range:nn.bots,div:*div};

                handle_left_with_right(axis.next(),&mut anchor,l1,l2,ncontext);
            }
            //at this point we have successfully broken up this problem
            //into two independant ones, and we can do this all over again for the two children.
            //potentially in parlalel.
           
            match join.next(depth){
                par::ParResult::Parallel([dleft,dright])=>{
                    let mut n2=ncontext.clone();
                    rayon::join(
                    ||recc(dleft,axis.next(),left,ncontext),
                    ||recc(dright,axis.next(),right,&mut n2)
                    );
                },
                par::ParResult::Sequential([dleft,dright])=>{

                    recc(dleft,axis.next(),left,ncontext);
                    recc(dright,axis.next(),right,ncontext);
                }
            }
        },
        None=>{
            //handle bots in itself
            tools::for_every_pair(nn.bots,|a,b|{ncontext.handle_bot_with_bot(a,b)});
        }
    }
}





trait Bok2{
    type No:Copy;
    type J:NodeTrait<T=Self::T,Num=<Self::T as HasAabb>::Num>;
    type T:HasAabb;
    type AnchorAxis:AxisTrait;
    fn is_far_enough<A:AxisTrait>(&mut self,axis:A,anchor:&mut Anchor<Self::AnchorAxis,Self::J>,misc:&Self::No)->bool;
    fn handle_node<A:AxisTrait>(&mut self,axis:A,n:ProtectedBBox<Self::T>,anchor:&mut Anchor<Self::AnchorAxis,Self::J>);
    fn handle_node_far_enough<A:AxisTrait>(&mut self,axis:A,a:&mut Self::No,anchor:&mut Anchor<Self::AnchorAxis,Self::J>);


    fn generic_rec2<
        A:AxisTrait,
        >(&mut self,this_axis:A,anchor:&mut Anchor<Self::AnchorAxis,Self::J>,stuff:CombinedVistrMut<Self::No,Self::J>){

        let ((_depth,(misc,nn)),rest)=stuff.next();
        let nn=nn.get_mut();
        if this_axis.is_equal_to(anchor.axis) && self.is_far_enough(this_axis,anchor,misc){
            self.handle_node_far_enough(this_axis,misc,anchor);
            return;
        }

        match rest{
            Some([left,right])=>{
                match nn.div{
                    Some(_)=>(),
                    None=>return
                };
                
                for i in nn.bots.iter_mut(){
                    self.handle_node(this_axis,i,anchor);    
                }

                self.generic_rec2(this_axis.next(),anchor,left);
                self.generic_rec2(this_axis.next(),anchor,right);
            },
            None=>{
                for i in nn.bots.iter_mut(){
                    self.handle_node(this_axis,i,anchor);    
                }
            }
        }
    }

}


///Parallel version.
pub fn nbody_par<A:AxisTrait,J:NodeTrait+Send+Sync,N:NodeMassTrait<Num=J::Num,Item=J::T>+Sync+Send>(t1:&mut DinoTree<A,J>,ncontext:&N,rect:Rect<J::Num>) where N::No:Send, J::T:Send+Copy{
    let axis=t1.axis();
    
    let mut misc_nodes=Vec::new();
    buildtree(axis,t1.vistr_mut(),&mut misc_nodes,ncontext,rect);

    let mut misc_tree=compt::dfs_order::CompleteTreeContainer::from_preorder(misc_nodes).unwrap();

    {
        let k=default_level_switch_sequential();
        let par=compute_default_level_switch_sequential(k,t1.get_height());

        let d=misc_tree.vistr_mut().zip(t1.vistr_mut()).with_depth(Depth(0));
        recc(par,axis,d,ncontext);    
    }

    apply_tree(axis,misc_tree.vistr().zip(t1.vistr_mut()).with_depth(Depth(0)),ncontext);
}


///Sequential version.
pub fn nbody<A:AxisTrait,J:NodeTrait+Send+Sync,N:NodeMassTrait<Num=J::Num,Item=J::T>+Send+Sync>(t1:&mut DinoTree<A,J>,ncontext:&N,rect:Rect<J::Num>) where J::T:Send+Sync{
    
    let axis=t1.axis();
    
    let mut misc_nodes=Vec::new();
    
    buildtree(axis,t1.vistr_mut(),&mut misc_nodes,ncontext,rect);

    let mut misc_tree=compt::dfs_order::CompleteTreeContainer::from_preorder(misc_nodes).unwrap();

    let d=misc_tree.vistr_mut().zip(t1.vistr_mut()).with_depth(Depth(0));        
    recc(par::Sequential,axis,d,ncontext);    

    let d=misc_tree.vistr().zip(t1.vistr_mut()).with_depth(Depth(0));
    apply_tree(axis,d,ncontext);
    
}

