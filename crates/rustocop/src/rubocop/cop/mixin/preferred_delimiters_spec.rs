use std::collections::BTreeMap;

use super::preferred_delimiters::PreferredDelimiters;

#[test]
fn default_is_expanded_for_every_percent_type_and_specific_values_override_it() {
    let config = BTreeMap::from([("default".into(), "()".into()), ("%w".into(), "[]".into())]);
    let delimiters = PreferredDelimiters::initialize("%w", config, None);
    assert_eq!(delimiters.delimiters().unwrap(), ['[', ']']);
    assert_eq!(delimiters.preferred_delimiters().unwrap()["%q"], "()");
}

#[test]
fn invalid_keys_are_reported_together_in_sorted_config_order() {
    let config = BTreeMap::from([("bad".into(), "()".into()), ("other".into(), "[]".into())]);
    let delimiters = PreferredDelimiters::initialize("%w", config, None);
    assert_eq!(
        delimiters.ensure_valid_preferred_delimiters().unwrap_err(),
        "Invalid preferred delimiter config key: bad, other"
    );
}

#[test]
fn supplied_precomputed_delimiters_bypass_config_validation() {
    let configured = BTreeMap::from([("bad".into(), "()".into())]);
    let supplied = BTreeMap::from([("%x".into(), "{}".into())]);
    let delimiters = PreferredDelimiters::initialize("%x", configured, Some(supplied));
    assert_eq!(delimiters.delimiters().unwrap(), ['{', '}']);
}
