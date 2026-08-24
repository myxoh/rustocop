pub(crate) mod prism;
pub(crate) mod text;

pub(crate) const INTENTIONALLY_PENDING_COP_NAMES: &[&str] = &[];

pub(crate) fn intentionally_pending(cop: &str) -> bool {
    INTENTIONALLY_PENDING_COP_NAMES.binary_search(&cop).is_ok()
}

pub(crate) fn cop_names() -> Vec<&'static str> {
    let mut names = prism::cop_names();
    names.extend(text::LEGACY_COP_NAMES);
    names.retain(|cop| !intentionally_pending(cop));
    names.sort_unstable();
    names.dedup();
    names
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn public_inventory_is_sorted_unique_and_complete() {
        let names = cop_names();
        assert_eq!(names.len(), 622);
        assert!(names.windows(2).all(|pair| pair[0] < pair[1]));
        assert!(INTENTIONALLY_PENDING_COP_NAMES
            .windows(2)
            .all(|pair| pair[0] < pair[1]));
        assert!(INTENTIONALLY_PENDING_COP_NAMES
            .iter()
            .all(|cop| !names.contains(cop)));
        assert!(text::LEGACY_COP_NAMES
            .windows(2)
            .all(|pair| pair[0] < pair[1]));
    }
}
