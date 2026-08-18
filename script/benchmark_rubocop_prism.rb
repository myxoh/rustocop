# frozen_string_literal: true

require "fileutils"
require "json"
require "open3"
require "rbconfig"
require "shellwords"
require "time"

root = File.expand_path("..", __dir__)
fixture_root = File.join(root, "spec/fixtures/rubocop_builtin_examples")
output_root = File.join(root, "tmp/performance-verification")
FileUtils.mkdir_p(output_root)

manifest = File.readlines(File.join(fixture_root, "manifest.tsv"), chomp: true).drop(1).map do |line|
  directory, cop = line.split("\t", 2)
  [cop, Dir[File.join(fixture_root, directory, "*.rb")].sort]
end
cops = manifest.map(&:first)
paths = manifest.map(&:last).then do |groups|
  (0...groups.map(&:length).max).flat_map { |index| groups.filter_map { |group| group[index] } }
end
raise "expected 500 files" unless paths.length == 500

config_path = File.join(output_root, "rubocop-prism.yml")
File.write(config_path, <<~YAML)
  AllCops:
    ParserEngine: parser_prism
    TargetRubyVersion: 3.4
    NewCops: enable
YAML

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

def duration(command)
  started = Process.clock_gettime(Process::CLOCK_MONOTONIC)
  _pid = Process.spawn(*command, out: File::NULL, err: File::NULL)
  _finished_pid, status = Process.wait2(_pid)
  raise "benchmark command failed with #{status.exitstatus}: #{command.shelljoin}" unless [0, 1].include?(status.exitstatus)

  Process.clock_gettime(Process::CLOCK_MONOTONIC) - started
end

def percentile(values, fraction)
  sorted = values.sort
  sorted[((sorted.length - 1) * fraction).round]
end

sizes = [1, 25, 100, 500]
runs = { 1 => 30, 25 => 20, 100 => 12, 500 => 7 }
warmups = { 1 => 3, 25 => 3, 100 => 2, 500 => 2 }
results = []

sizes.each do |size|
  selected = paths.first(size)
  rust_command = [native, *common, "--config", config_path, *selected]
  rubocop_command = [*rubocop, *common, "--config", config_path, *selected]

  rust_report = normalize(command_result(rust_command))
  rubocop_report = normalize(command_result(rubocop_command))
  raise "output mismatch at #{size} files" unless rust_report == rubocop_report

  warmups.fetch(size).times do |iteration|
    commands = iteration.even? ? [rust_command, rubocop_command] : [rubocop_command, rust_command]
    commands.each { |command| duration(command) }
  end

  samples = { "rustocop" => [], "rubocop_prism" => [] }
  runs.fetch(size).times do |iteration|
    order = iteration.even? ? %w[rustocop rubocop_prism] : %w[rubocop_prism rustocop]
    order.each do |name|
      command = name == "rustocop" ? rust_command : rubocop_command
      samples.fetch(name) << duration(command)
    end
  end

  medians = samples.transform_values { |values| percentile(values, 0.5) }
  results << {
    "files" => size,
    "runs" => runs.fetch(size),
    "warmups" => warmups.fetch(size),
    "verified_equal" => true,
    "rustocop" => {
      "median_seconds" => medians.fetch("rustocop"),
      "mean_seconds" => samples.fetch("rustocop").sum.fdiv(runs.fetch(size)),
      "p95_seconds" => percentile(samples.fetch("rustocop"), 0.95),
      "files_per_second" => size.fdiv(medians.fetch("rustocop"))
    },
    "rubocop_prism" => {
      "median_seconds" => medians.fetch("rubocop_prism"),
      "mean_seconds" => samples.fetch("rubocop_prism").sum.fdiv(runs.fetch(size)),
      "p95_seconds" => percentile(samples.fetch("rubocop_prism"), 0.95),
      "files_per_second" => size.fdiv(medians.fetch("rubocop_prism"))
    },
    "speedup" => medians.fetch("rubocop_prism").fdiv(medians.fetch("rustocop"))
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

puts "files\trustocop_ms\trubocop_prism_ms\tspeedup\tverified"
results.each do |result|
  puts format(
    "%d\t%.3f\t%.3f\t%.2fx\t%s",
    result.fetch("files"),
    result.dig("rustocop", "median_seconds") * 1000,
    result.dig("rubocop_prism", "median_seconds") * 1000,
    result.fetch("speedup"),
    result.fetch("verified_equal")
  )
end
puts "Report: #{json_path}"
