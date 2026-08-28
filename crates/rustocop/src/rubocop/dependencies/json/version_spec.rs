use super::VERSION;

#[test]
fn exposes_the_upstream_json_version() {
    assert_eq!(VERSION, "2.19.8");
}

#[test]
fn exposes_the_version_as_a_static_utf8_string() {
    let version: &'static str = VERSION;

    assert_eq!(version.as_bytes(), b"2.19.8");
}
