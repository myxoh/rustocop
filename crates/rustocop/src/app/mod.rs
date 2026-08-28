use std::env;

mod cli;
mod mixed;
mod report;
mod targets;

pub(crate) fn run(args: Vec<String>) -> i32 {
    match cli::parse_args(args) {
        Ok(cli::Command::Version) => {
            println!("{}", report::rustocop_version());
            0
        }
        Ok(cli::Command::VerboseVersion) => {
            println!(
                "{} (rustocop native, RuboCop-compatible JSON formatter)",
                report::rustocop_version()
            );
            0
        }
        Ok(cli::Command::ShowCops) => {
            for cop in crate::cops::cop_names() {
                println!("{cop}");
            }
            0
        }
        Ok(cli::Command::Run(options)) => run_inspection(&options),
        Err(error) => fail(error),
    }
}

fn run_inspection(options: &crate::config::RunOptions) -> i32 {
    if options.inspection.cop_config.is_compiled()
        && !options.include_non_native_cops
        && !options.non_native_cops.is_empty()
        && options.inspection.cops.requested().is_none()
    {
        eprintln!(
            "Warning - non native cops are ignored by default, to include them use \
             --included-non-native-cops NOTE performance is severely degraded when using non native cops."
        );
    }
    if let Some(custom_cops) = mixed::custom_cops(options) {
        return mixed::run(options, &custom_cops);
    }
    match targets::inspect(options) {
        Ok(results) => match report::write(options, &results) {
            Ok(()) => report::exit_status(options, &results),
            Err(error) => {
                eprintln!("{error}");
                2
            }
        },
        Err(error) => fail(error),
    }
}

fn fail(error: impl std::fmt::Display) -> i32 {
    eprintln!("rustocop: {error}");
    2
}

pub(crate) fn args() -> Vec<String> {
    env::args().skip(1).collect()
}
