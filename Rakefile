# frozen_string_literal: true

require "fileutils"
require "rake"
require "rbconfig"
require "rspec/core/rake_task"

RSpec::Core::RakeTask.new(:spec)

namespace :quality do
  desc "Enforce module size and Rust complexity ceilings"
  task :architecture do
    ruby "script/check_architecture.rb"
    sh "cargo", "clippy", "--manifest-path", "crates/rustocop/Cargo.toml",
       "--all-targets", "--", "-D", "warnings"
  end

  desc "Reject regressions in a completed full upstream compatibility report"
  task :compatibility_baseline do
    report = ENV.fetch("REPORT", "tmp/rubocop-1.87.0-compatibility.json")
    ruby "script/check_compatibility_baseline.rb", report
  end
end

namespace :build do
  desc "Build the Rust native binary into libexec/rustocop-native"
  task :native do
    cargo = ENV.fetch("CARGO", "cargo")
    manifest = "crates/rustocop/Cargo.toml"
    extension = RbConfig::CONFIG.fetch("host_os").match?(/mswin|mingw|cygwin/) ? ".exe" : ""
    source = "crates/rustocop/target/release/rustocop#{extension}"
    destination = "libexec/rustocop-native#{extension}"

    sh cargo, "build", "--release", "--manifest-path", manifest
    FileUtils.mkdir_p("libexec")
    FileUtils.cp(source, destination)
    FileUtils.chmod(0o755, destination)
  end
end

task spec: ["build:native", "quality:architecture"]

task default: :spec
