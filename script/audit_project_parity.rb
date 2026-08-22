# frozen_string_literal: true

require "json"
require "fileutils"
require "digest"
require "open3"
require "optparse"
require "pathname"
require "rbconfig"
require "time"
require "zlib"
require_relative "../lib/rustocop/project_corpus"
require_relative "../lib/rustocop/compatibility_status"

gem "rubocop", "=#{Rustocop::ProjectCorpus::RUBOCOP_VERSION}"
require "rubocop"

ROOT = File.expand_path("..", __dir__)
DEFAULT_CONFIG = File.join(ROOT, "benchmark/project-rubocop.yml")
DEFAULT_NATIVE = File.join(ROOT, "crates/rustocop/target/release/rustocop")
DEFAULT_RUBOCOP_REFERENCE = File.join(
  ROOT, "spec/compatibility_evidence/project_rubocop_reference.json.gz"
)
RUBOCOP_REFERENCE_VERSION = 2

options = {
  cops: nil,
  active: false,
  from_position: nil,
  count: 30,
  config: DEFAULT_CONFIG,
  native: DEFAULT_NATIVE,
  jobs: 4,
  build: true,
  report: nil,
  markdown: nil,
  rubocop_reference: DEFAULT_RUBOCOP_REFERENCE,
  refresh_rubocop_reference: false,
  dry_run: false
}

OptionParser.new do |parser|
  parser.banner = "Usage: ruby script/audit_project_parity.rb [options]"
  parser.on("--cops NAMES", "comma-separated cop names") { |value| options[:cops] = value.split(",").map(&:strip) }
  parser.on("--active", "audit every cop in the active Rustocop corpus") { options[:active] = true }
  parser.on("--from-position POSITION", Integer, "one-based sorted matrix position") { |value| options[:from_position] = value }
  parser.on("--count COUNT", Integer, "reverse-order cop count (default: 30)") { |value| options[:count] = value }
  parser.on("--config PATH") { |value| options[:config] = File.expand_path(value) }
  parser.on("--native PATH") { |value| options[:native] = File.expand_path(value) }
  parser.on("--jobs COUNT", Integer) { |value| options[:jobs] = value }
  parser.on("--[no-]build", "build the release binary before auditing (default: true)") { |value| options[:build] = value }
  parser.on("--report PATH") { |value| options[:report] = File.expand_path(value) }
  parser.on("--markdown PATH") { |value| options[:markdown] = File.expand_path(value) }
  parser.on("--rubocop-reference PATH", "compressed RuboCop result snapshot") do |value|
    options[:rubocop_reference] = File.expand_path(value)
  end
  parser.on("--refresh-rubocop-reference", "run RuboCop and replace the reference snapshot") do
    options[:refresh_rubocop_reference] = true
  end
  parser.on("--dry-run", "print the selected cops without running either engine") { options[:dry_run] = true }
end.parse!

matrix = RuboCop::Cop::Registry.global.map(&:cop_name).sort
active_matrix = Rustocop::CompatibilityStatus.load(root: ROOT).built_in_cops.sort
cops = if options[:active]
         abort "--active cannot be combined with --cops or --from-position" if
           options[:cops] || options[:from_position]
         active_matrix
       else
         options[:cops]
       end
unless cops
  start = options[:from_position] || matrix.length
  abort "--from-position must be within 1..#{matrix.length}" unless (1..matrix.length).cover?(start)
  cops = start.downto(1).first(options[:count]).map { |position| matrix.fetch(position - 1) }
end
unknown = cops - matrix
abort "unknown cops: #{unknown.join(', ')}" unless unknown.empty?
abort "no cops selected" if cops.empty?
if options[:refresh_rubocop_reference] && options[:rubocop_reference] == DEFAULT_RUBOCOP_REFERENCE && cops.sort != active_matrix
  abort "refusing to replace the default RuboCop reference with a partial active-cop selection; " \
        "pass --active or use --rubocop-reference PATH"
end

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
options[:report] ||= File.join(ROOT, "tmp/project-parity/project-gate-#{positions.max}-#{positions.min}.json")
options[:markdown] ||= options[:report].sub(/\.json\z/, ".md")

