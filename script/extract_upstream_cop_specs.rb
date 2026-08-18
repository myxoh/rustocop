# frozen_string_literal: true

require "fileutils"
require "json"
require "open3"
require "rbconfig"
require "rubocop"

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

counts = Hash.new(0)
File.foreach(output) { |line| counts[JSON.parse(line).fetch("cop")] += 1 }
registered_cops = RuboCop::Cop::Registry.global.map(&:cop_name)
missing_cops = registered_cops - counts.keys
abort "capture has no executable cases for: #{missing_cops.sort.join(", ")}" unless missing_cops.empty?

puts "Captured #{counts.values.sum} executable cases for #{counts.length} cops in #{output}"
