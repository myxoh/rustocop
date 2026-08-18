use super::helpers::*;
use super::{push_offense, Offense, SourceLine};
use crate::config::InspectionConfig;

pub(super) fn after_prism(
    path: &str,
    lines: &[SourceLine],
    options: &InspectionConfig,
    offenses: &mut Vec<Offense>,
) {
    check_rails(path, lines, options, offenses);
    check_rspec(path, lines, options, offenses);
}

fn check_rails(
    path: &str,
    lines: &[SourceLine],
    options: &InspectionConfig,
    offenses: &mut Vec<Offense>,
) {
    for (index, line) in lines.iter().enumerate() {
        let text = strip_comment(&line.body);

        if options.cop_enabled("Rails/DefaultScope") && text.contains("default_scope") {
            push_offense(
                offenses,
                "Rails/DefaultScope",
                "Avoid using `default_scope`.",
                index + 1,
                text.find("default_scope").unwrap_or(0) + 1,
                "default_scope".len(),
                false,
                false,
            );
        }

        if options.cop_enabled("Rails/FilePath")
            && (text.contains("File.join(Rails.root") || text.contains("Rails.root.to_s"))
        {
            push_offense(
                offenses,
                "Rails/FilePath",
                "Prefer `Rails.root.join`.",
                index + 1,
                text.find("Rails.root").unwrap_or(0) + 1,
                "Rails.root".len(),
                false,
                false,
            );
        }

        if options.cop_enabled("Rails/ApplicationJob")
            && path.contains("/app/jobs/")
            && text.contains("< ActiveJob::Base")
        {
            push_offense(
                offenses,
                "Rails/ApplicationJob",
                "Jobs should subclass `ApplicationJob`.",
                index + 1,
                text.find("ActiveJob::Base").unwrap_or(0) + 1,
                "ActiveJob::Base".len(),
                false,
                false,
            );
        }

        if options.cop_enabled("Rails/ReversibleMigration")
            && path.contains("/db/migrate/")
            && [
                "remove_column",
                "change_column",
                "drop_table",
                "remove_index",
                "execute ",
            ]
            .iter()
            .any(|needle| text.contains(needle))
        {
            push_offense(
                offenses,
                "Rails/ReversibleMigration",
                "Use reversible migration helpers or define `up` and `down` methods.",
                index + 1,
                leading_spaces(&line.body) + 1,
                line.body.trim().len(),
                false,
                false,
            );
        }
    }
}

