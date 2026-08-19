# frozen_string_literal: true

require "fileutils"
require "json"
require "open-uri"
require "open3"
require "pathname"
require "rbconfig"
require "shellwords"
require "tempfile"
require "time"
require_relative "support/benchmark"

extend BenchmarkSupport

ROOT = File.expand_path("..", __dir__)
CACHE_ROOT = File.join(ROOT, "tmp/project-benchmarks")
REPORT_PATH = File.join(CACHE_ROOT, "project-benchmarks.json")
CONFIG_PATH = File.join(ROOT, "benchmark/project-rubocop.yml")
NATIVE = File.join(ROOT, "libexec/rustocop-native")
RUBOCOP = [RbConfig.ruby, Gem.bin_path("rubocop", "rubocop")].freeze
RUNS = Integer(ENV.fetch("PROJECT_BENCHMARK_RUNS", "5"))
WARMUPS = Integer(ENV.fetch("PROJECT_BENCHMARK_WARMUPS", "1"))
COPS = %w[
  Layout/LineLength
  Metrics/AbcSize
  Metrics/MethodLength
  Style/Documentation
  Style/HashSyntax
  Style/RedundantReturn
  Style/Semicolon
  Style/StringLiterals
].freeze
PROJECTS = [
  {
    "name" => "chatwoot",
    "repository" => "chatwoot/chatwoot",
    "revision" => "8d93d69e8e356216e85c28de7c4240e66b8e83fa",
    "license" => "MIT outside enterprise/"
  },
  {
    "name" => "rubygems.org",
    "repository" => "rubygems/rubygems.org",
    "revision" => "3201f8831866f82eb9acd7f66287a978d0e59079",
    "license" => "MIT"
  },
  {
    "name" => "gitlab-ce",
    "repository" => "gitlabhq/gitlabhq",
    "revision" => "67a526442c20d20b6e80ebf916bd766b54018c5e",
    "license" => "MIT Community Edition"
  }
].freeze
EXCLUDED_COMPONENTS = %w[
  .git
  coverage
  ee
  enterprise
  log
  node_modules
  public
  tmp
  vendor
].freeze
EXCLUDED_FILES = %w[db/schema.rb].freeze

FileUtils.mkdir_p(CACHE_ROOT)

def download_archive(project)
  archive = File.join(CACHE_ROOT, "#{project.fetch("name")}-#{project.fetch("revision")}.tar.gz")
  return archive if File.file?(archive)

  url = [
    "https://codeload.github.com",
    project.fetch("repository"),
    "tar.gz",
    project.fetch("revision")
  ].join("/")
  puts "Downloading #{project.fetch("repository")} at #{project.fetch("revision")}..."
  Tempfile.create([project.fetch("name"), ".tar.gz"], CACHE_ROOT) do |temporary|
    URI.open(url, "rb") { |source| IO.copy_stream(source, temporary) }
    temporary.flush
    File.rename(temporary.path, archive)
  end
  archive
end

def extract_archive(project, archive)
  destination = File.join(CACHE_ROOT, "sources", "#{project.fetch("name")}-#{project.fetch("revision")}")
  return destination if File.directory?(destination)

  FileUtils.mkdir_p(File.dirname(destination))
  Dir.mktmpdir("extract-", CACHE_ROOT) do |temporary|
    system("tar", "-xzf", archive, "-C", temporary, exception: true)
    extracted = Dir.children(temporary).map { |name| File.join(temporary, name) }
    raise "archive did not contain exactly one root directory" unless extracted.one?

    File.rename(extracted.fetch(0), destination)
  end
  destination
end

def selected_source_files(source_root)
  Dir.glob(File.join(source_root, "**/*.rb"), File::FNM_DOTMATCH).sort.reject do |path|
    relative = Pathname(path).relative_path_from(Pathname(source_root)).to_s
    components = relative.split(File::SEPARATOR)
    components.any? { |component| EXCLUDED_COMPONENTS.include?(component) || component.start_with?(".") } ||
      EXCLUDED_FILES.include?(relative)
  end
