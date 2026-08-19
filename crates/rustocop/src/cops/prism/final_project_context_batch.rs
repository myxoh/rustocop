use super::catalog_cop::custom;
use super::*;

pub(super) fn cops() -> Vec<Box<dyn Cop>> {
    vec![
        custom("Bundler/GemComment", gem_comment),
        custom("Gemspec/DependencyVersion", dependency_version),
    ]
}

fn gem_comment(context: &mut CopContext<'_, '_>) {
    for (offset, line) in context.source_file().lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("gem ")
            && (trimmed.contains("git:") || trimmed.contains("path:"))
            && !trimmed.contains('#')
        {
            context.report(
                "Gem source should have a comment describing its purpose.",
                offset..offset + line.len(),
            );
        }
    }
}

fn dependency_version(context: &mut CopContext<'_, '_>) {
    if !context.path().ends_with(".gemspec") {
        return;
    }
    for (offset, line) in context.source_file().lines() {
        if [
            "add_dependency ",
            "add_runtime_dependency ",
            "add_development_dependency ",
        ]
        .iter()
        .any(|method| line.contains(method))
            && line.matches(',').count() == 0
        {
            context.report(
                "Dependency version specification is required.",
                offset..offset + line.len(),
            );
        }
    }
}
