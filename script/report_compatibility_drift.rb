# frozen_string_literal: true

require "fileutils"
require "json"
require "optparse"
require_relative "../lib/rustocop/compatibility_drift"
require_relative "../lib/rustocop/compatibility_status"

ROOT = File.expand_path("..", __dir__)
options = {
  corpus: File.join(ROOT, "tmp/rubocop-1.87.0-cop-cases.jsonl"),
  output: nil
}
OptionParser.new do |parser|
  parser.banner = "usage: report_compatibility_drift.rb REPORT [options]"
  parser.on("--corpus PATH") { |path| options[:corpus] = File.expand_path(path) }
  parser.on("--output PATH") { |path| options[:output] = File.expand_path(path) }
end.parse!
report_path = ARGV.shift or abort "compatibility report is required"
abort "unexpected arguments: #{ARGV.join(" ")}" unless ARGV.empty?
abort "compatibility corpus not found: #{options[:corpus]}" unless File.file?(options[:corpus])

status = Rustocop::CompatibilityStatus.load(root: ROOT)
report = JSON.parse(File.read(report_path))
contracts = Rustocop::CompatibilityDrift.correction_contracts(options.fetch(:corpus))
drift = Rustocop::CompatibilityDrift.analyze(report, status, correction_contracts: contracts)

section = lambda do |title, key|
  cops = drift.fetch(key)
  body = cops.empty? ? "- None" : cops.map { |cop| "- `#{cop}`" }.join("\n")
  "## #{title}\n\n#{body}"
end
document = <<~MARKDOWN
  # Compatibility promotion drift

  Generated from `#{File.basename(report_path)}` against RuboCop #{status.version}.

  #{section.call("Passing but not promoted", "passing_not_promoted")}

  #{section.call("Promoted but regressed", "verified_regressions")}

  #{section.call("Passing cops with correctable cases but no correction assertions", "passing_without_correction_assertions")}
MARKDOWN

if options[:output]
  FileUtils.mkdir_p(File.dirname(options[:output]))
  File.write(options[:output], document)
  puts "wrote #{options[:output]}"
else
  puts document
end