end

def build_corpus(project, source_root)
  destination = File.join(CACHE_ROOT, "corpora", "#{project.fetch("name")}-#{project.fetch("revision")}")
  return destination if File.file?(File.join(destination, ".complete"))

  files = selected_source_files(source_root)
  raise "#{project.fetch("repository")} contained no selected Ruby files" if files.empty?

  FileUtils.mkdir_p(File.dirname(destination))
  Dir.mktmpdir("corpus-", CACHE_ROOT) do |temporary|
    staging = File.join(temporary, "corpus")
    FileUtils.mkdir_p(staging)
    files.each do |source|
      relative = Pathname(source).relative_path_from(Pathname(source_root)).to_s
      target = File.join(staging, relative)
      FileUtils.mkdir_p(File.dirname(target))
      FileUtils.cp(source, target)
    end
    File.write(File.join(staging, ".complete"), "#{files.length}\n")
    File.rename(staging, destination)
  end
  destination
end

def command_result(command)
  stdout, stderr, status = Open3.capture3(*command)
  unless [0, 1].include?(status.exitstatus)
    raise "command failed (#{status.exitstatus}): #{command.shelljoin}\n#{stderr}"
  end
  warnings = stderr.lines.map(&:strip).reject(&:empty?)
  unexpected = warnings.reject do |warning|
    warning.match?(%r{Warning: RSpec/VariableName has the wrong namespace})
  end
  unless unexpected.empty?
    raise "command wrote unexpected stderr: #{command.shelljoin}\n#{unexpected.join("\n")}"
  end

  [JSON.parse(stdout), warnings]
end

def offense_signatures(report, corpus)
  report.fetch("files").flat_map do |file|
    reported_path = file.fetch("path")
    absolute_path = Pathname(reported_path).absolute? ? reported_path : File.expand_path(reported_path, ROOT)
    relative = Pathname(absolute_path).relative_path_from(Pathname(corpus)).to_s
    file.fetch("offenses").map do |offense|
      location = offense.fetch("location")
      [
        relative,
        offense.fetch("cop_name"),
        offense.fetch("severity"),
        offense.fetch("message"),
        location.fetch("start_line"),
        location.fetch("start_column"),
        location.fetch("last_line"),
        location.fetch("last_column")
      ]
    end
  end
end

def correctness(rustocop_report, rubocop_report, corpus)
  rustocop = offense_signatures(rustocop_report, corpus).tally
  rubocop = offense_signatures(rubocop_report, corpus).tally
  matched = rustocop.sum { |signature, count| [count, rubocop.fetch(signature, 0)].min }
  rustocop_count = rustocop.values.sum
  rubocop_count = rubocop.values.sum
  {
    "rustocop_offenses" => rustocop_count,
    "rubocop_offenses" => rubocop_count,
    "exact_matches" => matched,
    "precision" => rustocop_count.zero? ? 1.0 : matched.fdiv(rustocop_count),
    "recall" => rubocop_count.zero? ? 1.0 : matched.fdiv(rubocop_count)
  }
end

def source_measurements(corpus)
  files = Dir.glob(File.join(corpus, "**/*.rb")).sort
  {
    "files" => files.length,
    "bytes" => files.sum { |path| File.size(path) },
    "lines" => files.sum { |path| File.foreach(path).count }
  }
end

