/// Arg `idx` incremented by 1, if past `max`, return 0
pub fn inc_or_wrap(idx: Option<usize>, max: usize) -> usize {
    match max {
        0 => 0,
        _ => match idx {
            Some(i) => {
                if i >= max - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        },
    }
}

/// Arg `idx` decremented by 1, if -1, return `max`
pub fn dec_or_wrap(idx: Option<usize>, max: usize) -> usize {
    match max {
        0 => 0,
        _ => match idx {
            Some(i) => {
                match i {
                    0 => max - 1,
                    _ => i - 1,
                }
            }
            None => 0,
        },
    }
}

#[test]
fn inc_or_wrap_test() {
    assert_eq!(inc_or_wrap(Some(1), 3), 2);
    assert_eq!(inc_or_wrap(Some(2), 3), 0);
    assert_eq!(inc_or_wrap(Some(0), 0), 0);
}

#[test]
fn dec_or_wrap_test() {
    assert_eq!(dec_or_wrap(Some(1), 3), 0);
    assert_eq!(dec_or_wrap(Some(0), 3), 2);
    assert_eq!(dec_or_wrap(Some(0), 0), 0);
}
