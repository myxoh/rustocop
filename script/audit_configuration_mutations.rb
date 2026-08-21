# frozen_string_literal: true

require "json"
require "rbconfig"

ROOT = File.expand_path("..", __dir__)
AUDIT = File.join(ROOT, "script", "audit_project_parity.rb")
OUTPUT = File.join(ROOT, "tmp", "project-parity")

profiles = {
  "strict" => {
    "config" => "benchmark/project-rubocop-mutation-strict.yml",
    "cops" => %w[
      Metrics/CollectionLiteralLength
      Style/MutableConstant
      Style/InverseMethods
      Style/UnlessLogicalOperators
      Style/NegatedUnless
    ]
  },
  "policy" => {
    "config" => "benchmark/project-rubocop-mutation-policy.yml",
    "cops" => %w[
      Style/InverseMethods
      Style/NegatedUnless
    ]
  }
}.freeze

profiles.each do |name, profile|
  report = File.join(OUTPUT, "config-mutation-#{name}.json")
  markdown = report.sub(/\.json\z/, ".md")
  command = [
    RbConfig.ruby,
    AUDIT,
    "--config", File.join(ROOT, profile.fetch("config")),
    "--cops", profile.fetch("cops").join(","),
    "--report", report,
    "--markdown", markdown
  ]
  abort "configuration mutation audit failed to run: #{name}" unless system(*command, chdir: ROOT)

  results = JSON.parse(File.read(report)).fetch("combined_by_cop")
  failures = results.filter_map do |cop, result|
    classification = result.fetch("classification")
    "#{cop}=#{classification}" unless %w[project_exact dormant].include?(classification)
  end
  abort "configuration mutation parity failed (#{name}): #{failures.join(', ')}" unless failures.empty?
end

puts "Configuration mutation parity passed: #{profiles.length} profiles."
