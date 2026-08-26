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

  desc "Validate generated compatibility test contracts"
  task :test_contracts do
    ruby "script/generate_source_cop_inventory.rb", "--check"
    ruby "script/check_fixture_ownership.rb"
    ruby "script/generate_unit_fixtures.rb", "--check"
  end

  desc "Compare configured cop mutations against ten pinned real projects"
  task :configuration_mutations do
    ruby "script/audit_configuration_mutations.rb"
  end
end

namespace :fixtures do
  desc "Run cached RuboCop unit contracts (set COP=Department/Name to focus)"
  task :unit do
    environment = {}
    environment["RUSTOCOP_UNIT_COP"] = ENV.fetch("COP") if ENV["COP"]
    profile = ENV["COP"] ? "fixture" : "release"
    sh environment,
       "cargo", "test", "--manifest-path", "crates/rustocop/Cargo.toml",
       "--profile", profile, "cached_unit_contracts_match", "--", "--ignored", "--nocapture"
  end

  desc "Audit sequential per-cop cached-contract timings (set REPORT=path for JSON)"
  task :benchmark do
    environment = { "RUSTOCOP_UNIT_BENCHMARK" => "1" }
    environment["RUSTOCOP_UNIT_REPORT"] = File.expand_path(ENV.fetch("REPORT")) if ENV["REPORT"]
    sh environment,
       "cargo", "test", "--manifest-path", "crates/rustocop/Cargo.toml",
       "--release", "cached_unit_contracts_match", "--", "--ignored", "--nocapture"
  end

  desc "Recapture RuboCop 1.87 specs and regenerate the committed unit cache"
  task :refresh_unit do
    ruby "script/extract_upstream_cop_specs.rb"
    ruby "script/generate_unit_fixtures.rb"
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
