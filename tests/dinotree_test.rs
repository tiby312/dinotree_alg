use axgeom;
use dinotree_alg::*;


#[test]
fn test1() {
    for &num_bots in [1000, 0, 1].iter() {
        let s = dists::spiral::Spiral::new([400.0, 400.0], 12.0, 1.0);

        let mut bots: Vec<_> = s
            .take(num_bots)
            .enumerate()
            .map(|(id, pos)| bbox(axgeom::Rect::from_point(pos.inner_as(), axgeom::vec2same(8 + id as i64)),()))
            .collect();

        let mut tree = DinoTree::new(&mut bots);

        tree.assert_find_intersections_mut();
    }
}
