use inner_prelude::*;
use super::*;

pub fn k_nearest<
    A:AxisTrait,
    T:SweepTrait,
    F: FnMut(ColSingle<T>,T::Num),
    MF:Fn((T::Num,T::Num),&AABBox<T::Num>)->T::Num,
    MF2:Fn(T::Num,T::Num)->T::Num,
    >(tree:&mut DynTree<A,T>,point:(T::Num,T::Num),num:usize,mut func:F,mf:MF,mf2:MF2){

    let dt = tree.get_iter_mut();

    let mut c=ClosestCand::new(num);
    recc(A::new(),dt,&mf,&mf2,point,&mut c);
 
    for i in c.into_sorted(){
        let j=unsafe{&mut *i.0}.get_mut();
        func(ColSingle{inner:j.1,rect:j.0},i.1);
    }


}


use self::cand::ClosestCand;
mod cand{
    use super::*;

    pub struct ClosestCand<T:SweepTrait>{
        a:SmallVec<[(*mut T,T::Num);32]>,
        num:usize
    }
    impl<T:SweepTrait> ClosestCand<T>{

        //First is the closest
        pub fn into_sorted(self)->SmallVec<[(*mut T,T::Num);32]>{
            self.a
        }
        pub fn new(num:usize)->ClosestCand<T>{
            let a=SmallVec::with_capacity(num);
            ClosestCand{a,num}
        }

        pub fn consider(&mut self,a:(&mut T,T::Num)){
            let a=(a.0 as *mut T,a.1);

            if self.a.len()<self.num{
                

                let arr=&mut self.a;
                if arr.len()==0{
                    arr.push(a);
                }else{
                    let mut inserted=false;
                    for i in 0..arr.len(){
                        if a.1<arr[i].1{
                            arr.insert(i,a);
                            inserted=true;
                            break;
                        }
                    }
                    if !inserted{
                        arr.push(a);
                    }

                }

            }else{
                let arr=&mut self.a;
                for i in 0..arr.len(){
                    if a.1<arr[i].1{
                        arr.pop();
                        arr.insert(i,a);
                        break;
                    }
                }
                
            }
        }
        pub fn full_and_max_distance(&self)->Option<T::Num>{
            match self.a.get(self.num-1){
                Some(x)=>
                {
                    Some(x.1)
                },
                None=>{
                    None
                }
            }
        }
    }
}

fn recc<'x,'a,
    A: AxisTrait,
    T: SweepTrait + 'x,
    C: CTreeIterator<Item = &'x mut NodeDyn<T>>,
    MF:Fn((T::Num,T::Num),&AABBox<T::Num>)->T::Num,
    MF2:Fn(T::Num,T::Num)->T::Num,
    >(axis:A,stuff:C,mf:&MF,mf2:&MF2,point:(T::Num,T::Num),res:&mut ClosestCand<T>){

    let (nn,rest)=stuff.next();

    //known at compile time.
    let pp=if axis.is_xaxis(){
        point.0
    }else{
        point.1
    };

    
    match rest {
        Some((left, right)) => {
            let div = nn.div.unwrap();
    

            let (first,other)=if pp<div {
                (left,right)
            }else{
                (right,left)
            };

            recc(axis.next(), first,mf,mf2,point,res);
           
            let traverse_other=match res.full_and_max_distance(){
                Some(max)=>{
                    if mf2(pp,div)<max{
                        true
                    }else{
                        false
                    }
                },
                None=>{
                    true
                }
            };

            if traverse_other{
                recc(axis.next(),other,mf,mf2,point,res);
            }
        }
        _ => {
            
        }
    }

    let traverse_other=match res.full_and_max_distance(){
        Some(max)=>{
            match nn.div{
                Some(div)=>{
                    if mf2(pp,div)<max{
                        true
                    }else{
                        false
                    }
                },
                None=>{
                    true
                }
            }
        },
        None=>{
            true
        }
    };

    if traverse_other{
        for i in nn.range.iter_mut(){            
            let dis_sqr=mf(point,i.get().0);
            res.consider((i,dis_sqr));
        }
    }

}


#[cfg(test)]
mod test{
    use super::*;
    use test_support::*;
    use support::BBox;
    use test::*;


    #[test]
    fn test_k_nearest(){
        fn from_point(a:isize,b:isize)->AABBox<isize>{
            AABBox::new((a,a),(b,b))
        }

        let mut bots=Vec::new();
        bots.push(BBox::new(Bot::new(4),from_point(15,15)));
        bots.push(BBox::new(Bot::new(1),from_point(10,10)));
        bots.push(BBox::new(Bot::new(2),from_point(20,20)));
        bots.push(BBox::new(Bot::new(3),from_point(30,30)));
        bots.push(BBox::new(Bot::new(0),from_point(0,0)));

        let mut res=Vec::new();

        let min_rect=|point:(isize,isize),aabb:&AABBox<isize>|{
            let (px,py)=(point.0,point.1);
            //let (px,py)=(px.0,py.0);

            let ((a,b),(c,d))=aabb.get();

            let xx=num::clamp(px,a,b);
            let yy=num::clamp(py,c,d);

            (xx-px)*(xx-px) + (yy-py)*(yy-py)
        };

        let min_oned=|p1:isize,p2:isize|{
            //let (p1,p2)=(p1.0,p2.0);
            (p2-p1)*(p2-p1)
        };

        {
            let mut dyntree = DinoTree::new(&mut bots,  StartAxis::Yaxis);

            dyntree.k_nearest((40,40),3,|a,_dis|res.push(a.inner.id),&min_rect,&min_oned);
            assert!(res.len()==3);
            assert!(res[0]==3);
            assert!(res[1]==2);
            assert!(res[2]==4);

            res.clear();
            dyntree.k_nearest((-40,-40),3,|a,_dis|res.push(a.inner.id),min_rect,min_oned);
            assert!(res.len()==3);
            println!("res={:?}",res);
            assert!(res[0]==0);
            assert!(res[1]==1);
            assert!(res[2]==4);
        }


    }


