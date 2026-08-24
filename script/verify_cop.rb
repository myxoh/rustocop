# frozen_string_literal: true

require "fileutils"
require "json"
require "optparse"
require "rbconfig"

root = File.expand_path("..", __dir__)
options = { live_rubocop: false }
OptionParser.new do |parser|
  parser.banner = "Usage: ruby script/verify_cop.rb [--live-rubocop] Department/CopName [...]"
  parser.on("--live-rubocop", "also rerun the slow RuboCop and local differential layers") do
    options[:live_rubocop] = true
  end
end.parse!
cop_names = ARGV
abort "Pass at least one Department/CopName" if cop_names.empty?
manifest = File.join(root, "crates/rustocop/Cargo.toml")

cached_success = system(
  { "RUSTOCOP_UNIT_COP" => cop_names.join(",") },
  "cargo", "test", "--manifest-path", manifest, "--release",
  "cached_unit_contracts_match", "--", "--ignored", "--nocapture"
)
exit(cached_success ? 0 : 1) unless options[:live_rubocop]

native = File.join(root, "crates/rustocop/target/debug/rustocop")
comparison = File.join(root, "script/compare_upstream_cop_specs.rb")
report = File.join(root, "tmp/verify-#{cop_names.join('-').tr('/', '_').downcase}.json")

build = system("cargo", "build", "--manifest-path", manifest)
abort "Rust build failed" unless build

environment = { "RUSTOCOP_NATIVE_PATH" => native }
command = [
  RbConfig.ruby, comparison,
  "--only", cop_names.join(","),
  "--corrections",
  "--report", report
]
success = cached_success && system(environment, *command)

if success
  subject = cop_names.length == 1 ? "#{cop_names.first} passes" : "#{cop_names.join(', ')} pass"
  puts "#{subject} every cached unit contract and freshly captured upstream case."
  puts "Run the 50-project parity audit before calling it project-exact."
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
  warn "#{cop_names.join(', ')} still have compatibility failures; inspect #{report}."
end
exit(success ? 0 : 1)
