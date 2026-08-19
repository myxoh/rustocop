pub(crate) mod prism;
pub(crate) mod text;

pub(crate) fn cop_names() -> Vec<&'static str> {
    let mut names = prism::cop_names();
    names.extend(text::LEGACY_COP_NAMES);
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
        assert!(text::LEGACY_COP_NAMES
            .windows(2)
            .all(|pair| pair[0] < pair[1]));
    }
}
