# frozen_string_literal: true

require "json"
require "open3"
require "rbconfig"
require "shellwords"
require "time"
require_relative "support/benchmark"

extend BenchmarkSupport

root = File.expand_path("..", __dir__)
output_root = performance_output_root(root)
cops, paths = compatibility_corpus(root)
custom_cop_name = "Custom/SyntheticFileHeader"
custom_cop = File.join(root, "benchmark/custom_cops/synthetic_file_header.rb")
config = File.join(root, "benchmark/custom-cop-rubocop.yml")
rustocop = [File.join(root, "libexec/rustocop-native")]
rustocop_entrypoint = [RbConfig.ruby, File.join(root, "exe/rustocop")]
rubocop = [RbConfig.ruby, Gem.bin_path("rubocop", "rubocop")]

native_options = ["--no-parallel", "--format", "json", "--only", cops.join(",")]
custom_options = ["--require", custom_cop, "--config", config]
rubocop_options = ["--cache", "false", "--no-server", "--format", "json", *custom_options]
commands = {
  "native_binary" => [*rustocop, *native_options, *paths],
  "native_entrypoint" => [*rustocop_entrypoint, *native_options, *paths],
  "mixed" => [*rustocop, "--no-parallel", "--format", "json", *custom_options,
              "--only", [*cops, custom_cop_name].join(","), *paths],
  "mixed_entrypoint" => [*rustocop_entrypoint, "--no-parallel", "--format", "json", *custom_options,
                         "--only", [*cops, custom_cop_name].join(","), *paths],
  "rubocop_custom_only" => [*rubocop, *rubocop_options, "--only", custom_cop_name, *paths],
  "rubocop_all" => [*rubocop, *rubocop_options, "--only", [*cops, custom_cop_name].join(","), *paths]
}

def command_report(command)
  stdout, stderr, status = Open3.capture3(*command)
  raise "command failed (#{status.exitstatus}): #{command.shelljoin}\n#{stderr}" unless [0, 1].include?(status.exitstatus)
  raise "command wrote to stderr: #{command.shelljoin}\n#{stderr}" unless stderr.empty?

  JSON.parse(stdout)
end

def normalized(report)
  report = Marshal.load(Marshal.dump(report))
  report.fetch("metadata").transform_values! { "normalized" }
  report.fetch("files").each { |file| file["path"] = File.basename(file.fetch("path")) }
  report
end

reports = commands.transform_values { |command| normalized(command_report(command)) }
raise "mixed output differs from pure RuboCop" unless reports.fetch("mixed") == reports.fetch("rubocop_all")
raise "entrypoint mixed output differs" unless reports.fetch("mixed_entrypoint") == reports.fetch("rubocop_all")
raise "native entrypoint output differs" unless reports.fetch("native_entrypoint") == reports.fetch("native_binary")
custom_offenses = reports.fetch("rubocop_custom_only").dig("summary", "offense_count")
raise "expected one custom offense per file, got #{custom_offenses}" unless custom_offenses == paths.length

warmups = 2
runs = 7
warmups.times { |iteration| commands.keys.rotate(iteration % commands.length).each { |name| duration(commands.fetch(name)) } }
samples = commands.to_h { |name, _command| [name, []] }
runs.times do |iteration|
  commands.keys.rotate(iteration % commands.length).each do |name|
    samples.fetch(name) << duration(commands.fetch(name))
  end
end

measurements = samples.to_h do |name, values|
  [name, {
    "median_seconds" => percentile(values, 0.5),
    "p95_seconds" => percentile(values, 0.95),
    "samples_seconds" => values
  }]
end
report = {
  "generated_at" => Time.now.iso8601,
  "scope" => {
    "files" => paths.length,
    "bytes" => paths.sum { |path| File.size(path) },
    "native_cops" => cops.length,
    "custom_cops" => 1,
    "custom_offenses" => custom_offenses,
    "runs" => runs,
    "warmups" => warmups,
    "cache" => false,
    "server" => false,
    "parser_engine" => "parser_prism",
    "mixed_matches_rubocop" => true
  },
  "measurements" => measurements
}
json_path = File.join(output_root, "mixed-custom-cop-benchmark.json")
File.write(json_path, JSON.pretty_generate(report))

puts "variant\tmedian_ms\tp95_ms"
measurements.each do |name, measurement|
  puts format("%s\t%.3f\t%.3f", name, measurement.fetch("median_seconds") * 1000,
              measurement.fetch("p95_seconds") * 1000)
end
puts "Report: #{json_path}"
