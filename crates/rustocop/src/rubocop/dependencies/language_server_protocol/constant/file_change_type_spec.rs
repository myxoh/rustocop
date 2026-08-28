use super::FileChangeType;

#[test]
fn exposes_every_file_change_type() {
    assert_eq!(FileChangeType::CREATED, 1);
    assert_eq!(FileChangeType::CHANGED, 2);
    assert_eq!(FileChangeType::DELETED, 3);
}

#[test]
fn exposes_integer_protocol_discriminants() {
    let kinds: [i64; 3] = [
        FileChangeType::CREATED,
        FileChangeType::CHANGED,
        FileChangeType::DELETED,
    ];

    assert_eq!(kinds, [1, 2, 3]);
}
