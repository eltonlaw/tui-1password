use tui_1password::op;
use std::fs::File;
use std::io::prelude::*;

#[test]
fn is_valid_cache() {
    // Invalid: file that doesn't exist
    let not_exist_validity = op::is_valid_cache(&String::from("tests/resources/token-doesnt-exist"));
    assert_eq!(false, not_exist_validity);
    // Invalid: old file
    let expired_validity = op::is_valid_cache(&String::from("tests/resources/token-expired"));
    assert_eq!(false, expired_validity);
    // Valid: newly created file with export
    let mut file = File::create("tests/resources/token-new").unwrap();
    file.write_all(b"export OP_SESSION_my=\"f5uyNnTPFd_9K4RCsebDvV4MR-gX-t49x413vuIKbPM\"").unwrap();
    let new_validity = op::is_valid_cache(&String::from("tests/resources/token-new"));
    assert_eq!(true, new_validity);
}

#[test]
fn session_from_cache() {
    let sess = op::Session::from_cache(&String::from("tests/resources/token-expired"));
    assert_eq!("OP_SESSION_my", sess.name);
    assert_eq!("f5uyNnTPFd_9K4RCsebDvV4MR-gX-t49x413vuIKbPM", sess.token);
}