    #[bench]
    fn k_nearest_par_point(b: &mut Bencher) {
        use test_support::*;
        let mut p = PointGenerator::new(
            &test_support::make_rect((0, 200), (0, 200)),
            &[100, 42, 6],
        );

        let mut bots = Vec::new();
        let mut points=Vec::new();
        for id in 0..2000 {
            let ppp = p.random_point();
            points.push(ppp);
            //let k = test_support::create_rect_from_point(ppp);
            bots.push(BBox::new(
                Bot {
                    id,
                    col: Vec::new(),
                },
                AABBox::<isize>::new((ppp.0,ppp.0),(ppp.1,ppp.1)),
            ));
        }


        //println!("bot 716={:?}",&bots[716]);
        //println!("point 19={:?} bot19={:?}",&points[19],&bots[19]);    
        let mut tree = DinoTree::new(&mut bots,  StartAxis::Yaxis);


        

        b.iter(|| {

            for (i,p) in points.iter().enumerate(){
                let min_rect=|point:(isize,isize),aabb:&AABBox<isize>|{
                    let (px,py)=(point.0,point.1);
                    //let (px,py)=(px.0,py.0);

                    let ((a,b),(c,d))=aabb.get();

                    let xx=num::clamp(px,a,b);
                    let yy=num::clamp(py,c,d);

                    (xx-px)*(xx-px) + (yy-py)*(yy-py)
                };

                let min_oned=|p1:isize,p2:isize|{
                    //let (p1,p2)=(p1.0,p2.0);
                    (p2-p1)*(p2-p1)
                };

                tree.k_nearest(*p,1,|a,_|{
                    if a.inner.id!=i{
                        let ((a,b),(c,d))=a.rect.get();
                        assert_eq!(a,p.0);
                        assert_eq!(b,p.0);
                        assert_eq!(c,p.1);
                        assert_eq!(d,p.1);
                    }
                    
                },min_rect,min_oned);
            }
            /*
            let k=tree.intersect_every_pair_debug(|a, b| {
                a.inner.col.push(b.inner.id);
                b.inner.col.push(a.inner.id);
            });
            */
            //println!("{:?}",k.into_vec());
            //black_box(k);
        });

        //assert!(false);
    }


    #[bench]
    fn k_nearest_par_point2(b: &mut Bencher) {
        use test_support::*;
        let mut p = PointGenerator::new(
            &test_support::make_rect((0, 200), (0, 200)),
            &[100, 42, 6],
        );

        let mut bots = Vec::new();
        let mut points=Vec::new();
        for id in 0..2000 {
            let ppp = p.random_point();
            points.push(ppp);
            //let k = test_support::create_rect_from_point(ppp);
            bots.push(BBox::new(
                Bot {
                    id,
                    col: Vec::new(),
                },
                AABBox::<isize>::new((ppp.0,ppp.0),(ppp.1,ppp.1)),
            ));
        }


        //println!("bot 716={:?}",&bots[716]);
        //println!("point 19={:?} bot19={:?}",&points[19],&bots[19]);    
        let mut tree = DinoTree::new(&mut bots,  StartAxis::Xaxis);


        

        b.iter(|| {

            let mut total_dis=0;
            let mut num_found=0;
            for (i,p) in points.iter().enumerate(){
                let min_rect=|point:(isize,isize),aabb:&AABBox<isize>|{
                    let (px,py)=(point.0,point.1);
                    //let (px,py)=(px.0,py.0);

                    let ((a,b),(c,d))=aabb.get();

                    let xx=num::clamp(px,a,b);
                    let yy=num::clamp(py,c,d);

                    (xx-px)*(xx-px) + (yy-py)*(yy-py)
                };

                let min_oned=|p1:isize,p2:isize|{
                    //let (p1,p2)=(p1.0,p2.0);
                    (p2-p1)*(p2-p1)
                };


                let mut counter=0;
                tree.k_nearest(*p,2,|a,dis|{

                    if counter==1{
                        total_dis+=dis;
                        num_found+=1;
                    }
                    counter+=1;
                    
                },min_rect,min_oned);
            }

            let avg=total_dis/(points.len() as isize);
            //println!("avg dis={:?}",));
            //Check that the average distance the the nearest object to every other object
            //is small
            assert!(avg<10);
            assert_eq!(num_found,points.len());
            /*
            let k=tree.intersect_every_pair_debug(|a, b| {
                a.inner.col.push(b.inner.id);
                b.inner.col.push(a.inner.id);
            });
            */
            //println!("{:?}",k.into_vec());
            black_box(avg);
        });

        //assert!(false);
    }



}