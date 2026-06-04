# frozen_string_literal: true

require "fileutils"
require "rake"
require "rbconfig"
require "rspec/core/rake_task"

RSpec::Core::RakeTask.new(:spec)

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

task spec: "build:native"

task default: :spec
