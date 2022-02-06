use tui_1password;
use std::fs::File;
use std::io::prelude::*;

#[test]
fn is_valid_cache() {
    // Invalid: file that doesn't exist
    let not_exist_validity = tui_1password::is_valid_cache("tests/resources/1password-doesnt-exist");
    assert_eq!(false, not_exist_validity);
    // Invalid: old file
    let expired_validity = tui_1password::is_valid_cache("tests/resources/1password-expired");
    assert_eq!(false, expired_validity);
    // Valid: newly created file with export
    let mut file = File::create("tests/resources/1password-new").unwrap();
    file.write_all(b"export OP_SESSION_my=\"f5uyNnTPFd_9K4RCsebDvV4MR-gX-t49x413vuIKbPM\"").unwrap();
    let new_validity = tui_1password::is_valid_cache("tests/resources/1password-new");
    assert_eq!(true, new_validity);
}

#[test]
fn session_from_cache() {
    let sess = tui_1password::Session::from_cache("tests/resources/1password-expired");
    assert_eq!("OP_SESSION_my", sess.name);
    assert_eq!("\"f5uyNnTPFd_9K4RCsebDvV4MR-gX-t49x413vuIKbPM\"", sess.token);
}

#[test]
fn list_items() {
    let sess = tui_1password::Session::from_cache("tests/resources/1password-expired");
    assert_eq!("OP_SESSION_my", sess.name);
    assert_eq!("\"f5uyNnTPFd_9K4RCsebDvV4MR-gX-t49x413vuIKbPM\"", sess.token);
}
