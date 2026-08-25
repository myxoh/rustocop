//! Native typed Prism equivalents for rubocop-ast marker node classes and the
//! Prism builder adapter. These components add no Ruby-side behavior beyond
//! selecting a node class or parser builder.
// Source: lib/rubocop/ast/rubocop_compatibility.rb
// Source SHA-256: b7c248d935c127ca7510413dcd7fdea39af1342185fcf7fd59c64987d3f0e233

pub(crate) fn incompatible_cops(rubocop_version: &str) -> Vec<&'static str> {
    [
        ("0.89.0", "Layout/LineLength"),
        ("0.92.0", "Style/MixinUsage"),
    ]
    .into_iter()
    .filter_map(|(minimum, cop)| version_less_than(rubocop_version, minimum).then_some(cop))
    .collect()
}

pub(crate) fn compatibility_warning(rubocop_version: &str) -> Option<String> {
    let cops = incompatible_cops(rubocop_version);
    (!cops.is_empty()).then(|| {
        format!(
            "*** WARNING – Incompatible versions of `rubocop` and `rubocop-ast`\n\
You may encounter issues with the following Cop{}: {}\n\
Please upgrade rubocop to at least v0.92.0\n",
            if cops.len() > 1 { "s" } else { "" },
            cops.join(", ")
        )
    })
}

fn version_less_than(left: &str, right: &str) -> bool {
    fn components(version: &str) -> Vec<u64> {
        version
            .split(['.', '-'])
            .map_while(|part| part.parse().ok())
            .collect()
    }
    let mut left = components(left);
    let mut right = components(right);
    let length = left.len().max(right.len());
    left.resize(length, 0);
    right.resize(length, 0);
    left < right
}

#[cfg(test)]
mod tests {
    // Ported from rubocop-ast 1.49.1:
    // spec/rubocop/ast/ext/set_spec.rb
    // Spec SHA-256: 4396566ef4bcdc17702dcff6c4a1605b0ea8263dc5d5948948f2b5e007a7dd43
    // spec/rubocop/ast/rubocop_compatibility_spec.rb
    // Spec SHA-256: 223dbef6bc5d8d5c1b0cd9a897b5433bd363c82e84dd110632483a71a5c1b7b8
    use std::collections::HashSet;

    use ruby_prism::{parse, Node};

    use super::{compatibility_warning, incompatible_cops};

    fn first_is(source: &[u8], predicate: impl FnOnce(Node<'_>) -> bool) {
        let parsed = parse(source);
        let first = parsed
            .node()
            .as_program_node()
            .unwrap()
            .statements()
            .body()
            .first()
            .unwrap();
        assert!(predicate(first), "{}", String::from_utf8_lossy(source));
    }

    #[test]
    fn prism_builder_and_marker_nodes_have_typed_native_equivalents() {
        first_is(b"break", |node| node.as_break_node().is_some());
        first_is(b"1i", |node| node.as_imaginary_node().is_some());
        first_is(b"Foo", |node| node.as_constant_read_node().is_some());
        first_is(b"1.0", |node| node.as_float_node().is_some());
        first_is(b"1", |node| node.as_integer_node().is_some());
        first_is(b"next", |node| node.as_next_node().is_some());
        first_is(b"1r", |node| node.as_rational_node().is_some());
        first_is(b"return", |node| node.as_return_node().is_some());
        first_is(b":name", |node| node.as_symbol_node().is_some());
        first_is(b"[]", |node| node.as_array_node().is_some());
        first_is(b"{}", |node| node.as_hash_node().is_some());
    }

    #[test]
    fn native_set_membership_matches_rubys_case_equality_extension() {
        assert!(HashSet::from([1, 2, 3]).contains(&2));
    }

    #[test]
    fn rubocop_version_compatibility_warnings_match_the_pinned_contract() {
        assert_eq!(
            incompatible_cops("0.42.0"),
            ["Layout/LineLength", "Style/MixinUsage"]
        );
        let warning = compatibility_warning("0.42.0").unwrap();
        assert!(warning.contains("Cops: Layout/LineLength, Style/MixinUsage"));
        assert!(compatibility_warning("0.92.0").is_none());
        assert!(compatibility_warning("1.87.0").is_none());
    }
}
