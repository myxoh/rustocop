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

prism_config_path = File.join(output_root, "rubocop-prism.yml")
File.write(prism_config_path, <<~YAML)
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
