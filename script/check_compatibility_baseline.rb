# frozen_string_literal: true

require "json"
require "yaml"
require_relative "../lib/rustocop/compatibility_baseline"
require_relative "../lib/rustocop/compatibility_status"

ROOT = File.expand_path("..", __dir__)
report_path = ARGV.shift or abort "usage: check_compatibility_baseline.rb REPORT [STATUS]"
status_path = ARGV.shift
status = if status_path
           YAML.safe_load_file(status_path)
         else
           Rustocop::CompatibilityStatus.load(root: ROOT).data
         end

errors = Rustocop::CompatibilityBaseline.errors(
  JSON.parse(File.read(report_path)),
  status
)

if errors.empty?
  puts "Compatibility baseline preserved."
else
  warn "Compatibility baseline regression:"
  errors.each { |error| warn "  - #{error}" }
  exit 1
end