fn check_rspec(
    path: &str,
    lines: &[SourceLine],
    options: &InspectionConfig,
    offenses: &mut Vec<Offense>,
) {
    if !path.contains("/spec/") && !path.ends_with("_spec.rb") {
        return;
    }

    check_rspec_file_path(path, options, offenses);

    let mut group_stack = Vec::<SpecGroup>::new();
    let mut example_stack = Vec::<ExampleBlock>::new();

    for (index, line) in lines.iter().enumerate() {
        let trimmed = line.body.trim();

        if options.cop_enabled("RSpec/Focus")
            && (trimmed.starts_with("fdescribe")
                || trimmed.starts_with("fcontext")
                || trimmed.starts_with("fit")
                || trimmed.contains("focus: true"))
        {
            push_offense(
                offenses,
                "RSpec/Focus",
                "Focused spec found.",
                index + 1,
                leading_spaces(&line.body) + 1,
                trimmed.len(),
                false,
                false,
            );
        }

        if is_rspec_group_start(trimmed) {
            let depth = group_stack.len() + 1;
            if options.cop_enabled("RSpec/NestedGroups") && depth > 4 {
                push_offense(
                    offenses,
                    "RSpec/NestedGroups",
                    "Maximum example group nesting exceeded.",
                    index + 1,
                    leading_spaces(&line.body) + 1,
                    trimmed.len(),
                    false,
                    false,
                );
            }

            group_stack.push(SpecGroup {
                start: index,
                examples: 0,
                memoized_helpers: 0,
                setup_blocks: 0,
            });
        } else if is_rspec_example_start(trimmed) {
            if let Some(group) = group_stack.last_mut() {
                group.examples += 1;
            }
            example_stack.push(ExampleBlock { start: index });
        } else if options.cop_enabled("RSpec/MessageChain")
            && trimmed.contains("receive_message_chain")
        {
            push_offense(
                offenses,
                "RSpec/MessageChain",
                "Avoid stubbing using `receive_message_chain`.",
                index + 1,
                line.body.find("receive_message_chain").unwrap_or(0) + 1,
                "receive_message_chain".len(),
                false,
                false,
            );
        } else if options.cop_enabled("RSpec/PendingWithoutReason")
            && pending_without_reason(trimmed)
        {
            push_offense(
                offenses,
                "RSpec/PendingWithoutReason",
                "Give the reason for pending.",
                index + 1,
                leading_spaces(&line.body) + 1,
                trimmed.len(),
                false,
                false,
            );
        } else if trimmed.starts_with("before")
            || trimmed.starts_with("after")
            || trimmed.starts_with("around")
        {
            if let Some(group) = group_stack.last_mut() {
                group.setup_blocks += 1;
                if options.cop_enabled("RSpec/ScatteredSetup") && group.setup_blocks > 1 {
                    push_offense(
                        offenses,
                        "RSpec/ScatteredSetup",
                        "Do not scatter setup blocks throughout an example group.",
                        index + 1,
                        leading_spaces(&line.body) + 1,
                        trimmed.len(),
                        false,
                        false,
                    );
                }
            }
        } else if trimmed.starts_with("let(") || trimmed.starts_with("subject(") {
            if let Some(group) = group_stack.last_mut() {
                group.memoized_helpers += 1;
                if options.cop_enabled("RSpec/MultipleMemoizedHelpers")
                    && group.memoized_helpers > 10
                {
                    push_offense(
                        offenses,
                        "RSpec/MultipleMemoizedHelpers",
                        "Example group has too many memoized helpers.",
                        index + 1,
                        leading_spaces(&line.body) + 1,
                        trimmed.len(),
                        false,
                        false,
                    );
                }
            }

            if let Some(name) = symbol_argument(trimmed) {
                if options.cop_enabled("RSpec/VariableName") && !is_snake_case(name) {
                    push_offense(
                        offenses,
                        "RSpec/VariableName",
                        "Use snake_case for variable names.",
                        index + 1,
                        line.body.find(name).unwrap_or(0) + 1,
                        name.len(),
                        false,
                        false,
                    );
                }
            }
        }

        if trimmed == "end" {
            if let Some(example) = example_stack.pop() {
                let length = lines[example.start + 1..index]
                    .iter()
                    .filter(|line| !line.body.trim().is_empty())
                    .count();
                let expectations = lines[example.start + 1..index]
                    .iter()
                    .filter(|line| {
                        line.body.contains("expect(") || line.body.contains("is_expected")
                    })
                    .count();

                if options.cop_enabled("RSpec/ExampleLength") && length > 15 {
                    push_offense(
                        offenses,
                        "RSpec/ExampleLength",
                        &format!("Example has too many lines. [{}/15]", length),
                        example.start + 1,
                        leading_spaces(&lines[example.start].body) + 1,
                        lines[example.start].body.trim().len(),
                        false,
                        false,
                    );
                }
                if options.cop_enabled("RSpec/MultipleExpectations") && expectations > 1 {
                    push_offense(
                        offenses,
                        "RSpec/MultipleExpectations",
                        "Example has too many expectations.",
                        example.start + 1,
                        leading_spaces(&lines[example.start].body) + 1,
                        lines[example.start].body.trim().len(),
                        false,
                        false,
                    );
                }
                continue;
            }

            if let Some(group) = group_stack.pop() {
                if options.cop_enabled("RSpec/EmptyExampleGroup") && group.examples == 0 {
                    push_offense(
                        offenses,
                        "RSpec/EmptyExampleGroup",
                        "Empty example group detected.",
                        group.start + 1,
                        leading_spaces(&lines[group.start].body) + 1,
                        lines[group.start].body.trim().len(),
                        false,
                        false,
                    );
                }
            }
        }
    }
}

fn check_rspec_file_path(path: &str, options: &InspectionConfig, offenses: &mut Vec<Offense>) {
    if options.cop_enabled("RSpec/SpecFilePathSuffix") && !path.ends_with("_spec.rb") {
        push_offense(
            offenses,
            "RSpec/SpecFilePathSuffix",
            "Spec path should end with `_spec.rb`.",
            1,
            1,
            1,
            false,
            false,
        );
    }

    if options.cop_enabled("RSpec/SpecFilePathFormat")
        && path.contains("/spec/")
        && path.contains("__")
    {
        push_offense(
            offenses,
            "RSpec/SpecFilePathFormat",
            "Spec path has invalid format.",
            1,
            1,
            1,
            false,
            false,
        );
    }
}

#[derive(Debug)]
struct SpecGroup {
    start: usize,
    examples: usize,
    memoized_helpers: usize,
    setup_blocks: usize,
}

#[derive(Debug)]
struct ExampleBlock {
    start: usize,
}
