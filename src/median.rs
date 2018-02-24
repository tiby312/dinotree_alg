
use oned;
use oned::Binned;
use std;
use SweepTrait;
use NumTrait;
use std::marker::PhantomData;
use axgeom::AxisTrait;
use compt::LevelDesc;
use tools;
use InnerRect;

///Defines what divider-placement strategy to use.
pub trait MedianStrat:Sync{
    type Num:NumTrait;
    ///updates median and bins.
    fn compute<'a,A:AxisTrait,T:SweepTrait<Num=Self::Num>>(
        &self,
        level:LevelDesc,
        rest:&'a mut [T],
        mmm:&mut T::Num)->(T::Num,Binned<'a,T>);
}



///This median finding strategy revolves incrementally improving the dividers placement
///based off of how many bots were split between left and right in the last iteration.
pub mod relax{
    use super::*;

    ///Using this the user specifies how much they want the divider
    ///incrementally move.
    ///This is use-defined because depending how how big the space is that is being partitioned
    ///and depending how fast the object in it may move,
    ///you want want the dividers to move faster or slow.
    ///Various states are provided to the function that the user may choose
    ///to use as inputs to their calculation.
    ///Make the dividers move too much and they were be "jumpy"
    pub trait DivMoveStrat:Sync{
        type N:NumTrait;
        ///divider: this is what the user has to move towards the direction mag specifies.
        ///If mag is positive, add to the divider position, and vice versa.
        ///total: the total number of bots being segmented. (Natually for lower levels of the tree this number is smaller)
        ///mag: between -1 and 1. 
        fn move_divider(&self,divider:&mut Self::N,total:usize,mag:f32);
    }


    pub struct MedianRelax<N:NumTrait,D:DivMoveStrat<N=N>>{
        a:D,
        _p:PhantomData<N>
    }

    impl<N:NumTrait,D:DivMoveStrat<N=N>> MedianRelax<N,D>{
        pub fn new(a:D)->MedianRelax<N,D>{
            MedianRelax{a,_p:PhantomData}
        }
    }


    impl<N:NumTrait,D:DivMoveStrat<N=N>> MedianStrat for MedianRelax<N,D>{
        type Num=N;
        fn compute<'a,A:AxisTrait,T:SweepTrait<Num=N>>(
            &self,
            level:LevelDesc,
            rest:&'a mut [T],
            divider:&mut T::Num)->(T::Num,Binned<'a,T>){
            let div_axis=A::get();

            let med=*divider;

            let mut times=[0.0f64;2];
            let tt0=tools::Timer2::new();

            //TODO only do this at upper levels??
            let binned=oned::bin_par::<A,_>(&med,rest);
            times[0]=tt0.elapsed();

            //At this point we have binned into 3 bins. middile,left, and right.
            //In order to know just how many bots are to the left or right of the divider,
            //we also need to bin middile into those to the left and right of the divider.


            let tt0=tools::Timer2::new();
            {
                let (mleft,mright)=bin_middile(binned.middile,|a:&T,div:&T::Num|{
                    a.get().0.get().get_range(div_axis).left().cmp(div)
                },&med);
                
                //Now we have binned the middiles in addition to the left and right bins.
                //To find the total number of bots to the left and right, we sum.
                let fa=binned.left.len()+mleft.len();
                let fb=binned.right.len()+mright.len();
                
                {
                    let total=fa+fb;
                    let fa=fa as f32;
                    let half=(total/2) as f32;
     
                    if half>0.0{
                        let mag=-(fa-half)/half; //Mag is between -1 and 1
                        
                        //every tree construction, at maximum, move 10% of the width of this node.
                        //a higher percentage means the dividers jumpier and more unstable
                        //a lower percentage means the dividers will move slower to their optimal positions.
                        //if the dividers arnt doing a good job splitting the range into two even halfs,
                        //the performance benefits of a tree are lost.
                        
                        self.a.move_divider(divider,total,mag);
                    }
                }            
                
                
            }
            times[1]=tt0.elapsed();

            if level.get_depth()==0{
                println!("med times={:?}",times);
            }

            //Return the divider before we moved it.
            //This is the version of the divider that was actually used to
            //bin the bots into the 3 buckets.
            (med,binned)
        }  
     }
}

