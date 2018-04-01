//!Provides capability to draw the dividers of each node.
use inner_prelude::*;
use support::Numf32;
use compt::GenTree;

pub use dinotree_inner::compute_tree_height;

///The trait that your vertex object needs to implement to be used
///in the functions in this crate.
pub trait Vertex: std::default::Default + std::clone::Clone + Send {
    fn set_pos(&mut self, x: f32, y: f32);
}

///Returns the number of verticies. Pass a slice of this size to update().
pub fn get_num_verticies(height: usize) -> usize {
    let num_nodes = compt::compute_num_nodes(height);
    (num_nodes / 2) * 6
}

///Meant to then be drawn using triangles.
///User must provide a mutable slice of verticies of the length returned by get_num_verticies().
pub fn update<V: Vertex, T: SweepTrait<Num = Numf32>>(
    rect: axgeom::Rect<Numf32>,
    gentree: &DinoTree<T>,
    verticies: &mut [V],
    start_width: f32,
) {
    match &gentree.0 {
        &DynTreeEnum::Xa(ref a) => {
            self::update_inner::<XAXISS, V, T>(rect, a, verticies, start_width);
        }
        &DynTreeEnum::Ya(ref a) => {
            self::update_inner::<YAXISS, V, T>(rect, a, verticies, start_width);
        }
    }
}
///Panics if the slice given has a length not equal to what is returned by get_num_verticies().
fn update_inner<A: AxisTrait, V: Vertex, T: SweepTrait<Num = Numf32>>(
    rect: axgeom::Rect<Numf32>,
    gentree: &DynTree<A, T>,
    verticies: &mut [V],
    start_width: f32,
) {
    struct Node<'a, V: Vertex + 'a> {
        a: &'a mut [V],
    };

    let a = self::get_num_verticies(gentree.get_height());
    let b = verticies.len();
    assert_eq!(a, b);

    let height = gentree.get_height();
    let mut vert_tree = {
        let mut va = verticies;
        let nodes: GenTree<Node<V>> = GenTree::from_bfs(
            &mut || {
                let v = std::mem::replace(&mut va, &mut []);
                let (a, b) = v.split_at_mut(6);

                std::mem::replace(&mut va, b);

                Node { a: a }
            },
            gentree.get_height() - 1,
        );
        nodes
    };

    let level = gentree.get_level_desc();
    let d1 = gentree.get_iter();
    let d2 = vert_tree.create_down_mut();
    let zip = compt::LevelIter::new(d1.zip(d2), level);

    fn recc<
        'a,
        A: AxisTrait,
        T: SweepTrait<Num = Numf32> + 'a,
        V: Vertex + 'a,
        D: CTreeIterator<Item = (&'a NodeDyn<T>, &'a mut Node<'a, V>)>,
    >(
        height: usize,
        rect: Rect<Numf32>,
        d: LevelIter<D>,
        width: f32,
    ) {
        let div_axis = A::get();
        match d.next() {
            ((dd, nn), Some((left, right))) => {
                let line_axis = A::Next::get(); //axis.next();

                let range = rect.get_range(line_axis);

                let div=match nn.0.div{
                    Some(div)=>div,
                    None=>return
                };
                //let div=nn.0.div.unwrap();
                draw_node(height, *range, &div, (div_axis, dd), nn.1.a, width);

                let (b, c) = rect.subdivide(div, div_axis);

                recc::<A::Next, _, _, _>(height, b, left, width * 0.9);
                recc::<A::Next, _, _, _>(height, c, right, width * 0.9);
            }
            ((_dd, _nn), None) => {}
        }
    }
    recc::<A, _, _, _>(height, rect, zip, start_width);
}

fn draw_node<V: Vertex>(
    height: usize,
    range: Range<Numf32>,
    div: &Numf32,
    faafa: (Axis, compt::Depth),
    verticies: &mut [V],
    width: f32,
) {
    let (div_axis, level) = faafa;
    let line_axis = div_axis.next();

    let width = (((height - level.0) + 1) as f32) / (height as f32) * width;

    let a = div_axis;
    let b = line_axis;

    let mut p1 = axgeom::Vec2::new(0.0, 0.0);
    *p1.get_axis_mut(a) = div.0.into_inner();

    *p1.get_axis_mut(b) = range.start.0.into_inner();

    let mut p2 = axgeom::Vec2::new(0.0, 0.0);
    *p2.get_axis_mut(a) = div.0.into_inner();
    *p2.get_axis_mut(b) = range.end.0.into_inner();

    self::draw_line(verticies, &p1, &p2, width);
    //self::draw_line(&mut verticies[*counter*6..*counter*6+6],&p1,&p2,width);
}

fn draw_line<V: Vertex>(verticies: &mut [V], p1: &axgeom::Vec2, p2: &axgeom::Vec2, width: f32) {
    debug_assert!(verticies.len() == 6);

    //TODO make these floating points fast approx since they just graphics.
    let (p1, p2) = (*p1, *p2);

    let offset = p2 - p1;
    let len_sqr = offset.len_sqr();
    let norm = if len_sqr > 0.0001 {
        offset / len_sqr.sqrt()
    } else {
        axgeom::Vec2::new(1.0, 0.0)
    };

    let norm90 = norm.rotate90();

    let xxx = norm90 * width;
    let yyy = norm90 * -width;
    let topleft = p1 + xxx;
    let topright = p1 + yyy;
    let bottomleft = p2 + xxx;
    let bottomright = p2 + yyy;

    let topleft = topleft.get();
    let topright = topright.get();
    let bottomleft = bottomleft.get();
    let bottomright = bottomright.get();

    unsafe {
        verticies
            .get_unchecked_mut(0)
            .set_pos(*topleft.0, *topleft.1);
        verticies
            .get_unchecked_mut(1)
            .set_pos(*topright.0, *topright.1);
        verticies
            .get_unchecked_mut(2)
            .set_pos(*bottomleft.0, *bottomleft.1);
        verticies
            .get_unchecked_mut(3)
            .set_pos(*bottomright.0, *bottomright.1);
        verticies
            .get_unchecked_mut(4)
            .set_pos(*bottomleft.0, *bottomleft.1);
        verticies
            .get_unchecked_mut(5)
            .set_pos(*topright.0, *topright.1);
    }
}
