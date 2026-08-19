use std::env;

mod cli;
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
    match targets::inspect(options) {
        Ok(results) => {
            report::write(options, &results);
            report::exit_status(options, &results)
        }
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
