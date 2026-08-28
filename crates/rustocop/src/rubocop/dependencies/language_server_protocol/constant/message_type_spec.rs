use super::MessageType;

#[test]
fn exposes_every_message_type() {
    assert_eq!(MessageType::ERROR, 1);
    assert_eq!(MessageType::WARNING, 2);
    assert_eq!(MessageType::INFO, 3);
    assert_eq!(MessageType::LOG, 4);
}

#[test]
fn exposes_integer_protocol_discriminants() {
    let types: [i64; 4] = [
        MessageType::ERROR,
        MessageType::WARNING,
        MessageType::INFO,
        MessageType::LOG,
    ];

    assert_eq!(types, [1, 2, 3, 4]);
}
