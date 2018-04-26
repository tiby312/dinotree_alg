use inner_prelude::*;
use super::*;


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


pub trait NodeMassTrait{
    type T:SweepTrait;
    fn handle_with(&self,b:&mut Self);
    fn handle_bot(&mut Self::T,&mut Self::T);
    fn new(rect:&Rect<<Self::T as SweepTrait>::N>,b:&[Self::T])->Self;
    fn apply(&mut self,b:&mut [Self::T]);
    fn is_far_enough(&self,b:&Self)->bool;
}



pub fn nbody_seq<
    A:AxisTrait,
    T:SweepTrait,
    No:NodeMassTrait<T=T>,
>(tree:&mut DynTree<A,T>,rect:AABBox<T::Num>,dis:T::Num){
    
    let mut tree2={
        let dt=tree.get_iter();
        
        //TODO this is ugly
        let aa=if A::new().is_xaxis(){
            compt::TAxis::XAXIS
        }else{
            compt::TAxis::YAXIS
        };

        let tt:compt::Extra<_,Rect<T::Num>,_>=dt.with_axis(aa).with_extra(|aa:&(compt::TAxis,&NodeDyn<T>),rect:Rect<T::Num>|{
            let &(axis,node)=aa;

            //TODO this is ugly
            let axis=match axis{
                compt::TAxis::XAXIS=>{
                    axgeom::XAXIS
                },
                compt::TAxis::YAXIS=>{
                    axgeom::YAXIS
                }
            };

            match node.div{
                Some(div)=>{
                    rect.subdivide(div,axis)
                },
                None=>{
                    (rect,rect)
                }
            }
        },rect.0);
        let mut tt=tt.dfs_preorder_iter();
        
        compt::dfs::GenTreeDfsOrder::from_dfs_inorder(||{
            let (rect,nodedyn)=tt.next().unwrap();
            No::new(&rect,&nodedyn.1.range)
        },tree.get_height())
    };

    {
        let dt=tree.get_iter_mut();
        let dt2=tree2.create_down_mut();
        let dt3=dt.zip(dt2);

        //higher level nodes, will have bots along a longer line.
        //so approximating them all, has worse results for higher level nodes.
        //solution? the threshold at which point to use approximate decreases as you go down the tree levels.

        recc(A::new(),dt3,no);
    }

    //Now lets apply the nodemasses to the bots.
    {
        let dt=tree.get_iter_mut();
        let dt2=tree2.create_down_mut();
        let dt3=dt.zip(dt2);  
    
        dt3.dfs_preorder(|(node,mass)|{
            no.apply_nodemass(mass,&mut node.range);
        });
    }    

}

fn recc<'a,
    A:AxisTrait,
    T:SweepTrait+'a,
    No:NodeMassTrait<T=T>,
    C:CTreeIterator<Item=(&'a mut NodeDyn<T>,&'a mut No)>,
>(axis:A,stuff:C){

    let (nn,rest)=stuff.next();


    tools::for_every_pair(&mut nn.0.range,|a,b|{
        No::handle_bot(a,b);
    });



    match rest{
        Some((mut left,mut right))=>{
            {
                let left = compt::WrapGen::new(&mut left);

                for mut x in left.dfs_preorder_iter(){
                    let &mut (ref mut node1,ref mut mass1)=x.get_mut();
                    let right = compt::WrapGen::new(&mut right);
                    for mut y in right.dfs_preorder_iter(){
                        let &mut (ref mut node2,ref mut mass2)=y.get_mut();

                        if no.is_far_enough(mass2){
                            no.handle_with(mass2);
                        }else{
                            tools::for_bijective_pair(&mut node1.range,&mut node2.range,|a,b|{
                                No::handle_bot(a,b);
                            });
                        }
                    }
                }
            }

            //This can happen in parallel
            recc(axis.next(),left);
            recc(axis.next(),right);
        },
        None=>{

        }
    }
}