
extern crate axgeom;
extern crate dinotree_alg;
extern crate compt;

use dinotree_alg::prelude::*;
use compt::*;
use dinotree_alg::par::*;
    


#[test]
fn test_par_heur(){
    let p = compute_level_switch_sequential(6,6);
    assert_eq!(p.get_depth_to_switch_at(),0);
}
#[test]
fn test_parallel(){
    let k=Parallel::new(0);
    match k.next(){
        ParResult::Parallel(_)=>{
            panic!("fail");
        },
        ParResult::Sequential(_)=>{
            
        }
    }
}



fn assert_length<I: std::iter::ExactSizeIterator>(it: I) {
    let len = it.size_hint().0;
    assert_eq!(it.count(), len);
}

#[test]
fn test_zero_sized() {
    let mut bots = vec![(); 1];

    let mut bots = build_helper::create_bbox_mut(&mut bots,|_b|{axgeom::Rect::new(0isize,0,0,0)});

    let tree = DinoTree::new(axgeom::YAXISS, &mut bots);

    let (n, _) = tree.vistr().next();
    let n=n.get();
    assert_eq!(n.div.is_none(), true);
    assert_eq!(n.bots.len(), 1);
    assert!(n.cont.is_some());
}

#[test]
fn test_zero_sized2() {
    let mut bots = vec![(); 1];

    let mut bots = build_helper::create_bbox_mut(&mut bots,|_b|{axgeom::Rect::new(0isize,0,0,0)});

    let tree = DinoTree::new(axgeom::YAXISS, &mut bots);
    
    let (n, _) = tree.vistr().next();
    let n=n.get();
    assert_eq!(n.div.is_none(), true);
    assert_eq!(n.bots.len(), 1);
    assert!(n.cont.is_some());
}
#[test]
fn test_one() {
    let mut bots = vec![0usize; 1];

    let mut bots = build_helper::create_bbox_mut(&mut bots,|_b|{axgeom::Rect::new(0isize,0,0,0)});

    let tree = DinoTree::new(axgeom::YAXISS, &mut bots);

    let (n, _) = tree.vistr().next();
    let n=n.get();
    assert!(n.div.is_none());
    assert_eq!(n.bots.len(), 1);
    assert!(n.cont.is_some())
}

#[test]
fn test_empty() {
    let mut bots: Vec<()> = Vec::new();
    let mut bots = build_helper::create_bbox_mut(&mut bots,|_b|{axgeom::Rect::new(0isize,0,0,0)});
    let tree = DinoTree::new(axgeom::YAXISS, &mut bots);

    let (n, _) = tree.vistr().next();
    let n=n.get();
    assert_eq!(n.bots.len(), 0);
    assert!(n.div.is_none());
    assert!(n.cont.is_none());
}


#[test]
fn test_many() {
    let mut bots = vec![0usize; 1000];

    let mut bots = build_helper::create_bbox_mut(&mut bots,|_b|{axgeom::Rect::new(0isize,0,0,0)});

    let tree = DinoTree::new(axgeom::YAXISS, &mut bots);

    
    assert_eq!(
        tree.vistr().dfs_inorder_iter().flat_map(|a|a.range.iter()).count(),
        1000
    );


    let mut num_div = 0;
    for b in tree.vistr().dfs_inorder_iter() {
        if let Some(_) = b.div {
            if let Some(_) = b.cont {
                num_div += 1;
            }
        }
    }
    assert_eq!(num_div, 0);
}

#[test]
fn test_send_sync_dinotree(){
    let mut bots1:Vec<()>=Vec::new();
    let mut bots2:Vec<()>=Vec::new();

    let mut bots1=build_helper::create_bbox_mut(&mut bots1,|_|axgeom::Rect::new(0,0,0,0));
    let mut bots2=build_helper::create_bbox_mut(&mut bots2,|_|axgeom::Rect::new(0,0,0,0));

    //Check that its send
    let (t1,t2)=rayon::join(
            ||DinoTree::new(axgeom::XAXISS,&mut bots1),
            ||DinoTree::new(axgeom::YAXISS,&mut bots2));

    //Check that its sync
    let (p1,p2)=(&t1,&t2);
    rayon::join(||{p1},||{p2});
}



#[test]
fn test() {
    let mut bots = vec![0usize; 1234];

    let mut bots = build_helper::create_bbox_mut(&mut bots,|_b|{axgeom::Rect::new(0isize,0,0,0)});

    let mut tree = DinoTree::new(axgeom::YAXISS, &mut bots);

    assert!(tree.assert_invariants());

    assert_length(tree.vistr_mut().dfs_preorder_iter());
    assert_length(tree.vistr().dfs_preorder_iter());

    let num_nodes = tree.num_nodes();

    assert_eq!(
        tree
            .vistr_mut()
            .dfs_preorder_iter()
            .size_hint()
            .0,
        num_nodes
    );

    assert_eq!(tree.vistr().dfs_preorder_iter().size_hint().0, num_nodes);

    recc(tree.vistr_mut());
    //recursively check that the length is correct at each node.
    fn recc(a: VistrMut<NodeMut<BBoxMut<isize, usize>>>) {
        let (_nn, rest) = a.next();
        match rest {
            Some([mut left, mut right]) => {
                {
                    let left = left.create_wrap_mut();
                    let right = right.create_wrap_mut();
                    assert_length(left.dfs_preorder_iter());
                    assert_length(right.dfs_preorder_iter());
                }
                recc(left);
                recc(right);
            }
            None => {}
        }
    }
}
