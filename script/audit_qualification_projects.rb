# frozen_string_literal: true

require "json"
require "fileutils"
require "digest"
require "open3"
require "optparse"
require "pathname"
require "rbconfig"
require "time"
require_relative "../lib/rustocop/qualification_batch"

gem "rubocop", "=#{Rustocop::QualificationBatch::RUBOCOP_VERSION}"
require "rubocop"

ROOT = File.expand_path("..", __dir__)
DEFAULT_CONFIG = File.join(ROOT, "benchmark/project-rubocop.yml")
DEFAULT_NATIVE = File.join(ROOT, "crates/rustocop/target/release/rustocop")

options = {
  cops: nil,
  from_position: nil,
  count: 30,
  config: DEFAULT_CONFIG,
  native: DEFAULT_NATIVE,
  jobs: 4,
  build: true,
  report: nil,
  markdown: nil,
  dry_run: false
}

OptionParser.new do |parser|
  parser.banner = "Usage: ruby script/audit_qualification_projects.rb [options]"
  parser.on("--cops NAMES", "comma-separated cop names") { |value| options[:cops] = value.split(",").map(&:strip) }
  parser.on("--from-position POSITION", Integer, "one-based sorted matrix position") { |value| options[:from_position] = value }
  parser.on("--count COUNT", Integer, "reverse-order cop count (default: 30)") { |value| options[:count] = value }
  parser.on("--config PATH") { |value| options[:config] = File.expand_path(value) }
  parser.on("--native PATH") { |value| options[:native] = File.expand_path(value) }
  parser.on("--jobs COUNT", Integer) { |value| options[:jobs] = value }
  parser.on("--[no-]build", "build the release binary before auditing (default: true)") { |value| options[:build] = value }
  parser.on("--report PATH") { |value| options[:report] = File.expand_path(value) }
  parser.on("--markdown PATH") { |value| options[:markdown] = File.expand_path(value) }
  parser.on("--dry-run", "print the selected cops without running either engine") { options[:dry_run] = true }
end.parse!

matrix = RuboCop::Cop::Registry.global.map(&:cop_name).sort
cops = options[:cops]
unless cops
  start = options[:from_position] || matrix.length
  abort "--from-position must be within 1..#{matrix.length}" unless (1..matrix.length).cover?(start)
  cops = start.downto(1).first(options[:count]).map { |position| matrix.fetch(position - 1) }
end
unknown = cops - matrix
abort "unknown cops: #{unknown.join(', ')}" unless unknown.empty?
abort "no cops selected" if cops.empty?

if options[:dry_run]
  cops.each { |cop| puts "#{matrix.index(cop) + 1}\t#{cop}" }
  exit
end

rust_status, status_error, status_ok = Open3.capture3(
  "git", "status", "--porcelain", "--", "crates/rustocop", chdir: ROOT
)
abort "could not inspect native Rust source: #{status_error}" unless status_ok.success?
abort "native Rust source has uncommitted changes; commit it before auditing" unless rust_status.empty?
rust_commit, commit_error, commit_ok = Open3.capture3(
  "git", "log", "-1", "--format=%H", "--", "crates/rustocop", chdir: ROOT
)
abort "could not determine the native Rust commit: #{commit_error}" unless commit_ok.success?
rust_commit = rust_commit.strip

if options[:build]
  build_output, build_error, built = Open3.capture3(
    "cargo", "build", "--release", "--manifest-path",
    File.join(ROOT, "crates/rustocop/Cargo.toml"), chdir: ROOT
  )
  abort "Rust release build failed:\n#{build_output}\n#{build_error}" unless built.success?
end

abort "native binary not found: #{options[:native]}" unless File.executable?(options[:native])
abort "configuration not found: #{options[:config]}" unless File.file?(options[:config])

positions = cops.map { |cop| matrix.index(cop) + 1 }
options[:report] ||= File.join(ROOT, "tmp/qualification/project-gate-#{positions.max}-#{positions.min}.json")
options[:markdown] ||= options[:report].sub(/\.json\z/, ".md")

projects = Rustocop::QualificationBatch::PROJECTS.map do |project|
  corpus = File.join(
    ROOT, "tmp/project-benchmarks/corpora",
    "#{project.fetch('name')}-#{project.fetch('revision')}"
  )
  abort "project corpus not found: #{corpus}; run script/benchmark_projects.rb once" unless File.directory?(corpus)
  project.merge("corpus" => corpus)
end

rubocop_version = RuboCop::Version::STRING
abort "loaded RuboCop #{rubocop_version}, expected #{Rustocop::QualificationBatch::RUBOCOP_VERSION}" unless
  rubocop_version == Rustocop::QualificationBatch::RUBOCOP_VERSION
rubocop = [
  RbConfig.ruby,
  Gem.bin_path("rubocop", "rubocop", "=#{Rustocop::QualificationBatch::RUBOCOP_VERSION}")
].freeze