common = ["--format", "json", "--only", COPS.join(","), "--config", CONFIG_PATH]
results = PROJECTS.map do |project|
  archive = download_archive(project)
  source = extract_archive(project, archive)
  corpus = build_corpus(project, source)
  commands = {
    "rustocop_sequential" => [NATIVE, "--no-parallel", *common, corpus],
    "rustocop_jobs_4" => [NATIVE, "--jobs", "4", *common, corpus],
    "rubocop_prism" => [*RUBOCOP, "--cache", "false", "--no-server", *common, corpus]
  }

  puts "Verifying #{project.fetch("repository")}..."
  rustocop_report, rustocop_warnings = command_result(commands.fetch("rustocop_sequential"))
  parallel_report, parallel_warnings = command_result(commands.fetch("rustocop_jobs_4"))
  rubocop_report, rubocop_warnings = command_result(commands.fetch("rubocop_prism"))
  unless rustocop_warnings.empty? && parallel_warnings.empty?
    raise "rustocop unexpectedly wrote warnings for #{project.fetch("repository")}"
  end
  unless rustocop_report == parallel_report
    raise "parallel rustocop output differed from sequential output for #{project.fetch("repository")}"
  end
  measurements = source_measurements(corpus)
  inspected = [rustocop_report, rubocop_report].map { |report| report.dig("summary", "inspected_file_count") }
  unless inspected.all?(measurements.fetch("files"))
    raise "inspected file count mismatch for #{project.fetch("repository")}: #{inspected.inspect}"
  end
  comparison = correctness(rustocop_report, rubocop_report, corpus)
  raise "benchmark needs at least one RuboCop offense" if comparison.fetch("rubocop_offenses").zero?

  WARMUPS.times { commands.each_value { |command| duration(command) } }
  samples = commands.to_h { |name, _command| [name, []] }
  RUNS.times do |iteration|
    commands.keys.rotate(iteration % commands.length).each do |name|
      samples.fetch(name) << duration(commands.fetch(name))
    end
  end
  timing = samples.transform_values do |values|
    {
      "median_seconds" => percentile(values, 0.5),
      "p95_seconds" => percentile(values, 0.95)
    }
  end
  sequential = timing.dig("rustocop_sequential", "median_seconds")
  parallel = timing.dig("rustocop_jobs_4", "median_seconds")
  rubocop = timing.dig("rubocop_prism", "median_seconds")

  project.merge(
    "source_url" => "https://github.com/#{project.fetch("repository")}/tree/#{project.fetch("revision")}",
    "corpus" => measurements,
    "correctness" => comparison,
    "rubocop_warnings" => rubocop_warnings.uniq,
    "timing" => timing,
    "speedup_sequential" => rubocop.fdiv(sequential),
    "speedup_jobs_4" => rubocop.fdiv(parallel)
  )
end

report = {
  "generated_at" => Time.now.iso8601,
  "environment" => {
    "ruby" => RUBY_VERSION,
    "rubocop" => "1.87.0",
    "prism" => "1.9.0",
    "runs" => RUNS,
    "warmups" => WARMUPS
  },
  "configuration" => {
    "path" => "benchmark/project-rubocop.yml",
    "cops" => COPS,
    "excluded_components" => EXCLUDED_COMPONENTS,
    "excluded_files" => EXCLUDED_FILES
  },
  "results" => results
}
File.write(REPORT_PATH, JSON.pretty_generate(report))

puts "project\tfiles\toffenses(rust/rubocop)\texact\tsequential_ms\tjobs4_ms\trubocop_ms\tspeedup"
results.each do |result|
  puts [
    result.fetch("name"),
    result.dig("corpus", "files"),
    "#{result.dig("correctness", "rustocop_offenses")}/#{result.dig("correctness", "rubocop_offenses")}",
    result.dig("correctness", "exact_matches"),
    format("%.1f", result.dig("timing", "rustocop_sequential", "median_seconds") * 1000),
    format("%.1f", result.dig("timing", "rustocop_jobs_4", "median_seconds") * 1000),
    format("%.1f", result.dig("timing", "rubocop_prism", "median_seconds") * 1000),
    format("%.1fx", result.fetch("speedup_jobs_4"))
  ].join("\t")
end
puts "Report: #{REPORT_PATH}"
