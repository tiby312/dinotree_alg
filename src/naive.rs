use crate::query::*;
use crate::inner_prelude::*;

pub trait HasId{
    fn get_id(&self)->usize;
}

#[derive(Debug,Eq,Ord,PartialOrd,PartialEq)]
struct IDPair{
    a:usize,
    b:usize
}

impl IDPair{
    fn new(a:usize,b:usize)->IDPair{
        let (a,b)=if a<=b{
            (a,b)
        }else{
            (b,a)
        };
        IDPair{a,b}
    }
}


struct UnitMut2<N>{
    id:usize,
    mag:N
}

pub struct NaiveAlgs<'a,T>{
    bots:&'a mut [T]
}

impl<'a,T:HasAabb> NaiveAlgs<'a,T>{
    #[must_use]
    pub fn new(bots:&'a mut [T])->NaiveAlgs<T>{
        NaiveAlgs{bots}
    }


    #[must_use]
    pub fn raycast_mut(
        &mut self,
        ray:raycast::Ray<T::Num>,
        rtrait: &mut impl raycast::RayTrait<N=T::Num,T=T> )->raycast::RayCastResult<T>{
        let bots=ProtectedBBoxSlice::new(self.bots);
        raycast::raycast_naive_mut(bots,ray,rtrait)
    }

    #[must_use]
    pub fn k_nearest_mut(
        &mut self,
        point:Vec2<T::Num>,
        num:usize,
        knear:&mut impl k_nearest::Knearest<N=T::Num,T=T>) -> Vec<k_nearest::KnearestResult<T>>{
        let bots=ProtectedBBoxSlice::new(self.bots);
        k_nearest::k_nearest_naive_mut(bots,point,num,knear)
    }

    pub fn naive_for_all_not_in_rect_mut(&mut self,rect:&Rect<T::Num>,func:impl FnMut(ProtectedBBox<T>)){
        let bots=ProtectedBBoxSlice::new(self.bots);
        rect::naive_for_all_not_in_rect_mut(bots,rect,func);
    }
    pub fn for_all_not_in_rect_mut(&mut self,rect:&Rect<T::Num>,func:impl FnMut(ProtectedBBox<T>)){
        let bots=ProtectedBBoxSlice::new(self.bots);
        rect::naive_for_all_in_rect_mut(bots,rect,func);
    }

    pub fn for_all_intersect_rect_mut(&mut self,rect:&Rect<T::Num>,func:impl FnMut(ProtectedBBox<T>)){
        let bots=ProtectedBBoxSlice::new(self.bots);
        rect::naive_for_all_intersect_rect_mut(bots,rect,func);
    }

    pub fn find_collisions_mut(&mut self,mut func:impl FnMut(ProtectedBBox<T>,ProtectedBBox<T>)){
        let bots=ProtectedBBoxSlice::new(self.bots);
        colfind::query_naive_mut(bots,|a,b|{
            func(a,b)
        });
    }
    
    pub fn find_collisions_sweep_mut<A:AxisTrait>(&mut self,axis:A,mut func:impl FnMut(ProtectedBBox<T>,ProtectedBBox<T>)){
        colfind::query_sweep_mut(axis,self.bots,|a,b|{
            func(a,b)
        });
    }


}

impl<'a,T:HasInner> NaiveAlgs<'a,T>{

    pub fn assert_k_nearest_mut(&mut self,point:Vec2<T::Num>,num:usize,knear:&mut impl k_nearest::Knearest<N=T::Num,T=T>,rect:Rect<T::Num>) where T::Inner: HasId{

        let mut res_naive:Vec<_>={
            let bots=ProtectedBBoxSlice::new(self.bots);
            k_nearest::k_nearest_naive_mut(bots,point,num,knear).drain(..).map(|a|UnitMut2{id:a.bot.inner().get_id(),mag:a.mag}).collect()
        };
        let mut tree=DinoTreeBuilder::new(axgeom::XAXISS,self.bots).build_seq(); 

        let mut res_dinotree:Vec<_>=k_nearest::k_nearest_mut(&mut tree,point,num,knear,rect).drain(..).map(|a|UnitMut2{id:a.bot.inner().get_id(),mag:a.mag}).collect();

        assert_eq!(res_naive.len(),res_dinotree.len());
        
        let r_naive=util::SliceSplitMut::new(&mut res_naive,|a,b|a.mag==b.mag);
        let r_dinotree=util::SliceSplitMut::new(&mut res_dinotree,|a,b|a.mag==b.mag);

        for (a,b) in r_naive.zip(r_dinotree){
            assert_eq!(a.len(),b.len());
            a.sort_by(|a,b|a.id.cmp(&b.id));
            b.sort_by(|a,b|a.id.cmp(&b.id));

            let res = a.iter().zip(b.iter()).fold(true,|acc,(a,b)|{
                acc & ((a.id==b.id) && (a.mag==b.mag))
            });

            assert!(res);
        }
    }

    pub fn assert_for_all_not_in_rect_mut(&mut self,rect:&axgeom::Rect<T::Num>) where T::Inner:HasId{
        let mut naive_res=Vec::new();
        {
            let bots=ProtectedBBoxSlice::new(self.bots);
            rect::naive_for_all_not_in_rect_mut(bots,rect,|a|naive_res.push(a.inner().get_id()));
        }

        let mut dinotree_res=Vec::new();
        let mut tree=DinoTreeBuilder::new(axgeom::XAXISS,self.bots).build_seq(); 
        rect::for_all_not_in_rect_mut(&mut tree,rect,|a|dinotree_res.push(a.inner().get_id()));

        assert_eq!(naive_res.len(),dinotree_res.len());
        naive_res.sort();
        dinotree_res.sort();

        let res = naive_res.iter().zip(dinotree_res.iter()).fold(true,|acc,(a,b)|{
            acc & (*a==*b)
        });

        assert!(res);
    }