def capture(command)
  started = Process.clock_gettime(Process::CLOCK_MONOTONIC)
  stdout, stderr, status = Open3.capture3(*command, chdir: ROOT)
  elapsed = Process.clock_gettime(Process::CLOCK_MONOTONIC) - started
  {
    "stdout" => stdout,
    "stderr" => stderr,
    "exitstatus" => status.exitstatus,
    "seconds" => elapsed
  }
end

def accepted?(result)
  [0, 1].include?(result.fetch("exitstatus")) && !result.fetch("stdout").empty?
end

def native_command(native, jobs, common, corpus, cops)
  [native, "--jobs", jobs.to_s, "--format", "json", "--only", cops.join(","), *common, corpus]
end

def rubocop_command(rubocop, common, corpus, cops)
  [
    *rubocop, "--cache", "false", "--no-server", "--format", "json",
    "--only", cops.join(","), *common, corpus
  ]
end

def isolate_crash(cops, &fails)
  return cops if cops.one?

  midpoint = (cops.length + 1) / 2
  left = cops.first(midpoint)
  right = cops.drop(midpoint)
  return isolate_crash(left, &fails) if fails.call(left)
  return isolate_crash(right, &fails) if fails.call(right)

  cops
end

common = ["--config", options[:config]]
survivors = cops.dup
crashes = []
loop do
  failure = projects.lazy.filter_map do |project|
    result = capture(native_command(options[:native], options[:jobs], common, project.fetch("corpus"), survivors))
    [project, result] unless accepted?(result)
  end.first
  break unless failure

  project, result = failure
  culprit = isolate_crash(survivors) do |subset|
    probe = capture(native_command(options[:native], options[:jobs], common, project.fetch("corpus"), subset))
    !accepted?(probe)
  end
  abort "could not isolate interacting crash among: #{culprit.join(', ')}" unless culprit.one?

  cop = culprit.fetch(0)
  crashes << {
    "cop" => cop,
    "project" => project.fetch("name"),
    "stderr" => result.fetch("stderr").lines.first(12).join
  }
  survivors.delete(cop)
  abort "every selected cop crashed" if survivors.empty?
end

rubocop_errors = []
probe_corpus = projects.fetch(0).fetch("corpus")
loop do
  result = capture(rubocop_command(rubocop, common, probe_corpus, survivors))
  break if accepted?(result)

  culprit = isolate_crash(survivors) do |subset|
    probe = capture(rubocop_command(rubocop, common, probe_corpus, subset))
    !accepted?(probe)
  end
  abort "could not isolate interacting RuboCop error among: #{culprit.join(', ')}" unless culprit.one?

  cop = culprit.fetch(0)
  rubocop_errors << {
    "cop" => cop,
    "project" => projects.fetch(0).fetch("name"),
    "stderr" => result.fetch("stderr").lines.first(12).join
  }
  survivors.delete(cop)
  abort "every selected cop failed an engine gate" if survivors.empty?
end

def offense_signatures(report, corpus)
  report.fetch("files").flat_map do |file|
    reported = file.fetch("path")
    absolute = Pathname(reported).absolute? ? reported : File.expand_path(reported, ROOT)
    relative = Pathname(absolute).relative_path_from(Pathname(corpus)).to_s
    file.fetch("offenses").map do |offense|
      location = offense.fetch("location")
      {
        "path" => relative,
        "cop" => offense.fetch("cop_name"),
        "severity" => offense.fetch("severity"),
        "message" => offense.fetch("message"),
        "start_line" => location.fetch("start_line"),
        "start_column" => location.fetch("start_column"),
        "last_line" => location.fetch("last_line"),
        "last_column" => location.fetch("last_column")
      }
    end
  end
end

def signature(offense)
  offense.values_at(
    "path", "cop", "severity", "message",
    "start_line", "start_column", "last_line", "last_column"
  )
end

def compare(rust, ruby, cops)
  cops.to_h do |cop|
    rust_rows = rust.select { |item| item.fetch("cop") == cop }
    ruby_rows = ruby.select { |item| item.fetch("cop") == cop }
    rust_tally = rust_rows.map { |item| signature(item) }.tally
    ruby_tally = ruby_rows.map { |item| signature(item) }.tally
    exact = rust_tally.sum { |key, count| [count, ruby_tally.fetch(key, 0)].min }
    rust_only = rust_tally.flat_map do |key, count|
      [count - ruby_tally.fetch(key, 0), 0].max.times.map { key }
    end
    ruby_only = ruby_tally.flat_map do |key, count|
      [count - rust_tally.fetch(key, 0), 0].max.times.map { key }
    end
    [cop, {
      "rustocop" => rust_rows.length,
      "rubocop" => ruby_rows.length,
      "exact" => exact,
      "rustocop_only_examples" => rust_only.first(3),
      "rubocop_only_examples" => ruby_only.first(3)
    }]
  end
