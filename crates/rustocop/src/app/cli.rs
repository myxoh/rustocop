use std::env;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::config::{
    AutocorrectMode, CopConfig, CopSelection, InspectionConfig, Parallelism, RubyVersion,
    RunOptions, SourceEncoding,
};

pub(super) enum Command {
    Run(Box<RunOptions>),
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
        include_non_native_cops: false,
        non_native_cops: Vec::new(),
        force_exclusion: false,
        correction_loop: true,
        inspection: InspectionConfig {
            autocorrect: AutocorrectMode::None,
            ignore_disable_comments: false,
            cops: CopSelection::default_enabled(),
            target_ruby_version: RubyVersion::default(),
            source_encoding: SourceEncoding::Utf8,
            cop_config: Arc::new(CopConfig::default()),
            inspected_path: None,
            registry_context: None,
        },
    };

    while !args.is_empty() {
        let arg = args.remove(0);
        match arg.as_str() {
            "--version" => return Ok(Command::Version),
            "-V" => return Ok(Command::VerboseVersion),
            "--show-cops" => return Ok(Command::ShowCops),
            "-a" | "--autocorrect" | "--auto-correct" => {
                options.inspection.autocorrect = AutocorrectMode::Safe;
            }
            "-A" | "--autocorrect-all" | "--auto-correct-all" => {
                options.inspection.autocorrect = AutocorrectMode::All;
            }
            "--format" | "-f" => options.format = take_value(&mut args, &arg)?,
            "--only" => {
                options
                    .inspection
                    .cops
                    .select_only(&take_value(&mut args, &arg)?);
            }
            "--except" => options
                .inspection
                .cops
                .except(&take_value(&mut args, &arg)?),
            "--stdin" => options.stdin_path = Some(take_value(&mut args, &arg)?),
            "--parallel" => options.parallelism = Parallelism::Automatic,
            "--no-parallel" => options.parallelism = Parallelism::Sequential,
            "--jobs" => {
                options.parallelism =
                    Parallelism::Fixed(parse_jobs(&take_value(&mut args, &arg)?)?);
            }
            "--config" | "-c" => {
                let path = take_value(&mut args, &arg)?;
                apply_config(&mut options, &path)?;
                options.config_path = Some(path);
            }
            "--require" | "--plugin" => {
                let value = take_value(&mut args, &arg)?;
                options.rubocop_loaders.push((arg, value));
            }
            "--included-non-native-cops" => options.include_non_native_cops = true,
            "--resolved-enabled-cops" => {
                options
                    .inspection
                    .cops
                    .select_only(&take_value(&mut args, &arg)?);
            }
            "--registry-context" => {
                options.inspection.registry_context = Some(Arc::new(
                    cop_list(&take_value(&mut args, &arg)?)
                        .into_iter()
                        .collect(),
                ));
            }
            "--resolved-non-native-cops" => {
                options.non_native_cops = cop_list(&take_value(&mut args, &arg)?);
            }
            "--force-exclusion" => options.force_exclusion = true,
            "--no-correction-loop" => options.correction_loop = false,
            "--no-server" | "--display-cop-names" | "--extra-details" => {}
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
                options
                    .inspection
                    .cops
                    .select_only(arg.strip_prefix("--only=").unwrap_or_default());
            }
            _ if arg.starts_with("--except=") => options
                .inspection
                .cops
                .except(arg.strip_prefix("--except=").unwrap_or_default()),
            _ if arg.starts_with("--stdin=") => {
                options.stdin_path =
                    Some(arg.strip_prefix("--stdin=").unwrap_or_default().to_string());
            }
            _ if arg.starts_with("--config=") => {
                let path = arg.strip_prefix("--config=").unwrap_or_default();
                apply_config(&mut options, path)?;
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
            _ if arg.starts_with("--resolved-enabled-cops=") => {
                options.inspection.cops.select_only(
                    arg.strip_prefix("--resolved-enabled-cops=")
                        .unwrap_or_default(),
                );
            }
            _ if arg.starts_with("--registry-context=") => {
                options.inspection.registry_context = Some(Arc::new(
                    cop_list(arg.strip_prefix("--registry-context=").unwrap_or_default())
                        .into_iter()
                        .collect(),
                ));
            }
            _ if arg.starts_with("--resolved-non-native-cops=") => {
                options.non_native_cops = cop_list(
                    arg.strip_prefix("--resolved-non-native-cops=")
                        .unwrap_or_default(),
                );
            }
            _ if arg.starts_with('-') => return Err(format!("unsupported option {arg}")),
            _ => options.files.push(arg),
        }
    }

    if options.config_path.is_none() {
        if let Some(path) = discover_config_path(&options.files, options.stdin_path.as_deref()) {
            let path = path.to_string_lossy().to_string();
            apply_config(&mut options, &path)?;
            options.config_path = Some(path);
        }
    }

    if let Ok(source) = env::var("RUSTOCOP_RESOLVED_CONFIG_SOURCE") {
        apply_config_source(
            &mut options.inspection,
            &source,
            options.config_path.as_deref(),
        );
    }

    if options.format != "json" && options.format != "simple" {
        return Err(format!("unsupported formatter {}", options.format));
    }
    Ok(Command::Run(Box::new(options)))
}

