use dinotree::axgeom;
use dinotree::prelude::*;
use dinotree_alg::*;

pub trait HasId{
    fn get_id(&self)->usize;
}


#[derive(Debug,Eq,Ord,PartialOrd,PartialEq)]
struct IDPair{
    a:usize,
    b:usize
}

impl IDPair{
    pub fn new(a:usize,b:usize)->IDPair{
        let (a,b)=if a<=b{
            (a,b)
        }else{
            (b,a)
        };
        IDPair{a,b}
    }
}


pub fn assert_query<T:HasInner>(bots:&mut [T]) where T::Inner: HasId{
    
    let mut naive_pairs=Vec::new();
    colfind::query_naive_mut(bots,|mut a,mut b|{
        naive_pairs.push(IDPair::new(a.inner().get_id(),b.inner().get_id()));
    });


    let mut tree=DinoTreeBuilder::new(axgeom::XAXISS,bots).build_seq(); 
    
    let mut dinotree_pairs=Vec::new();
    colfind::QueryBuilder::new(&mut tree).query_seq(|mut a,mut b| {
        dinotree_pairs.push(IDPair::new(a.inner().get_id(),b.inner().get_id()));
    });


    naive_pairs.sort();
    dinotree_pairs.sort();

    let res = naive_pairs.iter().zip(dinotree_pairs.iter()).fold(true,|acc,(a,b)|{
        acc & (*a==*b)
    });


    assert!(res,"naive={} dinotree={}",naive_pairs.len(),dinotree_pairs.len());
}




pub struct Bot{
    id:usize,
    aabb:axgeom::Rect<i64>
}
impl HasId for Bot{
    fn get_id(&self)->usize{
        self.id
    }
}

#[test]
fn test1(){
    for &num_bots in [1000,0,1].iter(){
        let s=dists::spiral::Spiral::new([400.0,400.0],12.0,1.0);
        
        let mut bots:Vec<Bot>=s.take(num_bots).enumerate().map(|(id,pos)|{
            Bot{id,aabb:axgeom::Rect::from_point(pos.inner_as(),axgeom::vec2same(8+id as i64))}
        }).collect();

        let mut bb=create_bbox_mut(&mut bots,|b|{
            b.aabb
        });

        assert_query(&mut bb);
    }
}
