use super::*;

#[test]
fn parses_once_and_dispatches_to_enabled_cops() {
    let inspection = inspect(
        "eval(code)\nJSON.load(payload)\n",
        false,
        RubyVersion::default(),
        &|cop| matches!(cop, "Security/Eval" | "Security/JSONLoad"),
    );

    assert_eq!(inspection.findings.len(), 2);
    assert_eq!(inspection.findings[0].cop_name, "Security/Eval");
    assert_eq!(inspection.findings[1].cop_name, "Security/JSONLoad");
}

#[test]
fn applies_non_overlapping_autocorrections_from_the_shared_tree() {
    let inspection = inspect(
        "JSON.load(payload)\nIO.read(path)\n",
        true,
        RubyVersion::default(),
        &|cop| matches!(cop, "Security/JSONLoad" | "Security/IoMethods"),
    );

    assert_eq!(
        inspection.corrected_source,
        "JSON.parse(payload)\nFile.read(path)\n"
    );
    assert!(inspection.findings.iter().all(|finding| finding.corrected));
}

#[test]
fn swaps_unless_else_branches_with_their_inline_comments() {
    let source = concat!(
        "unless ready? # negative\n",
        "  wait\n",
        "else # positive\n",
        "  run\n",
        "end\n",
    );
    let inspection = inspect(source, true, RubyVersion::default(), &|cop| {
        cop == "Style/UnlessElse"
    });

    assert_eq!(inspection.findings.len(), 1);
    assert!(inspection.findings[0].corrected);
    assert_eq!(
        inspection.corrected_source,
        concat!(
            "if ready? # positive\n",
            "  run\n",
            "else # negative\n",
            "  wait\n",
            "end\n",
        )
    );
}

#[test]
fn applies_compatibility_batch_corrections_from_one_tree() {
    let source = concat!(
        "{a:3}\n",
        "STDOUT.puts('hello')\n",
        "require 'foo.rb'\n",
        "super name, age\n",
        "test { |a, b,| a + b }\n",
        "while cond do\nend\n",
    );
    let inspection = inspect(source, true, RubyVersion::default(), &|cop| {
        matches!(
            cop,
            "Layout/SpaceAfterColon"
                | "Style/GlobalStdStream"
                | "Style/RedundantFileExtensionInRequire"
                | "Style/SuperWithArgsParentheses"
                | "Style/TrailingCommaInBlockArgs"
                | "Style/WhileUntilDo"
        )
    });

    assert_eq!(inspection.findings.len(), 6);
    assert_eq!(
        inspection.corrected_source,
        concat!(
            "{a: 3}\n",
            "$stdout.puts('hello')\n",
            "require 'foo'\n",
            "super(name, age)\n",
            "test { |a, b| a + b }\n",
            "while cond\nend\n",
        )
    );
}

#[test]
fn replaces_empty_append_file_open_block() {
    let inspection = inspect(
        "File.open(filename, 'a') {}\n",
        true,
        RubyVersion::default(),
        &|cop| cop == "Style/FileTouch",
    );

    assert_eq!(inspection.findings.len(), 1);
    assert_eq!(inspection.corrected_source, "FileUtils.touch(filename)\n");
}

#[test]
fn public_prism_registry_is_sorted_and_unique() {
    let names = cop_names();
    assert!(names.windows(2).all(|pair| pair[0] < pair[1]));
    assert!(names.contains(&"Security/Eval"));
}

#[test]
fn target_ruby_version_is_available_to_cops() {
    let ruby_30 = inspect(
        "YAML.load(payload)\n",
        false,
        RubyVersion::new(3, 0),
        &|cop| cop == "Security/YAMLLoad",
    );
    let ruby_31 = inspect(
        "YAML.load(payload)\n",
        false,
        RubyVersion::new(3, 1),
        &|cop| cop == "Security/YAMLLoad",
    );

    assert_eq!(ruby_30.findings.len(), 1);
    assert!(ruby_31.findings.is_empty());
}

