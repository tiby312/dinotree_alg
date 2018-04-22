include!("../../examples/support/mod.rs");

pub fn create_unordered(a: &Bot, b: &Bot) -> (usize, usize) {
    if a.id < b.id {
        (a.id, b.id)
    } else {
        (b.id, a.id)
    }
}
pub fn compair_bot_pair(a: &(usize, usize), b: &(usize, usize)) -> std::cmp::Ordering {
    if a.0 < b.0 {
        std::cmp::Ordering::Less
    } else if a.0 > b.0 {
        std::cmp::Ordering::Greater
    } else {
        if a.1 < b.1 {
            std::cmp::Ordering::Less
        } else if a.1 > b.1 {
            std::cmp::Ordering::Greater
        } else {
            std::cmp::Ordering::Equal
        }
    }
}

