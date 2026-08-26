# frozen_string_literal: true

require "fileutils"
require "json"
require "open3"
require "rbconfig"
require "rubocop"
require_relative "../lib/rustocop/compatibility_status"

root = File.expand_path("..", __dir__)
output = File.expand_path(ARGV.fetch(0, "tmp/rubocop-1.87.0-cop-cases.jsonl"), root)
FileUtils.mkdir_p(File.dirname(output))
File.write(output, "")

departments = %w[migration bundler gemspec layout lint metrics naming style security]
spec_root = File.join(root, "spec/upstream/rubocop-1.87.0/spec/rubocop/cop")
targets = departments.map { |department| File.join(spec_root, department) }
helper = File.join(root, "spec/support/upstream_rubocop_spec_helper.rb")
command = [
  RbConfig.ruby, Gem.bin_path("rspec-core", "rspec"),
  "-r", helper, *targets, "--format", "progress"
]

env = { "RUSTOCOP_UPSTREAM_CAPTURE" => output }
status = system(env, *command)
abort "upstream RuboCop specs failed; capture is incomplete" unless status

compatibility_status = Rustocop::CompatibilityStatus.load(root: root)
pending = compatibility_status.intentionally_pending_cops.to_h { |cop| [cop, true] }
broken_cases = YAML.safe_load_file(
  File.join(root, "spec/upstream/rubocop-1.87.0/broken_fixture_cases.yml")
).fetch("cases").to_h { |entry| [entry.fetch("id"), true] }
retained = File.foreach(output).filter_map do |line|
  test_case = JSON.parse(line)
  line unless pending[test_case.fetch("cop")] || broken_cases[test_case.dig("example", "id")]
end
File.write(output, retained.join)

counts = Hash.new(0)
File.foreach(output) { |line| counts[JSON.parse(line).fetch("cop")] += 1 }
registered_cops = RuboCop::Cop::Registry.global.map(&:cop_name) - pending.keys
missing_cops = registered_cops - counts.keys
abort "capture has no executable cases for: #{missing_cops.sort.join(", ")}" unless missing_cops.empty?

puts "Captured #{counts.values.sum} executable cases for #{counts.length} active cops in #{output}"
puts "Excluded #{pending.length} intentionally-pending cops"
