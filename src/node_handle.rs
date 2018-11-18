
use inner_prelude::*;
use colfind::ColMulti;

pub trait NodeHandler{
    type T:HasAabb;
    
    fn handle_node(
        &mut self,
        axis:impl AxisTrait,
        bots:&mut [Self::T]
    );

    fn handle_children(
        &mut self,
        anchor:(impl AxisTrait,&mut [Self::T],&Range<<Self::T as HasAabb>::Num>),
        current:(impl AxisTrait,&mut [Self::T],Option<&Range<<Self::T as HasAabb>::Num>>)
    );
}

pub struct HandleNoSorted<K:ColMulti+Splitter>{
    pub func:K
}
impl<K:ColMulti+Splitter>  HandleNoSorted<K>{
    pub fn new(func:K)->Self{
        HandleNoSorted{func}
    }
}

impl<K:ColMulti+Splitter> Splitter for HandleNoSorted<K>{
    fn div(&mut self)->Self{
        HandleNoSorted{func:self.func.div()}
    }
    fn add(&mut self,a:Self){
        self.func.add(a.func);   
    }
    fn node_start(&mut self){
        self.func.node_start();
    }
    fn node_end(&mut self){
        self.func.node_start();
    }
}

impl<K:ColMulti+Splitter> NodeHandler for HandleNoSorted<K>{
    type T=K::T;
    fn handle_node(&mut self,axis:impl AxisTrait,bots:&mut [Self::T]){
        let func=&mut self.func;
        
        tools::for_every_pair(bots,|a,b|{
            if a.get().get_intersect_rect(b.get()).is_some(){
                func.collide(a,b);
            }
        });
    }
    fn handle_children(&mut self,anchor:(impl AxisTrait,&mut [Self::T],&Range<<K::T as HasAabb>::Num>),current:(impl AxisTrait,&mut [Self::T],Option<&Range<<K::T as HasAabb>::Num>>)){
        let (this_axis,this_range,cont)=current;
        let (anchor_axis,anchor_range,anchor_box)=anchor;
        let func=&mut self.func;
        
        let res=match cont{
            Some(cont)=>{
                if !this_axis.is_equal_to(anchor_axis) {
                    true
                } else {
                    if cont.intersects(anchor_box){
                        true
                    }else{
                        false
                    }
                }
            },
            None=>{
                true
            }
        };

        if res{
            for a in this_range.iter_mut(){
                for b in anchor_range.iter_mut(){
                    if a.get().get_intersect_rect(b.get()).is_some(){
                        func.collide(a,b);
                    }
                }
            }
        }
        
    }

}




pub struct HandleSorted<K:ColMulti+Splitter>{
    pub sweeper:oned::Sweeper<K::T>,
    pub func:K
}
impl<K:ColMulti+Splitter> HandleSorted<K>{
    pub fn new(a:K)->HandleSorted<K>{
        HandleSorted{sweeper:oned::Sweeper::new(),func:a}
    }
}
impl<K:ColMulti+Splitter> Splitter for HandleSorted<K>{
    fn div(&mut self)->Self{
        HandleSorted{sweeper:oned::Sweeper::new(),func:self.func.div()}
    }
    fn add(&mut self,a:Self){
        self.func.add(a.func);   
    }
    fn node_start(&mut self){
        self.func.node_start();
    }
    fn node_end(&mut self){
        self.func.node_start();
    }
}

impl<K:ColMulti+Splitter> NodeHandler for HandleSorted<K>{
    type T=K::T;
    fn handle_node(&mut self,axis:impl AxisTrait,bots:&mut [Self::T]){
        let func=&mut self.func;
        self.sweeper.find_2d(axis,bots,func);
    }
    fn handle_children(&mut self,anchor:(impl AxisTrait,&mut [Self::T],&Range<<K::T as HasAabb>::Num>),current:(impl AxisTrait,&mut [Self::T],Option<&Range<<K::T as HasAabb>::Num>>)){
        let (this_axis,this_range,cont)=current;
        let (anchor_axis,anchor_range,anchor_box)=anchor;
        let func=&mut self.func;
        match cont{
            Some(cont)=>{
                if !this_axis.is_equal_to(anchor_axis) {
                        let r1 = oned::get_section_mut(anchor_axis,this_range, anchor_box);
                        let r2= oned::get_section_mut(this_axis,anchor_range,cont);     
                        self.sweeper.find_perp_2d(r1,r2,func);
                } else {
                    if cont.intersects(anchor_box){
                        self.sweeper.find_parallel_2d(
                            this_axis.next(),
                            this_range,
                            anchor_range,
                            func,
                        );
                    }
                }
            },
            None=>{
                if !this_axis.is_equal_to(anchor_axis) {

                    let r1 =oned::get_section_mut(anchor_axis,this_range, anchor_box);
                    let r2= anchor_range;

                    self.sweeper.find_perp_2d(r1,r2,func);

                } else {
                    self.sweeper.find_parallel_2d(
                        this_axis.next(),
                        this_range,
                        anchor_range,
                        func,
                    );
                }
            }
        }
        
    }

}