end

project_results = projects.to_h do |project|
  corpus = project.fetch("corpus")
  rust_result = capture(native_command(options[:native], options[:jobs], common, corpus, survivors))
  abort "Rustocop failed after crash isolation: #{rust_result.fetch('stderr')}" unless accepted?(rust_result)
  ruby_result = capture(rubocop_command(rubocop, common, corpus, survivors))
  abort "RuboCop failed: #{ruby_result.fetch('stderr')}" unless accepted?(ruby_result)

  rust_offenses = offense_signatures(JSON.parse(rust_result.fetch("stdout")), corpus)
  ruby_offenses = offense_signatures(JSON.parse(ruby_result.fetch("stdout")), corpus)
  [project.fetch("name"), {
    "repository" => project.fetch("repository"),
    "revision" => project.fetch("revision"),
    "files" => Dir.glob(File.join(corpus, "**/*.rb")).length,
    "timing_seconds" => {
      "rustocop" => rust_result.fetch("seconds"),
      "rubocop" => ruby_result.fetch("seconds")
    },
    "warnings" => {
      "rustocop" => rust_result.fetch("stderr").lines.map(&:strip).reject(&:empty?),
      "rubocop" => ruby_result.fetch("stderr").lines.map(&:strip).reject(&:empty?)
    },
    "by_cop" => compare(rust_offenses, ruby_offenses, survivors)
  }]
end

combined = survivors.to_h do |cop|
  values = project_results.values.map { |project| project.fetch("by_cop").fetch(cop) }
  rust = values.sum { |row| row.fetch("rustocop") }
  ruby = values.sum { |row| row.fetch("rubocop") }
  exact = values.sum { |row| row.fetch("exact") }
  classification = if rust == ruby && exact == ruby
                     ruby.zero? ? "dormant" : "exact_active"
                   else
                     "mismatch"
                   end
  [cop, {
    "rustocop" => rust,
    "rubocop" => ruby,
    "exact" => exact,
    "classification" => classification
  }]
end
crashes.each do |crash|
  combined[crash.fetch("cop")] = {
    "rustocop" => nil,
    "rubocop" => nil,
    "exact" => nil,
    "classification" => "crash"
  }
end
rubocop_errors.each do |error|
  combined[error.fetch("cop")] = {
    "rustocop" => nil,
    "rubocop" => nil,
    "exact" => nil,
    "classification" => "rubocop_error"
  }
end

report = {
  "generated_at" => Time.now.iso8601,
  "rust_commit" => rust_commit,
  "native_sha256" => Digest::SHA256.file(options[:native]).hexdigest,
  "rubocop_version" => rubocop_version,
  "matrix_start" => positions.min,
  "matrix_end" => positions.max,
  "cops" => cops,
  "crashes" => crashes,
  "rubocop_errors" => rubocop_errors,
  "projects" => project_results,
  "combined_by_cop" => combined
}
FileUtils.mkdir_p(File.dirname(options[:report]))
File.write(options[:report], JSON.pretty_generate(report))

rows = cops.map do |cop|
  row = combined.fetch(cop)
  counts = %w[rustocop rubocop exact].map { |key| row[key].nil? ? "—" : row.fetch(key).to_s }
  "| `#{cop}` | #{counts.join(' | ')} | #{row.fetch('classification')} |"
end
summary = combined.values.group_by { |row| row.fetch("classification") }.transform_values(&:length)
markdown = <<~MARKDOWN
  # Qualification project gate: positions #{positions.max}–#{positions.min}

  Generated against Rust source `#{report.fetch('rust_commit')}` and RuboCop
  #{report.fetch('rubocop_version')} across the pinned project corpora.

  - Exact and active: #{summary.fetch('exact_active', 0)}
  - Exact but dormant: #{summary.fetch('dormant', 0)}
  - Mismatching: #{summary.fetch('mismatch', 0)}
  - Crashing: #{summary.fetch('crash', 0)}
  - RuboCop gate errors: #{summary.fetch('rubocop_error', 0)}

  | Cop | Rustocop | RuboCop | Exact | Classification |
  | --- | ---: | ---: | ---: | --- |
  #{rows.join("\n")}

  Exact-active cops are candidates for upstream, correction, evidence, and source
  boundary review. Dormant, mismatching, crashing, and engine-error cops receive
  no qualification credit.
MARKDOWN
FileUtils.mkdir_p(File.dirname(options[:markdown]))
File.write(options[:markdown], markdown)

puts "Project gate: #{summary.sort.map { |key, value| "#{key}=#{value}" }.join(', ')}"
puts "Report: #{options[:report]}"
puts "Summary: #{options[:markdown]}"
