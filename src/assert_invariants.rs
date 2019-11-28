use crate::inner_prelude::*;

#[must_use]
///Returns false if the tree's invariants are not met.
pub fn assert_invariants<A:AxisTrait,N:NodeTrait>(tree:&DinoTree<A,N>)->bool{
    inner(tree.axis(), tree.vistr().with_depth(compt::Depth(0))).is_ok()
}

fn inner<A: AxisTrait, N: NodeTrait>(
    axis: A,
    iter: compt::LevelIter<Vistr<N>>,
) -> Result<(), ()> {
    fn a_bot_has_value<N: NumTrait>(it: impl Iterator<Item = N>, val: N) -> bool {
        for b in it {
            if b == val {
                return true;
            }
        }
        false
    }


    macro_rules! assert2 {
        ($bla:expr) => {
            if !$bla {
                return Err(());
            }
        };
    }

    let ((_depth, nn), rest) = iter.next();
    let nn=nn.get();
    let axis_next = axis.next();

    let f = |a: &&N::T, b: &&N::T| -> Option<core::cmp::Ordering> {
        let j=a.get()
            .get_range(axis_next)
            .start
            .cmp(&b.get().get_range(axis_next).start);
        Some(j)
    };

    {
        use is_sorted::IsSorted;
        assert2!(IsSorted::is_sorted_by(&mut nn.bots.iter(),f));
    }

    if let Some([start, end]) = rest {
        match nn.div {
            Some(div) => {
                match nn.cont {
                    Some(cont) => {
                        for bot in nn.bots.iter() {
                            assert2!(bot.get().get_range(axis).contains(*div));
                        }

                        assert2!(a_bot_has_value(
                            nn.bots.iter().map(|b| b.get().get_range(axis).start),
                            *div
                        ));

                        for bot in nn.bots.iter() {
                            assert2!(cont.contains_range(bot.get().get_range(axis)));
                        }

                        assert2!(a_bot_has_value(
                            nn.bots.iter().map(|b| b.get().get_range(axis).start),
                            cont.start
                        ));
                        assert2!(a_bot_has_value(
                            nn.bots.iter().map(|b| b.get().get_range(axis).end),
                            cont.end
                        ));
                    }
                    None => assert2!(nn.bots.is_empty()),
                }

                inner(axis_next, start)?;
                inner(axis_next, end)?;
            }
            None => {
                for (_depth, n) in start.dfs_preorder_iter().chain(end.dfs_preorder_iter()) {
                    let n=n.get();
                    assert2!(n.bots.is_empty());
                    assert2!(n.cont.is_none());
                    assert2!(n.div.is_none());
                }
            }
        }
    }
    Ok(())
}
