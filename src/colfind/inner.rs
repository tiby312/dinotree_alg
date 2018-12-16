use crate::inner_prelude::*;




use super::node_handle::*;
use super::*;

struct GoDownRecurser<'a,T:HasAabb,N,NN:NodeHandler<T=T>,B:AxisTrait>{
    _p:PhantomData<std::sync::Mutex<(N,NN)>>,
    anchor:DestructuredNode<'a,T,B>,
    sweeper:&'a mut NN
}
impl<'a,T:HasAabb,N,NN:NodeHandler<T=T>,B:AxisTrait> GoDownRecurser<'a,T,N,NN,B>{

    fn new(anchor:DestructuredNode<'a,T,B>,sweeper:&'a mut NN)->GoDownRecurser<'a,T,N,NN,B>{
        GoDownRecurser{_p:PhantomData,anchor,sweeper}
    }

    fn go_down<
        A: AxisTrait, //this axis
    >(
        &mut self,
        this_axis: A,
        m: VistrMut<N,T>,
    ) {
        let anchor_axis=self.anchor.axis;
        let (nn,rest)=m.next();

        match rest{
            Some((extra,left,right))=>{
                let &FullComp{div,cont}=match extra{
                    Some(d)=>d,
                    None=>return
                };
                
                self.sweeper.handle_children((anchor_axis,&mut self.anchor.range,&self.anchor.cont),(this_axis,nn.range,Some(&cont)));
                
                //This can be evaluated at compile time!
                if this_axis.is_equal_to(anchor_axis) {
                    if !(div < self.anchor.cont.left) {
                        self.go_down(this_axis.next(), left);
                    };
                    if !(div > self.anchor.cont.right) {
                        self.go_down(this_axis.next(), right);
                    };
                } else {
                    self.go_down(this_axis.next(), left);
                    self.go_down(this_axis.next(),right);
                }
            },
            None=>{
                self.sweeper.handle_children((anchor_axis,&mut self.anchor.range,&self.anchor.cont),(this_axis,nn.range,None));
            }
        }
    }


}







struct DestructuredNode<'a,T:HasAabb+'a,AnchorAxis:AxisTrait+'a>{
    cont:Range<T::Num>,
    _div:T::Num,
    range:&'a mut [T],
    axis:AnchorAxis
}


pub struct ColFindRecurser<T:HasAabb+Send,K:Splitter+Send,S:NodeHandler<T=T>+Splitter+Send,N:Send>{
    _p:PhantomData<std::sync::Mutex<(T,K,S,N)>>
}
impl<T:HasAabb+Send,K:Splitter+Send,S:NodeHandler<T=T>+Splitter+Send,N:Send> ColFindRecurser<T,K,S,N>{
    pub fn new()->ColFindRecurser<T,K,S,N>{
        ColFindRecurser{_p:PhantomData}
    }
    pub fn recurse<A:AxisTrait,JJ:par::Joiner>(&self,this_axis:A,par:JJ,sweeper:&mut S,m:LevelIter<VistrMut<N,T>>,splitter:&mut K){

        sweeper.node_start();
        splitter.node_start();

        let((depth,nn),rest)=m.next();

        sweeper.handle_node(this_axis.next(),nn.range);
                    
        match rest{
            Some((extra,mut left,mut right))=>{
                let &FullComp{div,cont}=match extra{
                    Some(d)=>d,
                    None=>{
                        sweeper.node_end();
                        splitter.node_end();
                        return;
                    }
                };
                

                let nn=DestructuredNode{range:nn.range,cont,_div:div,axis:this_axis};
                {
                    let left=left.inner.create_wrap_mut();
                    let right=right.inner.create_wrap_mut();
                    let mut g=GoDownRecurser::new(nn,sweeper);
                    g.go_down(this_axis.next(), left);
                    g.go_down(this_axis.next(), right);
                }

                let mut splitter2=splitter.div();
                    
                let splitter={
                    let splitter2=&mut splitter2;
                    if !par.should_switch_to_sequential(depth) {
                        let mut sweeper2=sweeper.div();
                        
                        let (sweeper,splitter)={
                            let sweeper2=&mut sweeper2;
                            let af = move || {
                                self.recurse(this_axis.next(),par,sweeper,left,splitter);(sweeper,splitter)
                            };
                            let bf = move || {
                                self.recurse(this_axis.next(),par,sweeper2,right,splitter2)
                            };
                            rayon::join(af, bf).0
                        };
                        sweeper.add(sweeper2);
                        splitter
                    } else {
                        self.recurse(this_axis.next(),par.into_seq(),sweeper,left,splitter);
                        self.recurse(this_axis.next(),par.into_seq(),sweeper,right,splitter2);
                        splitter
                    }
                };

                splitter.add(splitter2);
            },
            None=>{
                sweeper.node_end();
                splitter.node_end();
            }
        }
    }
}



pub struct QueryFnMut<T,F>(F,PhantomData<std::sync::Mutex<T>>);
impl<T:HasAabb,F:FnMut(&mut T,&mut T)> QueryFnMut<T,F>{
    pub fn new(func:F)->QueryFnMut<T,F>{
        QueryFnMut(func,PhantomData)
    }
}
impl<T:HasAabb,F:FnMut(&mut T,&mut T)> ColMulti for QueryFnMut<T,F>{
    type T=T;
    fn collide(&mut self,a:&mut T,b:&mut T){
        self.0(a,b);
    }   
}
impl<T,F> Splitter for QueryFnMut<T,F>{
    fn div(&mut self)->Self{
        unreachable!()
    }
    fn add(&mut self,_:Self){
        unreachable!()
    }
    fn node_start(&mut self){}
    fn node_end(&mut self){}
}


pub struct QueryFn<T,F>(F,PhantomData<std::sync::Mutex<T>>);
impl<T:HasAabb,F:Fn(&mut T,&mut T)> QueryFn<T,F>{
    pub fn new(func:F)->QueryFn<T,F>{
        QueryFn(func,PhantomData)
    }
}
impl<T:HasAabb,F:Fn(&mut T,&mut T)> ColMulti for QueryFn<T,F>{
    type T=T;
    fn collide(&mut self,a:&mut T,b:&mut T){
        self.0(a,b);
    }   
}
impl<T,F:Copy> Splitter for QueryFn<T,F>{
    fn div(&mut self)->Self{
        QueryFn(self.0,PhantomData)
    }
    fn add(&mut self,_:Self){
        
    }
    fn node_start(&mut self){}
    fn node_end(&mut self){}
}

//TODO why?
//unsafe impl<T,F> Sync for Bo2<T,F>{}