    pub fn assert_for_all_in_rect_mut(&mut self,rect:&axgeom::Rect<T::Num>) where T::Inner:HasId{
        let mut naive_res=Vec::new();
        {
            let bots=ProtectedBBoxSlice::new(self.bots);
            rect::naive_for_all_in_rect_mut(bots,rect,|a|naive_res.push(a.inner().get_id()));
        }
        let mut dinotree_res=Vec::new();
        let mut tree=DinoTreeBuilder::new(axgeom::XAXISS,self.bots).build_seq(); 
        rect::for_all_in_rect_mut(&mut tree,rect,|a|dinotree_res.push(a.inner().get_id()));

        assert_eq!(naive_res.len(),dinotree_res.len());
        naive_res.sort();
        dinotree_res.sort();

        let res = naive_res.iter().zip(dinotree_res.iter()).fold(true,|acc,(a,b)|{
            acc & (*a==*b)
        });

        assert!(res);
    }


    pub fn assert_for_all_intersect_rect_mut(&mut self,rect:&axgeom::Rect<T::Num>) where T::Inner:HasId{
        let mut naive_res=Vec::new();
        {
            let bots=ProtectedBBoxSlice::new(self.bots);
            rect::naive_for_all_intersect_rect_mut(bots,rect,|a|naive_res.push(a.inner().get_id()));
        }
        let mut dinotree_res=Vec::new();
        let mut tree=DinoTreeBuilder::new(axgeom::XAXISS,self.bots).build_seq(); 
        rect::for_all_intersect_rect_mut(&mut tree,rect,|a|dinotree_res.push(a.inner().get_id()));

        assert_eq!(naive_res.len(),dinotree_res.len());
        naive_res.sort();
        dinotree_res.sort();

        let res = naive_res.iter().zip(dinotree_res.iter()).fold(true,|acc,(a,b)|{
            acc & (*a==*b)
        });

        assert!(res);
    }


    pub fn assert_raycast_mut(
        &mut self,
        rect:axgeom::Rect<T::Num>,
        ray:raycast::Ray<T::Num>,
        rtrait:&mut impl raycast::RayTrait<N=T::Num,T=T>) where T::Inner: HasId, T::Num:core::fmt::Debug{

        //TODO need to make sure naive also restricts its search to be in just the rect.
        //Otherwise in some cases this function will panic when it shouldnt.


        let res_naive=match raycast::raycast_naive_mut(ProtectedBBoxSlice::new(self.bots),ray,rtrait){
            raycast::RayCastResult::Hit(mut a,b)=>{
                Some( (a.drain(..).map(|a|a.inner().get_id()).collect::<Vec<_>>() ,b) )   
            },
            raycast::RayCastResult::NoHit=>{
                None
            }
        };

        let mut tree=DinoTreeBuilder::new(axgeom::XAXISS,self.bots).build_seq(); 

        let res_dinotree=match raycast::raycast_mut(&mut tree,rect,ray,rtrait){
            raycast::RayCastResult::Hit(mut a,b)=>{
                Some((a.drain(..).map(|a|a.inner().get_id()).collect::<Vec<_>>() ,b))
            },
            raycast::RayCastResult::NoHit=>{
                None
            }
        };

        match (res_naive,res_dinotree){
            (Some((mut naive_bots,naive_dis)),Some((mut dinotree_bots,dinotree_dis)))=>{
                assert_eq!(naive_dis,dinotree_dis);
                assert_eq!(naive_bots.len(),dinotree_bots.len());
                //let mut naive_bots:Vec<_> = naive_bots.iter().map(|a|a.inner().get_id()).collect();
                //let mut dinotree_bots:Vec<_> = dinotree_bots.iter().map(|a|a.inner().get_id()).collect();
                naive_bots.sort();
                dinotree_bots.sort();

                let res = naive_bots.iter().zip(dinotree_bots.iter()).fold(true,|acc,(a,b)|{
                    acc & (*a==*b)
                });

                assert!(res);
            },
            (None,None)=>{},
            _=>{
                panic!("fail");
            }
        }
        
    }

    pub fn assert_find_collisions_mut(&mut self) where T::Inner: HasId{
        
        let mut naive_pairs=Vec::new();
        colfind::query_naive_mut(ProtectedBBoxSlice::new(self.bots),|a,b|{
            naive_pairs.push(IDPair::new(a.inner().get_id(),b.inner().get_id()));
        });


        let mut tree=DinoTreeBuilder::new(axgeom::XAXISS,self.bots).build_seq(); 
        
        let mut dinotree_pairs=Vec::new();
        colfind::QueryBuilder::new(&mut tree).query_seq(|a,b| {
            dinotree_pairs.push(IDPair::new(a.inner().get_id(),b.inner().get_id()));
        });


        naive_pairs.sort();
        dinotree_pairs.sort();

        let res = naive_pairs.iter().zip(dinotree_pairs.iter()).fold(true,|acc,(a,b)|{
            acc & (*a==*b)
        });
        assert!(res,"naive={} dinotree={}",naive_pairs.len(),dinotree_pairs.len());
    }

}