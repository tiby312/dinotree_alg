
/*
#[cfg(test)]
mod bap {
    use super::*;
    use cgmath::{ Point2, Vector2};
    use collision::dbvt::TreeValue;
    use collision::*;
    use collision::dbvt::DynamicBoundingVolumeTree;
    use collision::algorithm::broad_phase::DbvtBroadPhase;

    #[derive(Debug, Clone)]
    struct Value {
        pub id: u32,
        pub aabb: Aabb2<f32>,
        fat_aabb: Aabb2<f32>,
    }

    impl Value {
        pub fn new(id: u32, aabb: Aabb2<f32>) -> Self {
            Self {
                id,
                fat_aabb: aabb.add_margin(Vector2::new(3., 3.)),
                aabb,
            }
        }
    }

    impl TreeValue for Value {
        type Bound = Aabb2<f32>;

        fn bound(&self) -> &Aabb2<f32> {
            &self.aabb
        }

        fn get_bound_with_margin(&self) -> Aabb2<f32> {
            self.fat_aabb.clone()
        }
    }

    fn aabb2(minx: f32, miny: f32, maxx: f32, maxy: f32) -> Aabb2<f32> {
        Aabb2::new(Point2::new(minx, miny), Point2::new(maxx, maxy))
    }

    #[bench]
    #[ignore]
    fn colfind_3rd_part(b: &mut Bencher) {
        //let mut rng = rand::thread_rng();
        let mut tree = DynamicBoundingVolumeTree::<Value>::new();

        let mut p = PointGenerator::new(
            &test_support::make_rect((0, 1000), (0, 1000)),
            &[100, 42, 6],
        );

        for id in 0..10000 {
            let ppp = p.random_point();
            let offset_x = ppp.0 as f32;
            let offset_y = ppp.1 as f32;

            tree.insert(Value::new(
                id,
                aabb2(offset_x + 2., offset_y + 2., offset_x + 4., offset_y + 4.),
            ));
            tree.tick();
        }

        let db = DbvtBroadPhase::new();

        let aa: Vec<bool> = (0..10000).map(|_| true).collect();
        b.iter(|| {
            let cols = db.find_collider_pairs(&tree, &aa);
            black_box(cols);
        });
    }
}
*/