use std::cmp::Ordering;
fn bin_middile<'a,T,X,F:Fn(&T,&X)->Ordering>(arr:&'a mut [T],func:F,div:&X)->(&'a mut [T],&'a mut [T]){
     //        equal   divider
     //         ^       ^
     //  equal  | less  | greater|  unsorted
    let mut divider=0;
    let mut equal=0;
    for i in 0..arr.len(){
        
        match func(&arr[i],div){
            //If the divider is greater than the bot
            std::cmp::Ordering::Greater=>{
                
            },
            std::cmp::Ordering::Less=>{
                arr.swap(divider,i);
                divider+=1;
            },
            _=>{
                arr.swap(divider,i);
                arr.swap(divider,equal);                 
                divider+=1;
                equal+=1;
            }
        }
    }
    let (_eq,rest)=arr.split_at_mut(equal);
    rest.split_at_mut(divider-equal)
}


/*
pub struct MedianRelax2<N:NumTrait>{
    _p:PhantomData<N>
}


impl<N:NumTrait> MedianStrat for MedianRelax2<N>{
    type Num=N;
    fn compute<'a,A:AxisTrait,T:SweepTrait<Num=N>>(
        &self,
        _depth:usize,
        rest:&'a mut [T],
        divider:&mut T::Num)->(T::Num,Binned<'a,T>){
        let div_axis=A::get();
        //let blee=Blee::new(div_axis.get());

        let med=*divider;

        let binned=oned::bin::<A,_>(&med,rest);

        let diff=|a:&&T,b:&&T|->Ordering{
            let a=a.get().0.get_range(div_axis).left();
            let b=b.get().0.get_range(div_axis).left();
            a.cmp(&b)
        };

        //At this point we have binned into 3 bins. middile,left, and right.
        //In order to know just how many bots are to the left or right of the divider,
        //we also need to bin middile into those to the left and right of the divider.
        {
            let (mleft,mright)=bin_middile(binned.middile,|a:&T,div:&T::Num|{
                a.get().0.get_range(div_axis).left().cmp(div)
            },&med);
            
            //Now we have binned the middiles in addition to the left and right bins.
            //To find the total number of bots to the left and right, we sum.
            let fa=binned.left.len()+mleft.len();
            let fb=binned.right.len()+mright.len();
                 
            
            if fa > fb {
                let k=mleft.iter().chain(binned.left.iter()).max_by(diff).unwrap();
                *divider=k.get().0.get_range(div_axis).left();
            }else if fa < fb{
                let k=mright.iter().chain(binned.right.iter()).min_by(diff).unwrap();
                *divider=k.get().0.get_range(div_axis).left();
            }else{
                //leave the divider where it is. it is perfectly dividing left and right.
            }
            
            
        }

        //Return the divider before we moved it.
        //This is the version of the divider that was actually used to
        //bin the bots into the 3 buckets.
        (med,binned)
    }  
 }
*/
 
///This median finding strategy revolves around using quickselect to find the median without use of the previous state.
pub mod strict{
    use super::*;

    pub struct MedianStrict<N:NumTrait>{
        _p:PhantomData<N>
    }

    impl<N:NumTrait> MedianStrict<N>{
        pub fn new()->MedianStrict<N>{
            MedianStrict{_p:PhantomData}
        }
    }
    impl<N:NumTrait> MedianStrat for MedianStrict<N>{
        type Num=N;
        
        fn compute<'a,A:AxisTrait,T:SweepTrait<Num=N>>(
            &self,
            level:LevelDesc,
            rest:&'a mut [T],
            mmm:&mut T::Num)->(T::Num,Binned<'a,T>){
            let div_axis=A::get();

            let med={
            
                

                let m = if rest.len() == 0{
                            std::default::Default::default()
                            //TODO what to do here?
                    }
                    else
                    {
                         let closure = |a: &T, b: &T| -> std::cmp::Ordering {
        
                            let arr=a.get().0.get().get_range(div_axis);
                            let brr=b.get().0.get().get_range(div_axis);
                      
                            if arr.left() > brr.left(){
                                return std::cmp::Ordering::Greater;
                            
                            }
                            std::cmp::Ordering::Less
                        };


                        let mm=rest.len()/2;
                        use pdqselect;
                        pdqselect::select_by(rest, mm, closure);
                        
                        let k=&rest[mm];
                        k.get().0.get().get_range(div_axis).start
                    };
                *mmm=m;
                m
                
            };

            let binned=oned::bin_par::<A,_>(&med,rest);

            (med,binned)
        }
    } 
}
