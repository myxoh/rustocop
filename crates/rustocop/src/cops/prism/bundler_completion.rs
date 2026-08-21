use super::*;

define_cops! {
    OrderedGems => "Bundler/OrderedGems" => source(ordered_gems),
    GemFilename => "Bundler/GemFilename" => source(gem_filename),
}

fn gem_filename(context: &mut CopContext<'_, '_>) {
    let raw_path = context.path();
    let path = raw_path
        .find("/spec/")
        .map(|at| &raw_path[at + 1..])
        .unwrap_or_else(|| raw_path.rsplit('/').next().unwrap_or(raw_path));
    let filename = path.rsplit('/').next().unwrap_or(path);
    let style = context.policy().enforced_style("Gemfile");
    let message = match (style, filename) {
        ("Gemfile", "gems.rb") => Some(format!(
            "`gems.rb` file was found but `Gemfile` is required (file path: {path})."
        )),
        ("Gemfile", "gems.locked") => Some(format!(
            "Expected a `Gemfile.lock` with `Gemfile` but found `gems.locked` file (file path: {path})."
        )),
        ("gems.rb", "Gemfile") => Some(format!(
            "`Gemfile` was found but `gems.rb` file is required (file path: {path})."
        )),
        ("gems.rb", "Gemfile.lock") => Some(format!(
            "Expected a `gems.locked` file with `gems.rb` but found `Gemfile.lock` (file path: {path})."
        )),
        _ => None,
    };
    if let Some(message) = message {
        context.report(message, 0..0);
    }
}

#[derive(Clone)]
struct GemDeclaration {
    name: String,
    line_start: usize,
    line_end: usize,
    chunk_start: usize,
    chunk_end: usize,
    section: usize,
}

fn ordered_gems(context: &mut CopContext<'_, '_>) {
    let filename = context.path().rsplit('/').next().unwrap_or(context.path());
    if filename != "(string)"
        && !matches!(filename, "Gemfile" | "gems.rb")
        && !filename.ends_with(".gemfile")
    {
        return;
    }
    let source = context.source();
    let lines = context.source_file().lines().collect::<Vec<_>>();
    let comments_separate = context.config_bool("TreatCommentsAsGroupSeparators", false);
    let punctuation = context.config_bool("ConsiderPunctuation", false);
    let mut declarations = Vec::new();
    let mut section = 0;
    for (index, (offset, line)) in lines.iter().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.is_empty() {
            section += 1;
            continue;
        }
        if !trimmed.starts_with("gem ") && !trimmed.starts_with("gem(") {
            if !trimmed.starts_with('#') && !line.starts_with(char::is_whitespace) {
                section += 1;
            }
            continue;
        }
        let arguments = trimmed
            .strip_prefix("gem")
            .unwrap_or_default()
            .trim_start_matches('(')
            .trim_start();
        if !arguments.starts_with(['\'', '"']) {
            section += 1;
            continue;
        }
        let Some(quote) = trimmed.find(['\'', '"']) else {
            section += 1;
            continue;
        };
        let delimiter = trimmed.as_bytes()[quote] as char;
        let Some(end_quote) = trimmed[quote + 1..].find(delimiter) else {
            continue;
        };
        let name = trimmed[quote + 1..quote + 1 + end_quote].to_string();
        let mut chunk_start = *offset;
        if !comments_separate {
            let mut previous = index;
            while previous > 0 && lines[previous - 1].1.trim_start().starts_with('#') {
                previous -= 1;
                chunk_start = lines[previous].0;
            }
        }
        let mut chunk_end = offset
            + line.len()
            + usize::from(source.as_bytes().get(offset + line.len()) == Some(&b'\n'));
        let mut next = index + 1;
        while next < lines.len()
            && lines[next].1.starts_with(char::is_whitespace)
            && !lines[next].1.trim_start().starts_with("gem ")
            && !lines[next].1.trim_start().starts_with('#')
        {
            chunk_end = lines[next].0
                + lines[next].1.len()
                + usize::from(
                    source.as_bytes().get(lines[next].0 + lines[next].1.len()) == Some(&b'\n'),
                );
            next += 1;
        }
        declarations.push(GemDeclaration {
            name,
            line_start: *offset,
            line_end: offset + line.len(),
            chunk_start,
            chunk_end,
            section,
        });
    }
    let normalized = |name: &str| {
        if punctuation {
            name.to_ascii_lowercase()
        } else {
            name.chars()
                .filter(|character| character.is_alphanumeric())
                .collect::<String>()
                .to_ascii_lowercase()
        }
    };
    for pair in declarations.windows(2) {
        let (previous, current) = (&pair[0], &pair[1]);
        if previous.section != current.section
            || comments_separate && current.chunk_start > previous.chunk_end
            || normalized(&previous.name) <= normalized(&current.name)
        {
            continue;
        }
        let mut section_declarations = declarations
            .iter()
            .filter(|declaration| declaration.section == current.section)
            .collect::<Vec<_>>();
        section_declarations.sort_by_key(|declaration| normalized(&declaration.name));
        let section_start = declarations
            .iter()
            .find(|declaration| declaration.section == current.section)
            .map_or(current.chunk_start, |declaration| declaration.chunk_start);
        let section_end = declarations
            .iter()
            .rev()
            .find(|declaration| declaration.section == current.section)
            .map_or(current.chunk_end, |declaration| declaration.chunk_end);
        let replacement = section_declarations
            .iter()
            .map(|declaration| &source[declaration.chunk_start..declaration.chunk_end])
            .collect::<String>();
        let indentation = source[current.line_start..current.line_end].len()
            - source[current.line_start..current.line_end]
                .trim_start()
                .len();
        context.replace(
            format!("Gems should be sorted in an alphabetical order within their section of the Gemfile. Gem `{}` should appear before `{}`.", current.name, previous.name),
            current.line_start + indentation..current.line_end,
            section_start..section_end,
            replacement,
        );
    }
}
