# frozen_string_literal: true

require "json"
require "yaml"
require_relative "../lib/rustocop/compatibility_baseline"

ROOT = File.expand_path("..", __dir__)
report_path = ARGV.shift or abort "usage: check_compatibility_baseline.rb REPORT [STATUS]"
status_path = ARGV.shift || File.join(ROOT, "spec/upstream/rubocop-1.87.0/status.yml")

errors = Rustocop::CompatibilityBaseline.errors(
  JSON.parse(File.read(report_path)),
  YAML.safe_load(File.read(status_path))
)

if errors.empty?
  puts "Compatibility baseline preserved."
else
  warn "Compatibility baseline regression:"
  errors.each { |error| warn "  - #{error}" }
  exit 1
end
