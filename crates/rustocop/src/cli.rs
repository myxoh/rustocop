use std::fs;

use crate::prism_engine::RubyVersion;
use crate::{Command, Options, Parallelism};

pub(super) fn parse_args(mut args: Vec<String>) -> Result<Command, String> {
    let mut options = Options {
        autocorrect: false,
        files: Vec::new(),
        format: "simple".to_string(),
        only: None,
        stdin_path: None,
        target_ruby_version: RubyVersion::default(),
        parallelism: Parallelism::Sequential,
    };

    while !args.is_empty() {
        let arg = args.remove(0);
        match arg.as_str() {
            "--version" => return Ok(Command::Version),
            "-V" => return Ok(Command::VerboseVersion),
            "--show-cops" => return Ok(Command::ShowCops),
            "-A" | "-a" | "--autocorrect" | "--autocorrect-all" | "--auto-correct"
            | "--auto-correct-all" => options.autocorrect = true,
            "--format" | "-f" => options.format = take_value(&mut args, &arg)?,
            "--only" => options.only = Some(take_value(&mut args, &arg)?),
            "--stdin" => options.stdin_path = Some(take_value(&mut args, &arg)?),
            "--parallel" => options.parallelism = Parallelism::Automatic,
            "--no-parallel" => options.parallelism = Parallelism::Sequential,
            "--jobs" => {
                options.parallelism =
                    Parallelism::Fixed(parse_jobs(&take_value(&mut args, &arg)?)?);
            }
            "--config" | "-c" => {
                let path = take_value(&mut args, &arg)?;
                options.target_ruby_version = target_ruby_version_from_config(&path);
            }
            "--require" | "--plugin" => {
                let _ = take_value(&mut args, &arg)?;
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
                options.only = Some(arg.strip_prefix("--only=").unwrap_or_default().to_string());
            }
            _ if arg.starts_with("--stdin=") => {
                options.stdin_path =
                    Some(arg.strip_prefix("--stdin=").unwrap_or_default().to_string());
            }
            _ if arg.starts_with("--config=") => {
                let path = arg.strip_prefix("--config=").unwrap_or_default();
                options.target_ruby_version = target_ruby_version_from_config(path);
            }
            _ if arg.starts_with("--jobs=") => {
                let value = arg.strip_prefix("--jobs=").unwrap_or_default();
                options.parallelism = Parallelism::Fixed(parse_jobs(value)?);
            }
            _ if arg.starts_with("--require=") || arg.starts_with("--plugin=") => {}
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

fn target_ruby_version_from_config(path: &str) -> RubyVersion {
    fs::read_to_string(path)
        .ok()
        .and_then(|source| target_ruby_version_from_source(&source))
        .unwrap_or_default()
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
}
