use std::fs;

use std::sync::Arc;

use crate::config::{
    CopConfig, CopSelection, InspectionConfig, Parallelism, RubyVersion, RunOptions,
};

pub(super) enum Command {
    Run(RunOptions),
    Version,
    VerboseVersion,
    ShowCops,
}

pub(super) fn parse_args(mut args: Vec<String>) -> Result<Command, String> {
    let mut options = RunOptions {
        files: Vec::new(),
        format: "simple".to_string(),
        stdin_path: None,
        parallelism: Parallelism::Sequential,
        rubocop_loaders: Vec::new(),
        config_path: None,
        inspection: InspectionConfig {
            autocorrect: false,
            cops: CopSelection::default_enabled(),
            target_ruby_version: RubyVersion::default(),
            cop_config: Arc::new(CopConfig::default()),
        },
    };

    while !args.is_empty() {
        let arg = args.remove(0);
        match arg.as_str() {
            "--version" => return Ok(Command::Version),
            "-V" => return Ok(Command::VerboseVersion),
            "--show-cops" => return Ok(Command::ShowCops),
            "-A" | "-a" | "--autocorrect" | "--autocorrect-all" | "--auto-correct"
            | "--auto-correct-all" => options.inspection.autocorrect = true,
            "--format" | "-f" => options.format = take_value(&mut args, &arg)?,
            "--only" => {
                options.inspection.cops = CopSelection::only(&take_value(&mut args, &arg)?);
            }
            "--stdin" => options.stdin_path = Some(take_value(&mut args, &arg)?),
            "--parallel" => options.parallelism = Parallelism::Automatic,
            "--no-parallel" => options.parallelism = Parallelism::Sequential,
            "--jobs" => {
                options.parallelism =
                    Parallelism::Fixed(parse_jobs(&take_value(&mut args, &arg)?)?);
            }
            "--config" | "-c" => {
                let path = take_value(&mut args, &arg)?;
                apply_config(&mut options.inspection, &path)?;
                options.config_path = Some(path);
            }
            "--require" | "--plugin" => {
                let value = take_value(&mut args, &arg)?;
                options.rubocop_loaders.push((arg, value));
            }
            "--force-exclusion" | "--no-server" | "--display-cop-names" | "--extra-details" => {}
            "--cache" => {
                if args.first().is_some_and(|value| !value.starts_with('-')) {
                    args.remove(0);
                }
            }
            "--" => {
                options.files.extend(args);
                break;
            }
            _ if arg.starts_with("--format=") => {
                options.format = arg
                    .strip_prefix("--format=")
                    .unwrap_or_default()
                    .to_string();
            }
            _ if arg.starts_with("--only=") => {
                options.inspection.cops =
                    CopSelection::only(arg.strip_prefix("--only=").unwrap_or_default());
            }
            _ if arg.starts_with("--stdin=") => {
                options.stdin_path =
                    Some(arg.strip_prefix("--stdin=").unwrap_or_default().to_string());
            }
            _ if arg.starts_with("--config=") => {
                let path = arg.strip_prefix("--config=").unwrap_or_default();
                apply_config(&mut options.inspection, path)?;
                options.config_path = Some(path.to_string());
            }
            _ if arg.starts_with("--jobs=") => {
                let value = arg.strip_prefix("--jobs=").unwrap_or_default();
                options.parallelism = Parallelism::Fixed(parse_jobs(value)?);
            }
            _ if arg.starts_with("--require=") || arg.starts_with("--plugin=") => {
                let (name, value) = arg.split_once('=').unwrap_or((&arg, ""));
                options
                    .rubocop_loaders
                    .push((name.to_string(), value.to_string()));
            }
            _ if arg.starts_with('-') => return Err(format!("unsupported option {arg}")),
            _ => options.files.push(arg),
        }
    }

    if options.format != "json" && options.format != "simple" {
        return Err(format!("unsupported formatter {}", options.format));
    }
    Ok(Command::Run(options))
}

fn take_value(args: &mut Vec<String>, option: &str) -> Result<String, String> {
    if args.is_empty() {
        return Err(format!("missing value for {option}"));
    }
    Ok(args.remove(0))
}

fn parse_jobs(value: &str) -> Result<usize, String> {
    value
        .parse::<usize>()
        .ok()
        .filter(|jobs| *jobs > 0)
        .ok_or_else(|| format!("invalid worker count {value}"))
}

fn apply_config(config: &mut InspectionConfig, path: &str) -> Result<(), String> {
    let source = fs::read_to_string(path)
        .map_err(|error| format!("could not read config {path}: {error}"))?;
    config.target_ruby_version = target_ruby_version_from_source(&source).unwrap_or_default();
    config.cop_config = Arc::new(CopConfig::from_source(&source));
    Ok(())
}

fn target_ruby_version_from_source(source: &str) -> Option<RubyVersion> {
    source.lines().find_map(|line| {
        let value = line.trim().strip_prefix("TargetRubyVersion:")?;
        RubyVersion::parse(value.split('#').next()?.trim())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_target_ruby_version_from_rubocop_config() {
        let config = "AllCops:\n  TargetRubyVersion: 3.1 # compatibility target\n";
        assert_eq!(
            target_ruby_version_from_source(config),
            Some(RubyVersion::new(3, 1))
        );
    }

    #[test]
    fn ignores_unrelated_configuration() {
        assert_eq!(
            target_ruby_version_from_source("AllCops:\n  NewCops: enable\n"),
            None
        );
    }

    #[test]
    fn parses_parallel_worker_options() {
        let Command::Run(automatic) = parse_args(vec!["--parallel".to_string()]).unwrap() else {
            panic!("expected run command");
        };
        let Command::Run(fixed) = parse_args(vec!["--jobs".to_string(), "4".to_string()]).unwrap()
        else {
            panic!("expected run command");
        };
        assert_eq!(automatic.parallelism, Parallelism::Automatic);
        assert_eq!(fixed.parallelism, Parallelism::Fixed(4));
    }

    #[test]
    fn rejects_invalid_parallel_worker_count() {
        assert_eq!(
            parse_args(vec!["--jobs=0".to_string()]).err().as_deref(),
            Some("invalid worker count 0")
        );
    }

    #[test]
    fn preserves_custom_cop_loaders_for_rubocop() {
        let Command::Run(options) = parse_args(vec![
            "--require".to_string(),
            "custom/cop.rb".to_string(),
            "--plugin=custom-plugin".to_string(),
        ])
        .unwrap() else {
            panic!("expected run command");
        };

        assert_eq!(
            options.rubocop_loaders,
            [
                ("--require".to_string(), "custom/cop.rb".to_string()),
                ("--plugin".to_string(), "custom-plugin".to_string())
            ]
        );
    }

    #[test]
    fn rejects_an_unreadable_requested_config() {
        let missing = format!("{}/missing-rubocop.yml", env!("CARGO_MANIFEST_DIR"));
        let error = parse_args(vec![format!("--config={missing}")])
            .err()
            .expect("missing config should fail");

        assert!(error.starts_with(&format!("could not read config {missing}:")));
    }
}