fn cop_list(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .map(str::to_string)
        .collect()
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

fn apply_config(options: &mut RunOptions, path: &str) -> Result<(), String> {
    let cop_config = CopConfig::from_path(path)?;
    options.inspection.target_ruby_version = cop_config
        .value("AllCops", "TargetRubyVersion")
        .and_then(RubyVersion::parse)
        .unwrap_or_default();
    options.non_native_cops = cop_config.non_native_cops().to_vec();
    options.inspection.cop_config = Arc::new(cop_config);
    Ok(())
}

fn discover_config_path(files: &[String], stdin_path: Option<&str>) -> Option<PathBuf> {
    let current = env::current_dir().ok()?;
    let target = files.first().map(String::as_str).or(stdin_path);
    let mut directory = target.map_or_else(
        || current.clone(),
        |target| {
            let path = Path::new(target);
            let path = if path.is_absolute() {
                path.to_path_buf()
            } else {
                current.join(path)
            };
            if path.is_dir() {
                path
            } else {
                path.parent().unwrap_or(&current).to_path_buf()
            }
        },
    );
    const CANDIDATES: &[&str] = &[
        ".rustocop.yml",
        ".config/rustocop/config.yml",
        "rustocop.yml",
        ".rubocop.yml",
        ".config/rubocop/config.yml",
    ];
    loop {
        for candidate in CANDIDATES {
            let path = directory.join(candidate);
            if path.is_file() {
                return Some(path);
            }
        }
        if !directory.pop() {
            break;
        }
    }
    None
}

fn apply_config_source(config: &mut InspectionConfig, source: &str, config_path: Option<&str>) {
    let cop_config = CopConfig::from_resolved_source(source, config_path);
    config.target_ruby_version = cop_config
        .value("AllCops", "TargetRubyVersion")
        .and_then(RubyVersion::parse)
        .unwrap_or_default();
    config.cop_config = Arc::new(cop_config);
}

#[cfg(test)]
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
    fn distinguishes_safe_and_all_autocorrection() {
        let Command::Run(safe) = parse_args(vec!["-a".to_string()]).unwrap() else {
            panic!("expected run command");
        };
        let Command::Run(all) = parse_args(vec!["-A".to_string()]).unwrap() else {
            panic!("expected run command");
        };
        assert_eq!(safe.inspection.autocorrect, AutocorrectMode::Safe);
        assert_eq!(all.inspection.autocorrect, AutocorrectMode::All);
    }

    #[test]
    fn can_limit_autocorrection_to_one_pass_for_spec_compatibility() {
        let command = parse_args(vec!["-A".into(), "--no-correction-loop".into()]).unwrap();
        let Command::Run(options) = command else {
            panic!("expected run command");
        };

        assert!(!options.correction_loop);
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
    fn accepts_resolved_cop_sets_and_non_native_opt_in() {
        let Command::Run(options) = parse_args(vec![
            "--included-non-native-cops".to_string(),
            "--resolved-enabled-cops=Layout/LineLength,Style/StringLiterals".to_string(),
            "--resolved-non-native-cops=RSpec/Focus,Custom/Example".to_string(),
        ])
        .unwrap() else {
            panic!("expected run command");
        };

        assert!(options.include_non_native_cops);
        assert_eq!(
            options.non_native_cops,
            ["RSpec/Focus".to_string(), "Custom/Example".to_string()]
        );
        assert!(options.inspection.cop_enabled("Layout/LineLength"));
        assert!(!options.inspection.cop_enabled("Lint/Debugger"));
    }

    #[test]
    fn accepts_a_separate_registry_context_for_focused_audits() {
        let Command::Run(options) = parse_args(vec![
            "--only=Lint/MissingCopEnableDirective".to_string(),
            "--registry-context=Lint/MissingCopEnableDirective,Metrics/ClassLength".to_string(),
        ])
        .unwrap() else {
            panic!("expected run command");
        };

        assert!(options
            .inspection
            .cop_enabled("Lint/MissingCopEnableDirective"));
        assert!(!options.inspection.cop_enabled("Metrics/ClassLength"));
        assert!(options
            .inspection
            .registry_cop_enabled("Metrics/ClassLength"));
    }

    #[test]
    fn parses_except_and_force_exclusion() {
        let Command::Run(options) = parse_args(vec![
            "--except=Style,Layout/LineLength".to_string(),
            "--only=Style/StringLiterals,Layout/LineLength".to_string(),
            "--force-exclusion".to_string(),
        ])
        .unwrap() else {
            panic!("expected run command");
        };

        assert!(options.force_exclusion);
        assert!(!options.inspection.cop_enabled("Style/StringLiterals"));
        assert!(!options.inspection.cop_enabled("Layout/LineLength"));
    }

    #[test]
    fn rejects_an_unreadable_requested_config() {
        let missing = format!("{}/missing-rubocop.yml", env!("CARGO_MANIFEST_DIR"));
        let error = parse_args(vec![format!("--config={missing}")])
            .err()
            .expect("missing config should fail");

        assert!(error.starts_with(&format!("could not read config {missing}:")));
    }

    #[test]
    fn discovers_compiled_config_before_rubocop_config() {
        let directory = std::env::temp_dir().join(format!(
            "rustocop-cli-config-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(directory.join("app")).unwrap();
        std::fs::write(
            directory.join(".rustocop.yml"),
            "Rustocop:\n  SchemaVersion: 1\n  BuiltInCops:\n    - Style/StringLiterals\n  NonNativeCops:\n    - RSpec/Focus\nAllCops:\n  DisabledByDefault: true\nStyle/StringLiterals:\n  Enabled: true\n",
        )
        .unwrap();
        std::fs::write(
            directory.join(".rubocop.yml"),
            "Style/StringLiterals:\n  Enabled: false\n",
        )
        .unwrap();
        let target = directory.join("app/example.rb");
        std::fs::write(&target, "'example'\n").unwrap();

        let Command::Run(options) = parse_args(vec![target.to_string_lossy().to_string()]).unwrap()
        else {
            panic!("expected run command");
        };

        assert!(options.inspection.cop_config.is_compiled());
        assert_eq!(options.non_native_cops, vec!["RSpec/Focus"]);
        assert!(options.inspection.cop_enabled("Style/StringLiterals"));
        assert!(!options.inspection.cop_enabled("RSpec/Focus"));
        assert_eq!(
            options.config_path.as_deref(),
            Some(directory.join(".rustocop.yml").to_string_lossy().as_ref())
        );
        std::fs::remove_dir_all(directory).unwrap();
    }
}
