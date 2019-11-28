use axgeom;
use dinotree_alg::prelude::*;
use dinotree_alg::query::assert::*;

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
