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

abort "memory benchmark currently requires macOS /usr/bin/time -l" unless RbConfig::CONFIG.fetch("host_os").include?("darwin")

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

def normalized_report(command)
  stdout, stderr, status = Open3.capture3(*command)
  raise "command failed (#{status.exitstatus}): #{command.shelljoin}\n#{stderr}" unless [0, 1].include?(status.exitstatus)
  raise "command wrote to stderr: #{command.shelljoin}\n#{stderr}" unless stderr.empty?

  report = JSON.parse(stdout)
  report.fetch("metadata").transform_values! { "normalized" }
  report.fetch("files").each { |file| file["path"] = File.basename(file.fetch("path")) }
  report
end

def peak_rss_bytes(command)
  reader, writer = IO.pipe
  pid = Process.spawn("/usr/bin/time", "-l", *command, out: File::NULL, err: writer)
  writer.close
  accounting = reader.read
  reader.close
  _finished_pid, status = Process.wait2(pid)
  raise "benchmark command failed with #{status.exitstatus}: #{command.shelljoin}" unless [0, 1].include?(status.exitstatus)

  match = accounting.match(/^\s*(\d+)\s+maximum resident set size$/)
  raise "could not read peak RSS from /usr/bin/time output:\n#{accounting}" unless match

  Integer(match[1])
end

def percentile(values, fraction)
  sorted = values.sort
  sorted[((sorted.length - 1) * fraction).round]
end

sizes = [1, 25, 100, 500]
runs = 7
results = sizes.map do |size|
  selected = paths.first(size)
  commands = {
    "rustocop" => [native, *common, "--config", config_path, *selected],
    "rubocop_prism" => [*rubocop, *common, "--config", config_path, *selected]
  }

  raise "output mismatch at #{size} files" unless normalized_report(commands.fetch("rustocop")) ==
                                                normalized_report(commands.fetch("rubocop_prism"))

  commands.each_value { |command| peak_rss_bytes(command) }
  samples = { "rustocop" => [], "rubocop_prism" => [] }
  runs.times do |iteration|
    order = iteration.even? ? %w[rustocop rubocop_prism] : %w[rubocop_prism rustocop]
    order.each { |name| samples.fetch(name) << peak_rss_bytes(commands.fetch(name)) }
  end

  medians = samples.transform_values { |values| percentile(values, 0.5) }
  {
    "files" => size,
    "runs" => runs,
    "verified_equal" => true,
    "rustocop" => {
      "median_peak_rss_bytes" => medians.fetch("rustocop"),
      "p95_peak_rss_bytes" => percentile(samples.fetch("rustocop"), 0.95)
    },
    "rubocop_prism" => {
      "median_peak_rss_bytes" => medians.fetch("rubocop_prism"),
      "p95_peak_rss_bytes" => percentile(samples.fetch("rubocop_prism"), 0.95)
    },
    "rss_ratio" => medians.fetch("rubocop_prism").fdiv(medians.fetch("rustocop"))
  }
end

report = {
  "generated_at" => Time.now.iso8601,
  "scope" => {
    "rubocop_version" => "1.87.0",
    "ruby_version" => RUBY_VERSION,
    "target_ruby_version" => "3.4",
    "parser_engine" => "parser_prism",
    "cops" => cops,
    "corpus_files" => paths.length,
    "cache" => false,
    "server" => false,
    "formatter" => "json",
    "process_model" => "single process, sequential files",
    "measurement" => "macOS /usr/bin/time -l maximum resident set size"
  },
  "results" => results
}

json_path = File.join(output_root, "memory-benchmark.json")
File.write(json_path, JSON.pretty_generate(report))

puts "files\trustocop_mib\trubocop_prism_mib\tratio\tverified"
results.each do |result|
  puts format(
    "%d\t%.2f\t%.2f\t%.2fx\t%s",
    result.fetch("files"),
    result.dig("rustocop", "median_peak_rss_bytes").fdiv(1024**2),
    result.dig("rubocop_prism", "median_peak_rss_bytes").fdiv(1024**2),
    result.fetch("rss_ratio"),
    result.fetch("verified_equal")
  )
end
puts "Report: #{json_path}"
