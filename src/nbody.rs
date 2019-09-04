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
//! # Safety
//!
//! There is unsafe code to reuse code between sequential and parallel versions.
//!
use crate::inner_prelude::*;


pub trait NodeMassTrait:Clone{
    //type T:HasAabbMut<Num=Self::Num,Inner=Self::Inner>+Send;
    type No:Copy+Send;
    type Num:NumTrait;
    type Inner;

    //Returns the bounding rectangle for this node.
    fn get_rect(no:&Self::No)->&Rect<Self::Num>;

    //gravitate this node mass with another node mass
    fn handle_node_with_node(&self,a:&mut Self::No,b:&mut Self::No);

    //gravitate a bot with a bot
    fn handle_bot_with_bot(&self,a:BBoxRefMut<Self::Num,Self::Inner>,b:BBoxRefMut<Self::Num,Self::Inner>);

    //gravitate a nodemass with a bot
    fn handle_node_with_bot(&self,a:&mut Self::No,b:BBoxRefMut<Self::Num,Self::Inner>);

    fn is_far_enough(&self,b:[Self::Num;2])->bool;

    fn is_far_enough_half(&self,b:[Self::Num;2])->bool;

    //This unloads the force accumulated by this node to the bots.
    fn apply_to_bots<'a,I:Iterator<Item=BBoxRefMut<'a,Self::Num,Self::Inner>>> (&'a self,a:&'a Self::No,it:I);

