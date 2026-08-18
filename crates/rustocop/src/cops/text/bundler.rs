use super::helpers::*;
use super::{push_offense, Offense, SourceLine};
use crate::config::InspectionConfig;
use std::path::Path;

pub(super) fn before_prism(
    path: &str,
    lines: &mut [SourceLine],
    options: &InspectionConfig,
    offenses: &mut Vec<Offense>,
) {
    check_bundler_ordered_gems(path, lines, options, offenses);
}

fn check_bundler_ordered_gems(
    path: &str,
    lines: &mut [SourceLine],
    options: &InspectionConfig,
    offenses: &mut Vec<Offense>,
) {
    let cop = "Bundler/OrderedGems";
    if !options.cop_enabled(cop)
        || Path::new(path)
            .file_name()
            .is_none_or(|name| name != "Gemfile")
    {
        return;
    }

    let mut index = 0;
    while index < lines.len() {
        if gem_name(&lines[index].body).is_none() {
            index += 1;
            continue;
        }

        let start = index;
        while index < lines.len() && gem_name(&lines[index].body).is_some() {
            index += 1;
        }

        let mut names = lines[start..index]
            .iter()
            .filter_map(|line| gem_name(&line.body))
            .collect::<Vec<String>>();
        let sorted_names = {
            let mut sorted = names.clone();
            sorted.sort_by_key(|name| name.to_ascii_lowercase());
            sorted
        };

        if names != sorted_names {
            push_offense(
                offenses,
                cop,
                "Gems should be sorted in an alphabetical order within their section of the Gemfile.",
                start + 1,
                1,
                lines[start].body.chars().count().max(1),
                true,
                options.autocorrect,
            );

            if options.autocorrect {
                let mut section = lines[start..index].to_vec();
                section.sort_by_key(|line| {
                    gem_name(&line.body)
                        .unwrap_or_default()
                        .to_ascii_lowercase()
                });
                lines[start..index].clone_from_slice(&section);
                names = sorted_names;
            }
        }

        if names.is_empty() {
            index += 1;
        }
    }
}
