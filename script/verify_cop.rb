# frozen_string_literal: true

require "fileutils"
require "json"
require "rbconfig"

root = File.expand_path("..", __dir__)
cop_name = ARGV.fetch(0) { abort "Usage: ruby script/verify_cop.rb Department/CopName" }
manifest = File.join(root, "crates/rustocop/Cargo.toml")
native = File.join(root, "crates/rustocop/target/debug/rustocop")
comparison = File.join(root, "script/compare_upstream_cop_specs.rb")
support_generator = File.join(root, "script/generate_cop_support.rb")
report = File.join(root, "tmp/verify-#{cop_name.tr('/', '_').downcase}.json")

build = system("cargo", "build", "--manifest-path", manifest)
abort "Rust build failed" unless build

environment = { "RUSTOCOP_NATIVE_PATH" => native }
command = [
  RbConfig.ruby, comparison,
  "--only", cop_name,
  "--corrections",
  "--report", report
]
success = system(environment, *command)

if success
  docs_updated = system(environment, RbConfig.ruby, support_generator)
  abort "support documentation generation failed" unless docs_updated
  puts "#{cop_name} passes every captured diagnostic and correction case."
  puts "Run the full upstream comparison before promoting it to Verified."
else
  if File.file?(report)
    results = JSON.parse(File.read(report)).fetch("results", {})
    failure = results.values.filter_map { |result| result["first_failure"] }.first
    if failure
      example = failure.fetch("example", {})
      warn "First failure: #{example["description"] || example["id"]}"
      warn "Expected: #{JSON.generate(failure["expected"])}"
      warn "Actual:   #{JSON.generate(failure["actual"])}"
    end
  end
  warn "#{cop_name} still has compatibility failures; inspect #{report}."
end
exit(success ? 0 : 1)