    fn new<'a,I:Iterator<Item=BBoxRef<'a,Self::Num,Self::Inner>>> (&'a self,it:I,rect:Rect<Self::Num>)->Self::No;
}


///Naive version simply visits every pair.
pub fn naive_mut<T:HasAabbMut>(bots:ElemSliceMut<T>,func:impl FnMut(BBoxRefMut<T::Num,T::Inner>,BBoxRefMut<T::Num,T::Inner>)){
    tools::for_every_pair(bots,func);
}


use compt::dfs_order;
type CombinedVistr<'a,N,T> = compt::LevelIter<compt::Zip<dfs_order::Vistr<'a,N,dfs_order::PreOrder>,VistrMut<'a,T>>>;
type CombinedVistrMut<'a,N,T> = compt::LevelIter<compt::Zip<dfs_order::VistrMut<'a,N,dfs_order::PreOrder>,VistrMut<'a,T>>>;


fn wrap_mut<'a:'b,'b,N,T:HasAabbMut>(bla:&'b mut CombinedVistrMut<'a,N,T>)->CombinedVistrMut<'b,N,T>{
    let depth=bla.depth();

    let (a,b)=bla.as_inner_mut().as_inner_mut();

    let a=a.create_wrap_mut();
    let b=b.create_wrap_mut();

    a.zip(b).with_depth(Depth(depth))
}

//pseudo code
//build up a tree where every nodemass has the mass of all the bots in that node and all the bots under it.
fn buildtree<'a,
    T:HasAabbMut+Send+'a,
    N:NodeMassTrait<Num=T::Num,Inner=T::Inner>
    >
    (axis:impl AxisTrait,node:VistrMut<T>,misc_nodes:&mut Vec<N::No>,ncontext:&N,rect:Rect<T::Num>){


    fn recc<'a,T:HasAabbMut+Send+'a,N:NodeMassTrait<Num=T::Num,Inner=T::Inner>>
        (axis:impl AxisTrait,stuff:VistrMut<T>,misc_nodes:&mut Vec<N::No>,ncontext:&N,rect:Rect<T::Num>){
        
        let (nn,rest)=stuff.next();
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
                            let i1=left.create_wrap().dfs_preorder_iter().flat_map(|a|a.bots.iter());
                            let i2=right.create_wrap().dfs_preorder_iter().flat_map(|a|a.bots.iter());
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
    N:NodeMassTrait<Num=T::Num,Inner=T::Inner>,
    T:HasAabbMut
    >
    (_axis:impl AxisTrait,node:CombinedVistr<N::No,T>,ncontext:&N){

    fn recc<N:NodeMassTrait<Num=T::Num,Inner=T::Inner>,T:HasAabbMut>
        (stuff:CombinedVistr<N::No,T>,ncontext:&N){
        
        let ((_,(misc,nn)),rest)=stuff.next();
        match rest{
            Some([mut left,mut right])=>{

                let i1=left.as_inner_mut().as_inner_mut().1.create_wrap_mut().dfs_preorder_iter().flat_map(|a|a.bots.iter_mut());
                let i2=right.as_inner_mut().as_inner_mut().1.create_wrap_mut().dfs_preorder_iter().flat_map(|a|a.bots.iter_mut());
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
struct Anchor<'a,A:AxisTrait,T:HasAabbMut>{
	axis:A,
    range:ElemSliceMut<'a,T>,
    div:T::Num
}

fn handle_anchor_with_children<'a,
	A:AxisTrait,
	B:AxisTrait,
    N:NodeMassTrait<Num=T::Num,Inner=T::Inner>,
    T:HasAabbMut>
(thisa:A,anchor:&mut Anchor<B,T>,left:CombinedVistrMut<N::No,T>,right:CombinedVistrMut<N::No,T>,ncontext:&N){
    

    struct BoLeft<'a,B:AxisTrait,N:NodeMassTrait,T:HasAabbMut>{
        _anchor_axis:B,
        _p:PhantomData<(N::No,T)>,
        ncontext:&'a N,
    }
    
    impl<'a,B:AxisTrait,N:NodeMassTrait<Num=T::Num,Inner=T::Inner>,T:HasAabbMut> Bok2 for BoLeft<'a,B,N,T>{
        type No=N::No;
        type T=T;
        type AnchorAxis=B;

        fn handle_node<A:AxisTrait>(&mut self,_axis:A,mut b:BBoxRefMut<T::Num,T::Inner>,anchor:&mut Anchor<B,Self::T>){
            for i in anchor.range.as_mut().iter_mut(){
                self.ncontext.handle_bot_with_bot(i,b.as_mut());
            }
        }
        fn handle_node_far_enough<A:AxisTrait>(&mut self,_axis:A,a:&mut N::No,anchor:&mut Anchor<B,Self::T>){
            for i in anchor.range.as_mut().iter_mut(){
                self.ncontext.handle_node_with_bot(a,i);
            }
        }

        fn is_far_enough<A:AxisTrait>(&mut self,axis:A,anchor:&mut Anchor<B,Self::T>,misc:&Self::No)->bool{
            let rect=N::get_rect(misc);
            let range=rect.get_range(axis);
            self.ncontext.is_far_enough([anchor.div,range.right])
        }
    }

    struct BoRight<'a,B:AxisTrait,N:NodeMassTrait,T:HasAabbMut>{
        _anchor_axis:B,
        _p:PhantomData<(N::No,T)>,
        ncontext:&'a N
    }
    
    impl<'a,B:AxisTrait,N:NodeMassTrait<Num=T::Num,Inner=T::Inner>,T:HasAabbMut> Bok2 for BoRight<'a,B,N,T>{
        type No=N::No;
        type T=T;
        type AnchorAxis=B;

        fn handle_node<A:AxisTrait>(&mut self,_axis:A,mut b:BBoxRefMut<T::Num,T::Inner>,anchor:&mut Anchor<B,Self::T>){
            for i in anchor.range.as_mut().iter_mut(){
                self.ncontext.handle_bot_with_bot(i,b.as_mut());
            }
        }
        fn handle_node_far_enough<A:AxisTrait>(&mut self,_axis:A,a:&mut N::No,anchor:&mut Anchor<B,Self::T>){
            for i in anchor.range.as_mut().iter_mut(){
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
        let mut bo= BoLeft{_anchor_axis:anchor.axis,_p:PhantomData,ncontext};
        bo.generic_rec2(thisa,anchor,left);  
    }
    {
        let mut bo= BoRight{_anchor_axis:anchor.axis,_p:PhantomData,ncontext};
        bo.generic_rec2(thisa,anchor,right);  
    }
}

fn handle_left_with_right<'a,A:AxisTrait,B:AxisTrait,N:NodeMassTrait<Num=T::Num,Inner=T::Inner>,T:HasAabbMut>
    (axis:A,anchor:&mut Anchor<B,T>,left:CombinedVistrMut<'a,N::No,T>,mut right:CombinedVistrMut<'a,N::No,T>,ncontext:&N){


	struct Bo4<'a,B:AxisTrait,N:NodeMassTrait,T:HasAabbMut>{
        _anchor_axis:B,
        bot:BBoxRefMut<'a,N::Num,N::Inner>,
        ncontext:&'a N,
        div:N::Num,
        _p:PhantomData<T>
    }

    impl<'a,B:AxisTrait,N:NodeMassTrait<Num=T::Num,Inner=T::Inner>,T:HasAabbMut> Bok2 for Bo4<'a,B,N,T>{
    	type No=N::No;
        type T=T;
        type AnchorAxis=B;
    	fn handle_node<A:AxisTrait>(&mut self,_axis:A,b:BBoxRefMut<T::Num,T::Inner>,_anchor:&mut Anchor<B,Self::T>){
            self.ncontext.handle_bot_with_bot(self.bot.as_mut(),b);
    	}
    	fn handle_node_far_enough<A:AxisTrait>(&mut self,_axis:A,a:&mut N::No,_anchor:&mut Anchor<B,Self::T>){
    		self.ncontext.handle_node_with_bot(a,self.bot.as_mut());
    	}
        fn is_far_enough<A:AxisTrait>(&mut self,axis:A,_anchor:&mut Anchor<B,Self::T>,misc:&Self::No)->bool{
            let rect=N::get_rect(misc);
            let range=rect.get_range(axis);
            self.ncontext.is_far_enough_half([self.div,range.left])
        }
    }
    struct Bo2<'a,B:AxisTrait,N:NodeMassTrait,T:HasAabbMut>{
        _anchor_axis:B,
        node:&'a mut N::No,
        ncontext:&'a N,
        div:N::Num,
        _p:PhantomData<T>
    }

    impl<'a,B:AxisTrait,N:NodeMassTrait<Num=T::Num,Inner=T::Inner>,T:HasAabbMut> Bok2 for Bo2<'a,B,N,T>{
    	type No=N::No;
        type T=T;
        type AnchorAxis=B;
        fn handle_node<A:AxisTrait>(&mut self,_axis:A,b:BBoxRefMut<T::Num,T::Inner>,_anchor:&mut Anchor<B,Self::T>){
            self.ncontext.handle_node_with_bot(self.node,b);
    	}
    	fn handle_node_far_enough<A:AxisTrait>(&mut self,_axis:A,a:&mut N::No,_anchor:&mut Anchor<B,Self::T>){
    		self.ncontext.handle_node_with_node(self.node,a);
    	}
        fn is_far_enough<A:AxisTrait>(&mut self,axis:A,_anchor:&mut Anchor<B,Self::T>,misc:&Self::No)->bool{
            let rect=N::get_rect(misc);
            let range=rect.get_range(axis);
            self.ncontext.is_far_enough_half([self.div,range.left])
        }
    }

    struct Bo<'a:'b,'b,B:AxisTrait,N:NodeMassTrait,T:HasAabbMut>{
        _anchor_axis:B,
        right:&'b mut CombinedVistrMut<'a,N::No,T>,
        ncontext:&'b N
    }
    
    impl<'a:'b,'b,B:AxisTrait,N:NodeMassTrait<Num=T::Num,Inner=T::Inner>,T:HasAabbMut> Bok2 for Bo<'a,'b,B,N,T>{
    	type No=N::No;
        type T=T;
        type AnchorAxis=B;
        fn handle_node<A:AxisTrait>(&mut self,axis:A,b:BBoxRefMut<T::Num,T::Inner>,anchor:&mut Anchor<B,Self::T>){
            let r=wrap_mut(&mut self.right);
            let anchor_axis=anchor.axis;

            let mut bok=Bo4{_anchor_axis:anchor_axis,bot:b,ncontext:self.ncontext,div:anchor.div,_p:PhantomData};
            bok.generic_rec2(axis,anchor,r);
    	}
    	fn handle_node_far_enough<A:AxisTrait>(&mut self,axis:A,a:&mut N::No,anchor:&mut Anchor<B,Self::T>){
            let r=wrap_mut(&mut self.right);
            let anchor_axis=anchor.axis;

            let mut bok=Bo2{_anchor_axis:anchor_axis,node:a,ncontext:self.ncontext,div:anchor.div,_p:PhantomData};
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


fn recc<J:par::Joiner,A:AxisTrait,N:NodeMassTrait<Num=T::Num,Inner=T::Inner>+Sync+Send,T:HasAabbMut>(join:J,axis:A,it:CombinedVistrMut<N::No,T>,ncontext:&N) where T:Send,N::No:Send{
    

    let ((depth,(_,mut nn)),rest)=it.next();
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
    type T:HasAabbMut;
    type AnchorAxis:AxisTrait;
    fn is_far_enough<A:AxisTrait>(&mut self,axis:A,anchor:&mut Anchor<Self::AnchorAxis,Self::T>,misc:&Self::No)->bool;
    fn handle_node<A:AxisTrait>(&mut self,axis:A,n:BBoxRefMut<<Self::T as HasAabb>::Num,<Self::T as HasAabb>::Inner>,anchor:&mut Anchor<Self::AnchorAxis,Self::T>);
    fn handle_node_far_enough<A:AxisTrait>(&mut self,axis:A,a:&mut Self::No,anchor:&mut Anchor<Self::AnchorAxis,Self::T>);


    fn generic_rec2<
        A:AxisTrait,
        >(&mut self,this_axis:A,anchor:&mut Anchor<Self::AnchorAxis,Self::T>,stuff:CombinedVistrMut<Self::No,Self::T>){

        let ((_depth,(misc,nn)),rest)=stuff.next();
        
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
pub fn nbody_par<K:DinoTreeRefMutTrait,N:NodeMassTrait<Num=K::Num,Inner=K::Inner>+Sync+Send>(mut t1:K,ncontext:&N,rect:Rect<K::Num>) where N::No:Send, K::Item:Send+Copy{
    let axis=t1.axis();
    
    let mut misc_nodes=Vec::new();
    buildtree(axis,t1.vistr_mut(),&mut misc_nodes,ncontext,rect);

    let mut misc_tree=compt::dfs_order::CompleteTreeContainer::from_preorder(misc_nodes).unwrap();

    {
        let k=default_level_switch_sequential();
        let par=compute_default_level_switch_sequential(k,t1.height());

        let d=misc_tree.vistr_mut().zip(t1.vistr_mut()).with_depth(Depth(0));
        recc(par,axis,d,ncontext);    
    }

    apply_tree(axis,misc_tree.vistr().zip(t1.vistr_mut()).with_depth(Depth(0)),ncontext);
}


///Sequential version.
pub fn nbody<K:DinoTreeRefMutTrait<Inner=N::Inner,Num=N::Num>,N:NodeMassTrait+Send+Sync>(mut t1:K,ncontext:&N,rect:Rect<K::Num>) where K::Item:Send+Sync{
    
    let axis=t1.axis();
    
    let mut misc_nodes=Vec::new();
    
    buildtree(axis,t1.vistr_mut(),&mut misc_nodes,ncontext,rect);

    let mut misc_tree=compt::dfs_order::CompleteTreeContainer::from_preorder(misc_nodes).unwrap();

    let d=misc_tree.vistr_mut().zip(t1.vistr_mut()).with_depth(Depth(0));        
    recc(par::Sequential,axis,d,ncontext);    

    let d=misc_tree.vistr().zip(t1.vistr_mut()).with_depth(Depth(0));
    apply_tree(axis,d,ncontext);
    
}