#[test]
fn corrects_verified_collection_call_and_condition_cops() {
    let source = concat!(
        "arr[0]\n",
        "items.flatten.join\n",
        "service::call\n",
        "if /foo/\nend\n",
        "items.sort_by { |item| item }\n",
    );
    let inspection = inspect(source, true, RubyVersion::new(3, 4), &|cop| {
        matches!(
            cop,
            "Style/ArrayFirstLast"
                | "Style/RedundantArrayFlatten"
                | "Style/ColonMethodCall"
                | "Lint/RegexpAsCondition"
                | "Style/RedundantSortBy"
        )
    });

    assert_eq!(inspection.findings.len(), 5);
    assert_eq!(
        inspection.corrected_source,
        concat!(
            "arr.first\n",
            "items.join\n",
            "service.call\n",
            "if /foo/ =~ $_\nend\n",
            "items.sort\n",
        )
    );
}

#[test]
fn leaves_chained_bracket_access_unchanged() {
    let inspection = inspect("arr[0][-1]\n", true, RubyVersion::default(), &|cop| {
        cop == "Style/ArrayFirstLast"
    });

    assert!(inspection.findings.is_empty());
    assert_eq!(inspection.corrected_source, "arr[0][-1]\n");
}

#[test]
fn runs_verified_suspicious_call_and_control_flow_cops() {
    let source = concat!(
        "a.x == a.x\n",
        "hash.key?(value.object_id)\n",
        "rand(1)\n",
        "return unless value&.empty?\n",
        "begin\n  work\nend while active\n",
    );
    let inspection = inspect(source, true, RubyVersion::default(), &|cop| {
        matches!(
            cop,
            "Lint/BinaryOperatorWithIdenticalOperands"
                | "Lint/HashCompareByIdentity"
                | "Lint/RandOne"
                | "Lint/SafeNavigationWithEmpty"
                | "Lint/Loop"
        )
    });

    assert_eq!(inspection.findings.len(), 5);
    assert_eq!(
        inspection.corrected_source,
        concat!(
            "a.x == a.x\n",
            "hash.key?(value.object_id)\n",
            "rand(1)\n",
            "return unless value && value.empty?\n",
            "loop do\n  work\nbreak unless active\nend\n",
        )
    );
}

#[test]
fn registers_the_twenty_cop_parity_batch() {
    let names = cop_names();
    for cop in [
        "Bundler/GemVersion",
        "Layout/InitialIndentation",
        "Layout/MultilineArrayLineBreaks",
        "Lint/DuplicateMagicComment",
        "Lint/EmptyInterpolation",
        "Lint/ErbNewArguments",
        "Lint/HashNewWithKeywordArgumentsAsDefault",
        "Lint/InterpolationCheck",
        "Lint/LambdaWithoutLiteralBlock",
        "Lint/RequireRangeParentheses",
        "Lint/RequireRelativeSelfPath",
        "Lint/SharedMutableDefault",
        "Lint/TopLevelReturnWithArgument",
        "Naming/AsciiIdentifiers",
        "Style/MultilineIfThen",
        "Style/OptionalArguments",
        "Style/OptionalBooleanParameter",
        "Style/ReturnNil",
        "Style/Send",
        "Style/VariableInterpolation",
    ] {
        assert!(names.contains(&cop), "missing {cop}");
    }
}

#[test]
fn corrects_representative_parity_batch_offenses_together() {
    let source = concat!(
        "return nil\n",
        "send(:work)\n",
        "Hash.new(key: :value)\n",
        "lambda(&callback)\n",
    );
    let inspection = inspect(source, true, RubyVersion::default(), &|cop| {
        matches!(
            cop,
            "Style/ReturnNil"
                | "Style/Send"
                | "Lint/HashNewWithKeywordArgumentsAsDefault"
                | "Lint/LambdaWithoutLiteralBlock"
        )
    });

    assert_eq!(inspection.findings.len(), 4);
    assert_eq!(
        inspection.corrected_source,
        concat!(
            "return\n",
            "send(:work)\n",
            "Hash.new({key: :value})\n",
            "callback\n",
        )
    );
}
