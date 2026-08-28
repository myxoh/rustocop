use super::WatchKind;

#[test]
fn exposes_every_watch_kind_bit() {
    assert_eq!(WatchKind::CREATE, 1);
    assert_eq!(WatchKind::CHANGE, 2);
    assert_eq!(WatchKind::DELETE, 4);
}

#[test]
fn values_preserve_the_protocol_bitmask_contract() {
    let all: i64 = WatchKind::CREATE | WatchKind::CHANGE | WatchKind::DELETE;
    assert_eq!(all, 7);
}
