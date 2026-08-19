# frozen_string_literal: true

require "json"
require "open3"
require "rbconfig"
require "shellwords"
require "time"
require_relative "../lib/rustocop/benchmark_documentation"
require_relative "support/benchmark"

extend BenchmarkSupport

root = File.expand_path("..", __dir__)
output_root = performance_output_root(root)
cops, paths = benchmark_corpus(root)
prism_config_path = prism_config(output_root)

native = File.join(root, "libexec/rustocop-native")
rubocop = [RbConfig.ruby, Gem.bin_path("rubocop", "rubocop")]
common = ["--cache", "false", "--no-server", "--format", "json", "--only", cops.join(",")]

def command_result(command)
  stdout, stderr, status = Open3.capture3(*command)
  raise "command failed (#{status.exitstatus}): #{command.shelljoin}\n#{stderr}" unless [0, 1].include?(status.exitstatus)
  raise "command wrote to stderr: #{command.shelljoin}\n#{stderr}" unless stderr.empty?

  JSON.parse(stdout)
end

def normalize(report)
  report = Marshal.load(Marshal.dump(report))
  report.fetch("metadata").transform_values! { "normalized" }
  report.fetch("files").each { |file| file["path"] = File.basename(file.fetch("path")) }
  report
end

sizes = [1, 25, 100, 500]
runs = { 1 => 30, 25 => 20, 100 => 12, 500 => 7 }
warmups = { 1 => 3, 25 => 3, 100 => 2, 500 => 2 }
results = []

sizes.each do |size|
  selected = paths.first(size)
  commands = {
    "rustocop" => [native, *common, "--config", prism_config_path, *selected],
    "rubocop" => [*rubocop, *common, "--config", prism_config_path, *selected]
  }

  reports = commands.transform_values { |command| normalize(command_result(command)) }
  reports.each do |name, report|
    raise "#{name} output mismatch at #{size} files" unless report == reports.fetch("rustocop")
  end

  warmups.fetch(size).times do |iteration|
    commands.keys.rotate(iteration % commands.length).each do |name|
      duration(commands.fetch(name))
    end
  end

  samples = commands.to_h { |name, _command| [name, []] }
  runs.fetch(size).times do |iteration|
    commands.keys.rotate(iteration % commands.length).each do |name|
      samples.fetch(name) << duration(commands.fetch(name))
    end
  end

  medians = samples.transform_values { |values| percentile(values, 0.5) }
  measurement = lambda do |name|
    {
      "median_seconds" => medians.fetch(name),
      "mean_seconds" => samples.fetch(name).sum.fdiv(runs.fetch(size)),
      "p95_seconds" => percentile(samples.fetch(name), 0.95),
      "files_per_second" => size.fdiv(medians.fetch(name))
    }
  end
  results << {
    "files" => size,
    "runs" => runs.fetch(size),
    "warmups" => warmups.fetch(size),
    "verified_equal" => true,
    "rustocop" => measurement.call("rustocop"),
    "rubocop" => measurement.call("rubocop"),
    "speedup_vs_rubocop" => medians.fetch("rubocop").fdiv(medians.fetch("rustocop"))
  }
end

report = {
  "generated_at" => Time.now.iso8601,
  "scope" => {
    "rubocop_version" => "1.87.0",
    "prism_version" => "1.9.0",
    "ruby_version" => RUBY_VERSION,
    "target_ruby_version" => "3.4",
    "parser_engine" => "parser_prism",
    "cops" => cops,
    "corpus_files" => paths.length,
    "cache" => false,
    "server" => false,
    "formatter" => "json"
  },
  "results" => results
}

json_path = File.join(output_root, "rubocop-prism-benchmark.json")
File.write(json_path, JSON.pretty_generate(report))
Rustocop::BenchmarkDocumentation.update_rubocop_prism(root, report)

puts "files\trustocop_ms\trubocop_prism_ms\tspeedup\tverified"
results.each do |result|
  puts format(
    "%d\t%.3f\t%.3f\t%.2fx\t%s",
    result.fetch("files"),
    result.dig("rustocop", "median_seconds") * 1000,
    result.dig("rubocop", "median_seconds") * 1000,
    result.fetch("speedup_vs_rubocop"),
    result.fetch("verified_equal")
  )
end
puts "Report: #{json_path}"
