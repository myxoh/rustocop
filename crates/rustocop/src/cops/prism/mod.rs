use ruby_prism::{parse, CallNode, Node, Visit};
use std::sync::Arc;

mod additional_rules;
mod additional_rules_literals;
mod additional_rules_more;
mod cop_context;
mod diagnostic;
mod dsl;
mod layout;
mod lint;
mod lint_control_flow;
mod lint_suspicious_calls;
mod matchers;
mod parity_calls;
mod parity_source;
mod security;
mod source_file;
mod source_helpers;
mod source_rules;
mod source_rules_layout;
mod source_rules_misc;
mod style;
mod style_calls;
mod style_collections;
mod style_compat;
mod style_rewrites;
mod style_source;

use crate::config::{CopConfig, RubyVersion};
use cop_context::{CopContext, CopPolicy};
use diagnostic::{Context, Reporter};
pub(crate) use diagnostic::{Finding, Inspection};
use dsl::*;
use matchers::*;
use source_file::SourceFile;

pub(super) trait Cop: Sync {
    fn name(&self) -> &'static str;

    fn on_source(&self, _source: &str, _context: &mut Context) {}

    fn on_node<'pr>(
        &self,
        node: &Node<'pr>,
        _ancestors: &[Node<'pr>],
        _source: &str,
        context: &mut Context,
    ) {
        if let Some(call) = node.as_call_node() {
            self.on_call(&call, context);
        }
    }

    fn on_call(&self, _node: &CallNode<'_>, _context: &mut Context) {}
}

struct Registry {
    cops: Vec<Box<dyn Cop>>,
}

pub struct Engine {
    registry: Registry,
}

impl Engine {
    pub fn new(enabled: &dyn Fn(&str) -> bool) -> Self {
        let registry = Registry::enabled(enabled);
        Self { registry }
    }

    pub(crate) fn implements(&self, cop_name: &str) -> bool {
        self.registry.cops.iter().any(|cop| cop.name() == cop_name)
    }

    pub fn inspect(
        &self,
        path: &str,
        source: &str,
        autocorrect: bool,
        target_ruby_version: RubyVersion,
        cop_config: Arc<CopConfig>,
    ) -> Inspection {
        let parsed = parse(source.as_bytes());
        let mut context = Context::new(autocorrect, path, target_ruby_version, cop_config);
        for cop in &self.registry.cops {
            cop.on_source(source, &mut context);
        }
        let mut runner = Runner {
            registry: &self.registry,
            context: &mut context,
            source,
            ancestors: Vec::new(),
        };
        runner.visit(&parsed.node());
        context.finish(source)
    }
}

pub(crate) fn cop_names() -> Vec<&'static str> {
    let mut names = Registry::enabled(&|_| true)
        .cops
        .into_iter()
        .map(|cop| cop.name())
        .collect::<Vec<_>>();
    names.sort_unstable();
    names.dedup();
    names
}

impl Registry {
    fn enabled(enabled: &dyn Fn(&str) -> bool) -> Self {
        let cops = lint::cops()
            .into_iter()
            .chain(additional_rules::cops())
            .chain(additional_rules_literals::cops())
            .chain(additional_rules_more::cops())
            .chain(lint_control_flow::cops())
            .chain(lint_suspicious_calls::cops())
            .chain(layout::cops())
            .chain(parity_calls::cops())
            .chain(parity_source::cops())
            .chain(security::cops())
            .chain(style::cops())
            .chain(style_calls::cops())
            .chain(style_collections::cops())
            .chain(style_compat::cops())
            .chain(style_rewrites::cops())
            .chain(style_source::cops())
            .chain(source_rules::cops())
            .chain(source_rules_layout::cops())
            .chain(source_rules_misc::cops());

        Self {
            cops: cops.filter(|cop| enabled(cop.name())).collect(),
        }
    }
}

struct Runner<'registry, 'context> {
    registry: &'registry Registry,
    context: &'context mut Context,
    source: &'context str,
    ancestors: Vec<Node<'context>>,
}

impl<'pr> Visit<'pr> for Runner<'_, 'pr> {
    fn visit_branch_node_enter(&mut self, node: Node<'pr>) {
        for cop in &self.registry.cops {
            cop.on_node(&node, &self.ancestors, self.source, self.context);
        }
        self.ancestors.push(node);
    }

    fn visit_branch_node_leave(&mut self) {
        self.ancestors.pop();
    }

    fn visit_leaf_node_enter(&mut self, node: Node<'pr>) {
        for cop in &self.registry.cops {
            cop.on_node(&node, &self.ancestors, self.source, self.context);
        }
    }
}

#[cfg(test)]
fn inspect(
    source: &str,
    autocorrect: bool,
    target_ruby_version: RubyVersion,
    enabled: &dyn Fn(&str) -> bool,
) -> Inspection {
    Engine::new(enabled).inspect(
        "example.rb",
        source,
        autocorrect,
        target_ruby_version,
        Arc::new(CopConfig::default()),
    )
}

#[cfg(test)]
mod tests {
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
}
