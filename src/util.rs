use std::path::Path;

/// Arg `idx` incremented by 1, if past `max`, return 0
pub fn inc_or_wrap(idx: usize, max: usize) -> usize {
    if (max == 0) || (idx >= max - 1) {
        return 0;
    } else {
        return idx + 1;
    }
}

/// Arg `idx` decremented by 1, if -1, return `max`
pub fn dec_or_wrap(idx: usize, max: usize) -> usize {
    if max == 0  {
        return 0;
    } else if idx == 0 {
        return max - 1;
    } else {
        return idx - 1;
    }
}

/// Returns true if file path exists, else false
pub fn file_exists(fp: &String) -> bool {
    return Path::new(fp.as_str()).is_file();
}

#[test]
fn inc_or_wrap_test() {
    assert_eq!(inc_or_wrap(1, 3), 2);
    assert_eq!(inc_or_wrap(2, 3), 0);
    assert_eq!(inc_or_wrap(0, 0), 0);
}

#[test]
fn dec_or_wrap_test() {
    assert_eq!(dec_or_wrap(1, 3), 0);
    assert_eq!(dec_or_wrap(0, 3), 2);
    assert_eq!(dec_or_wrap(0, 0), 0);
}

#[test]
fn file_exists_test() {
    assert_eq!(true, file_exists(&String::from("Cargo.toml")));
    assert_eq!(false, file_exists(&String::from("doesnt-exist.toml")));
}
