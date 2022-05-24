pub fn inc_or_wrap(idx: Option<usize>, max: usize) -> usize {
    match idx {
        Some(i) => {
            if i >= max - 1 {
                0
            } else {
                i + 1
            }
        }
        None => 0,
    }
}

pub fn dec_or_wrap(idx: Option<usize>, max: usize) -> usize {
    match idx {
        Some(i) => {
            match i {
                0 => max - 1,
                _ => i - 1,
            }
        }
        None => 0,
    }
}