projects = Rustocop::ProjectCorpus::PROJECTS.map do |project|
  corpus = File.join(
    ROOT, "tmp/project-benchmarks/corpora",
    "#{project.fetch('name')}-#{project.fetch('revision')}"
  )
  abort "project corpus not found: #{corpus}; run script/benchmark_projects.rb once" unless File.directory?(corpus)
  project.merge("corpus" => corpus)
end

rubocop_version = RuboCop::Version::STRING
abort "loaded RuboCop #{rubocop_version}, expected #{Rustocop::ProjectCorpus::RUBOCOP_VERSION}" unless
  rubocop_version == Rustocop::ProjectCorpus::RUBOCOP_VERSION
rubocop = [
  RbConfig.ruby,
  Gem.bin_path("rubocop", "rubocop", "=#{Rustocop::ProjectCorpus::RUBOCOP_VERSION}")
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
  [0, 1].include?(result.fetch("exitstatus")) &&
    !result.fetch("stdout").empty? &&
    !result.fetch("stderr").include?("An error occurred while")
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

def read_gzip_json(path)
  Zlib::GzipReader.open(path) { |gzip| JSON.parse(gzip.read) }
rescue Errno::ENOENT
  abort "RuboCop reference not found: #{path}; rerun with --refresh-rubocop-reference"
rescue Zlib::GzipFile::Error, JSON::ParserError => e
  abort "invalid RuboCop reference #{path}: #{e.message}"
end

def write_gzip_json(path, value)
  FileUtils.mkdir_p(File.dirname(path))
  temporary = "#{path}.#{$$}.tmp"
  Zlib::GzipWriter.open(temporary) do |gzip|
    gzip.mtime = 0
    gzip.write(JSON.generate(value))
  end
  File.rename(temporary, path)
ensure
  FileUtils.rm_f(temporary) if defined?(temporary)
end

def validate_reference!(reference, path, rubocop_version:, config_sha256:, projects:, cops:)
  expected_projects = projects.map { |project| project.slice("name", "repository", "revision") }
  failures = []
  failures << "format version" unless reference["version"] == RUBOCOP_REFERENCE_VERSION
  failures << "kind" unless reference["kind"] == "rubocop_project_reference"
  failures << "RuboCop version" unless reference["rubocop_version"] == rubocop_version
  failures << "configuration" unless reference["config_sha256"] == config_sha256
  failures << "pinned projects" unless reference["project_revisions"] == expected_projects
  cached_projects = reference.fetch("projects", {})
  expected_projects.each do |project|
    name = project.fetch("name")
    corpus = projects.find { |candidate| candidate.fetch("name") == name }.fetch("corpus")
    expected_files = Dir.glob(File.join(corpus, "**/*.rb")).length
    failures << "#{name} corpus" unless cached_projects.dig(name, "files") == expected_files
  end
  missing_cops = cops - reference.fetch("cops", [])
  failures << "selected cops (missing #{missing_cops.join(', ')})" unless missing_cops.empty?
  return if failures.empty?

  abort "stale RuboCop reference #{path}: #{failures.join(', ')}; " \
        "rerun with --refresh-rubocop-reference"
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

def encode_offenses(offenses, cops)
  paths = offenses.map { |offense| offense.fetch("path") }.uniq
  messages = offenses.map { |offense| offense.fetch("message") }.uniq
  path_indexes = paths.each_with_index.to_h
  cop_indexes = cops.each_with_index.to_h
  message_indexes = messages.each_with_index.to_h
  rows = offenses.map do |offense|
    [
      path_indexes.fetch(offense.fetch("path")),
      cop_indexes.fetch(offense.fetch("cop")),
      offense.fetch("severity"),
      message_indexes.fetch(offense.fetch("message")),
      offense.fetch("start_line"),
      offense.fetch("start_column"),
      offense.fetch("last_line"),
      offense.fetch("last_column")
    ]
  end
  { "paths" => paths, "messages" => messages, "offenses" => rows }
end

def decode_offenses(project, cops)
  paths = project.fetch("paths")
  messages = project.fetch("messages")
  project.fetch("offenses").map do |row|
    {
      "path" => paths.fetch(row.fetch(0)),
      "cop" => cops.fetch(row.fetch(1)),
      "severity" => row.fetch(2),
      "message" => messages.fetch(row.fetch(3)),
      "start_line" => row.fetch(4),
      "start_column" => row.fetch(5),
      "last_line" => row.fetch(6),
      "last_column" => row.fetch(7)
    }
  end
end

config_sha256 = Digest::SHA256.file(options[:config]).hexdigest
reference_source = options[:refresh_rubocop_reference] ? "refreshed" : "cached"
if options[:refresh_rubocop_reference]
  rubocop_survivors = cops.dup
  rubocop_errors = []
  probe_corpus = projects.fetch(0).fetch("corpus")
  probe_rubocop_result = nil
  loop do
    warn "RuboCop engine gate: #{projects.fetch(0).fetch('name')} (#{rubocop_survivors.length} cops)"
    result = capture(rubocop_command(rubocop, common, probe_corpus, rubocop_survivors))
    if accepted?(result)
      probe_rubocop_result = result
      break
    end

    culprit = isolate_crash(rubocop_survivors) do |subset|
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
    rubocop_survivors.delete(cop)
    abort "every selected cop failed an engine gate" if rubocop_survivors.empty?
  end

  reference_projects = projects.to_h do |project|
    warn "RuboCop reference: #{project.fetch('name')} (#{rubocop_survivors.length} cops)"
    result = if project == projects.fetch(0)
               probe_rubocop_result
             else
               capture(rubocop_command(rubocop, common, project.fetch("corpus"), rubocop_survivors))
             end
    abort "RuboCop failed: #{result.fetch('stderr')}" unless accepted?(result)

    offenses = offense_signatures(JSON.parse(result.fetch("stdout")), project.fetch("corpus"))
    [project.fetch("name"), {
      "files" => Dir.glob(File.join(project.fetch("corpus"), "**/*.rb")).length,
      "seconds" => result.fetch("seconds"),
      "warning_count" => result.fetch("stderr").lines.count { |line| !line.strip.empty? }
    }.merge(encode_offenses(offenses, cops))]
  end
  rubocop_reference = {
    "version" => RUBOCOP_REFERENCE_VERSION,
    "kind" => "rubocop_project_reference",
    "generated_at" => Time.now.iso8601,
    "rubocop_version" => rubocop_version,
    "config_sha256" => config_sha256,
    "project_revisions" => projects.map { |project| project.slice("name", "repository", "revision") },
    "cops" => cops,
    "rubocop_errors" => rubocop_errors,
    "projects" => reference_projects
  }
  write_gzip_json(options[:rubocop_reference], rubocop_reference)
else
  rubocop_reference = read_gzip_json(options[:rubocop_reference])
end
validate_reference!(
  rubocop_reference, options[:rubocop_reference], rubocop_version: rubocop_version,
  config_sha256: config_sha256, projects: projects, cops: cops
)
rubocop_errors = rubocop_reference.fetch("rubocop_errors").select { |error| cops.include?(error.fetch("cop")) }

rust_survivors = cops.dup
crashes = []
rust_results = nil
loop do
  candidate_results = {}
  failure = nil
  projects.each do |project|
    warn "Rust crash gate: #{project.fetch('name')} (#{rust_survivors.length} cops)"
    result = capture(native_command(options[:native], options[:jobs], common, project.fetch("corpus"), rust_survivors))
    unless accepted?(result)
      failure = [project, result]
      break
    end
    candidate_results[project.fetch("name")] = result
  end
  unless failure
    rust_results = candidate_results
    break
  end

  project, result = failure
  culprit = isolate_crash(rust_survivors) do |subset|
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
  rust_survivors.delete(cop)
  abort "every selected cop crashed" if rust_survivors.empty?
end
survivors = rust_survivors - rubocop_errors.map { |error| error.fetch("cop") }
survivor_lookup = survivors.to_h { |cop| [cop, true] }

def signature(offense)
  offense.values_at(
    "path", "cop", "severity", "message",
    "start_line", "start_column", "last_line", "last_column"
  )
end

def compare(rust, ruby, cops)
  rust_by_cop = rust.group_by { |item| item.fetch("cop") }
  ruby_by_cop = ruby.group_by { |item| item.fetch("cop") }
  cops.to_h do |cop|
    rust_rows = rust_by_cop.fetch(cop, [])
    ruby_rows = ruby_by_cop.fetch(cop, [])
    rust_tally = rust_rows.map { |item| signature(item) }.tally
    ruby_tally = ruby_rows.map { |item| signature(item) }.tally
    exact = rust_tally.sum { |key, count| [count, ruby_tally.fetch(key, 0)].min }
    rust_only = unmatched_examples(rust_tally, ruby_tally)
    ruby_only = unmatched_examples(ruby_tally, rust_tally)
    [cop, {
      "rustocop" => rust_rows.length,
      "rubocop" => ruby_rows.length,
      "exact" => exact,
      "rustocop_only_examples" => rust_only,
      "rubocop_only_examples" => ruby_only
    }]
  end
end

def unmatched_examples(left, right, limit = 3)
  left.each_with_object([]) do |(key, count), examples|
    missing = [count - right.fetch(key, 0), 0].max
    [missing, limit - examples.length].min.times { examples << key }
    break examples if examples.length == limit
  end
end

project_results = projects.to_h do |project|
  corpus = project.fetch("corpus")
  warn "Exact comparison: #{project.fetch('name')} (#{survivors.length} cops)"
  rust_result = rust_results.fetch(project.fetch("name"))
  ruby_result = rubocop_reference.fetch("projects").fetch(project.fetch("name"))

  rust_offenses = offense_signatures(JSON.parse(rust_result.fetch("stdout")), corpus)
  ruby_offenses = decode_offenses(ruby_result, rubocop_reference.fetch("cops")).select do |offense|
    survivor_lookup.key?(offense.fetch("cop"))
  end
  [project.fetch("name"), {
    "repository" => project.fetch("repository"),
    "revision" => project.fetch("revision"),
    "files" => Dir.glob(File.join(corpus, "**/*.rb")).length,
    "timing_seconds" => {
      "rustocop" => rust_result.fetch("seconds"),
      "rubocop" => ruby_result.fetch("seconds")
    },
    "warning_counts" => {
      "rustocop" => rust_result.fetch("stderr").lines.count { |line| !line.strip.empty? },
      "rubocop" => ruby_result.fetch("warning_count")
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
                     ruby.zero? ? "dormant" : "project_exact"
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
  "rubocop_reference" => {
    "path" => Pathname(options[:rubocop_reference]).relative_path_from(Pathname(ROOT)).to_s,
    "sha256" => Digest::SHA256.file(options[:rubocop_reference]).hexdigest,
    "generated_at" => rubocop_reference.fetch("generated_at"),
    "config_sha256" => config_sha256,
    "source" => reference_source
  },
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
  # Ten-project parity audit: positions #{positions.max}–#{positions.min}

  Generated against Rust source `#{report.fetch('rust_commit')}` and RuboCop
  #{report.fetch('rubocop_version')} across the pinned project corpora.

  - Project-exact: #{summary.fetch('project_exact', 0)}
  - Exact but dormant: #{summary.fetch('dormant', 0)}
  - Mismatching: #{summary.fetch('mismatch', 0)}
  - Crashing: #{summary.fetch('crash', 0)}
  - RuboCop gate errors: #{summary.fetch('rubocop_error', 0)}

  | Cop | Rustocop | RuboCop | Exact | Classification |
  | --- | ---: | ---: | ---: | --- |
  #{rows.join("\n")}

  Project-exact cops match complete diagnostic signatures across every pinned
  project. Dormant cops were not exercised; mismatching, crashing, and
  engine-error cops are not compatible with this corpus.
MARKDOWN
FileUtils.mkdir_p(File.dirname(options[:markdown]))
File.write(options[:markdown], markdown)

puts "Project gate: #{summary.sort.map { |key, value| "#{key}=#{value}" }.join(', ')}"
puts "Report: #{options[:report]}"
puts "Summary: #{options[:markdown]}"
