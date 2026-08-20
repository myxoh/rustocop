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
    ruby "script/report_compatibility_drift.rb", report,
         "--output", "tmp/compatibility-promotion-drift.md"
  end

  desc "Reject unclassified heuristic cops in trusted test surfaces"
  task :test_contracts do
    ruby "script/check_test_cop_classifications.rb"
    ruby "script/check_hardening_contracts.rb"
    ruby "script/generate_qualification_progress.rb", "--check"
    ruby "script/generate_source_cop_inventory.rb", "--check"
    ruby "script/generate_compatibility_corpus.rb", "--check"
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

task spec: ["build:native", "quality:architecture", "quality:test_contracts"]

task default: :spec
