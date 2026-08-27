# frozen_string_literal: true

require_relative "lib/rustocop/version"

Gem::Specification.new do |spec|
  spec.name = "rustocop"
  spec.version = Rustocop::VERSION
  spec.authors = ["rustocop contributors"]
  spec.email = ["contributors@example.com"]

  spec.summary = "A fast, unfinished local RuboCop substitute backed by Rust."
  spec.description = "Vibe-coded local linting for faster feedback, with real RuboCop still expected to run in CI."
  spec.homepage = "https://github.com/myxoh/rustocop"
  spec.license = "MIT"
  spec.required_ruby_version = ">= 3.1"

  spec.metadata = {
    "allowed_push_host" => "https://rubygems.org",
    "rubygems_mfa_required" => "true",
    "source_code_uri" => spec.homepage
  }

  spec.files = Dir[
    "LICENSE.txt",
    "README.md",
    "Rakefile",
    "crates/rustocop/Cargo.toml",
    "crates/rustocop/Cargo.lock",
    "crates/rustocop/src/**/*",
    "docs/**/*",
    "exe/*",
    "lib/**/*",
    "libexec/*"
  ]
  spec.bindir = "exe"
  spec.executables = ["rustocop", "rustocop-config"]
  spec.require_paths = ["lib"]

  spec.add_development_dependency "rake", "~> 13.0"
  spec.add_development_dependency "rspec", "~> 3.13"
  spec.add_development_dependency "rubocop", ">= 1.84", "< 2.0"
end
