# frozen_string_literal: true

require "json"
require "fileutils"
require "digest"
require "optparse"
require "pathname"
require "rbconfig"
require "time"
require_relative "../lib/rustocop/artifact_store"
require_relative "../lib/rustocop/diagnostic_signatures"
require_relative "../lib/rustocop/process_runner"
require_relative "../lib/rustocop/project_mismatch_inventory"
require_relative "../lib/rustocop/repository_layout"
require_relative "../lib/rustocop/source_fingerprint"
require_relative "../lib/rustocop/project_corpus"
require_relative "../lib/rustocop/compatibility_status"

gem "rubocop", "=#{Rustocop::ProjectCorpus::RUBOCOP_VERSION}"
require "rubocop"

LAYOUT = Rustocop::RepositoryLayout.default
ROOT = LAYOUT.root
DEFAULT_CONFIG = LAYOUT.benchmark_config
DEFAULT_NATIVE = LAYOUT.native_binary
DEFAULT_RUBOCOP_REFERENCE = LAYOUT.compatibility_evidence("project_rubocop_reference.json.gz")
RUBOCOP_REFERENCE_COMPATIBILITY = File.join(ROOT, "lib/rustocop/rubocop_reference_compatibility.rb")
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
  mismatch_inventory: nil,
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
  parser.on("--mismatch-inventory PATH", "compressed exhaustive mismatch artifact") do |value|
    options[:mismatch_inventory] = File.expand_path(value)
  end
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

rust_status = Rustocop::ProcessRunner.capture(
  "git", "status", "--porcelain", "--", "crates/rustocop", chdir: ROOT
)
abort "could not inspect native Rust source: #{rust_status.stderr}" unless rust_status.success?
rust_commit = if rust_status.stdout.empty?
                commit = Rustocop::ProcessRunner.capture(
                  "git", "log", "-1", "--format=%H", "--", "crates/rustocop", chdir: ROOT
                )
                abort "could not determine the native Rust commit: #{commit.stderr}" unless commit.success?
                commit.stdout.strip
              end

if options[:build]
  build = Rustocop::ProcessRunner.capture(
    "cargo", "build", "--release", "--manifest-path",
    LAYOUT.rust_manifest, chdir: ROOT
  )
  abort "Rust release build failed:\n#{build.stdout}\n#{build.stderr}" unless build.success?
end

abort "native binary not found: #{options[:native]}" unless File.executable?(options[:native])
abort "configuration not found: #{options[:config]}" unless File.file?(options[:config])

positions = cops.map { |cop| matrix.index(cop) + 1 }
options[:report] ||= File.join(ROOT, "tmp/project-parity/project-gate-#{positions.max}-#{positions.min}.json")
options[:markdown] ||= options[:report].sub(/\.json\z/, ".md")
options[:mismatch_inventory] ||= if options[:report].end_with?(".json")
                                   options[:report].sub(/\.json\z/, ".mismatches.json.gz")
                                 else
                                   "#{options[:report]}.mismatches.json.gz"
                                 end

projects = Rustocop::ProjectCorpus::PROJECTS.map do |project|
  corpus = LAYOUT.project_corpus(project)
  unless File.directory?(corpus)
    abort "project corpus not found: #{corpus}; " \
      "run PROJECT_BENCHMARK_PREPARE_ONLY=1 bundle exec ruby script/benchmark_projects.rb"
  end
  project.merge("corpus" => corpus)
end

rubocop_version = RuboCop::Version::STRING
abort "loaded RuboCop #{rubocop_version}, expected #{Rustocop::ProjectCorpus::RUBOCOP_VERSION}" unless
  rubocop_version == Rustocop::ProjectCorpus::RUBOCOP_VERSION
rubocop = [
  RbConfig.ruby,
  "-r", RUBOCOP_REFERENCE_COMPATIBILITY,
  Gem.bin_path("rubocop", "rubocop", "=#{Rustocop::ProjectCorpus::RUBOCOP_VERSION}")
].freeze

def capture(command)
  Rustocop::ProcessRunner.capture(*command, chdir: ROOT).to_h
end

def accepted?(result)
  [0, 1].include?(result.fetch("exitstatus")) &&
    !result.fetch("stdout").empty? &&
    !result.fetch("stderr").include?("An error occurred while")
end

def cop_inspection_error?(result)
  stderr = result.fetch("stderr")
  stderr.include?("An error occurred while") || stderr.include?("cannot be used with --only")
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

def capture_rubocop(rubocop, common, corpus, cops)
  special = "Lint/RedundantCopDisableDirective"
  return capture(rubocop_command(rubocop, common, corpus, cops)) unless cops.include?(special) && cops.length > 1

  results = [cops - [special], [special]].map do |selection|
    capture(rubocop_command(rubocop, common, corpus, selection))
  end
  failed = results.find { |result| !accepted?(result) }
  return failed if failed

  selections = [cops - [special], [special]]
  reports = results.map { |result| JSON.parse(result.fetch("stdout")) }
  files = reports
    .zip(selections)
    .flat_map do |report, selection|
      report.fetch("files").map do |file|
        file.merge(
          "offenses" => file.fetch("offenses").select do |offense|
            selection.include?(offense.fetch("cop_name"))
          end
        )
      end
    end
    .group_by { |file| file.fetch("path") }
    .map do |path, entries|
      { "path" => path, "offenses" => entries.flat_map { |entry| entry.fetch("offenses") } }
    end
  offense_count = files.sum { |file| file.fetch("offenses").length }
  {
    "stdout" => JSON.generate(
      "metadata" => reports.fetch(0).fetch("metadata"),
      "files" => files,
      "summary" => {
        "offense_count" => offense_count,
        "target_file_count" => files.length,
        "inspected_file_count" => files.length
      }
    ),
    "stderr" => results.map { |result| result.fetch("stderr") }.join,
    "exitstatus" => offense_count.zero? ? 0 : 1,
    "seconds" => results.sum { |result| result.fetch("seconds") }
  }
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
def encode_offenses(offenses, cops)
  selected = cops.to_h { |cop| [cop, true] }
  offenses = offenses.select { |offense| selected[offense.fetch("cop")] }
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
    result = capture_rubocop(rubocop, common, probe_corpus, rubocop_survivors)
    if accepted?(result)
      probe_rubocop_result = result
      break
    end
    abort "RuboCop could not parse the #{projects.fetch(0).fetch('name')} corpus: #{result.fetch('stderr')}" unless
      cop_inspection_error?(result)

    culprit = isolate_crash(rubocop_survivors) do |subset|
      probe = capture_rubocop(rubocop, common, probe_corpus, subset)
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
               capture_rubocop(rubocop, common, project.fetch("corpus"), rubocop_survivors)
             end
    until accepted?(result)
      abort "RuboCop could not parse the #{project.fetch('name')} corpus: #{result.fetch('stderr')}" unless
        cop_inspection_error?(result)

      culprit = isolate_crash(rubocop_survivors) do |subset|
        probe = capture_rubocop(rubocop, common, project.fetch("corpus"), subset)
        !accepted?(probe)
      end
      abort "could not isolate interacting RuboCop error among: #{culprit.join(', ')}" unless culprit.one?

      cop = culprit.fetch(0)
      rubocop_errors << {
        "cop" => cop,
        "project" => project.fetch("name"),
        "stderr" => result.fetch("stderr").lines.first(12).join
      }
      rubocop_survivors.delete(cop)
      abort "every selected cop failed a RuboCop reference gate" if rubocop_survivors.empty?

      warn "RuboCop reference retry: #{project.fetch('name')} (#{rubocop_survivors.length} cops)"
      result = capture_rubocop(rubocop, common, project.fetch("corpus"), rubocop_survivors)
    end

    offenses = Rustocop::DiagnosticSignatures.hashes_from_report(
      JSON.parse(result.fetch("stdout")), corpus: project.fetch("corpus"), root: ROOT
    )
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
  Rustocop::ArtifactStore.write_gzip_json(options[:rubocop_reference], rubocop_reference)
else
  begin
    rubocop_reference = Rustocop::ArtifactStore.read_gzip_json(
      options[:rubocop_reference], label: "RuboCop reference"
    )
  rescue Rustocop::ArtifactStore::Error => e
    abort "#{e.message}; rerun with --refresh-rubocop-reference"
  end
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

mismatch_projects = {}
project_results = projects.to_h do |project|
  corpus = project.fetch("corpus")
  warn "Exact comparison: #{project.fetch('name')} (#{survivors.length} cops)"
  rust_result = rust_results.fetch(project.fetch("name"))
  ruby_result = rubocop_reference.fetch("projects").fetch(project.fetch("name"))

  rust_offenses = Rustocop::DiagnosticSignatures.hashes_from_report(
    JSON.parse(rust_result.fetch("stdout")), corpus:, root: ROOT
  )
  ruby_offenses = decode_offenses(ruby_result, rubocop_reference.fetch("cops")).select do |offense|
    survivor_lookup.key?(offense.fetch("cop"))
  end
  comparison = Rustocop::ProjectMismatchInventory.compare(rust_offenses, ruby_offenses, survivors)
  mismatch_paths = comparison.entries.map { |entry| entry.fetch(1) }.uniq.sort
  mismatch_projects[project.fetch("name")] = {
    "repository" => project.fetch("repository"),
    "revision" => project.fetch("revision"),
    "files" => mismatch_paths.to_h do |path|
      source_path = File.join(corpus, path)
      abort "mismatch source not found: #{source_path}" unless File.file?(source_path)

      [path, {
        "sha256" => Digest::SHA256.file(source_path).hexdigest,
        "bytes" => File.size(source_path)
      }]
    end,
    "entries" => comparison.entries
  }
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
    "by_cop" => comparison.by_cop
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

generated_at = Time.now.iso8601
inventory = {
  "format_version" => Rustocop::ProjectMismatchInventory::FORMAT_VERSION,
  "generated_at" => generated_at,
  "report" => Pathname(options[:report]).relative_path_from(Pathname(ROOT)).to_s,
  "config" => {
    "path" => Pathname(options[:config]).relative_path_from(Pathname(ROOT)).to_s,
    "sha256" => config_sha256
  },
  "fields" => Rustocop::ProjectMismatchInventory::ENTRY_FIELDS,
  "distinct_mismatches" => mismatch_projects.values.sum { |project| project.fetch("entries").length },
  "unmatched_offenses" => mismatch_projects.values.sum do |project|
    project.fetch("entries").sum { |entry| entry.fetch(-1) }
  end,
  "projects" => mismatch_projects
}
Rustocop::ArtifactStore.write_gzip_json(options[:mismatch_inventory], inventory)
inventory_metadata = {
  "path" => Pathname(options[:mismatch_inventory]).relative_path_from(Pathname(ROOT)).to_s,
  "sha256" => Digest::SHA256.file(options[:mismatch_inventory]).hexdigest,
  "format_version" => inventory.fetch("format_version"),
  "distinct_mismatches" => inventory.fetch("distinct_mismatches"),
  "unmatched_offenses" => inventory.fetch("unmatched_offenses")
}

report = {
  "generated_at" => generated_at,
  "rust_commit" => rust_commit,
  "cop_source_sha256" => Rustocop::SourceFingerprint.cops(root: ROOT),
  "native_sha256" => Digest::SHA256.file(options[:native]).hexdigest,
  "rubocop_version" => rubocop_version,
  "rubocop_reference" => {
    "path" => Pathname(options[:rubocop_reference]).relative_path_from(Pathname(ROOT)).to_s,
    "sha256" => Digest::SHA256.file(options[:rubocop_reference]).hexdigest,
    "generated_at" => rubocop_reference.fetch("generated_at"),
    "config_sha256" => config_sha256,
    "source" => reference_source
  },
  "mismatch_inventory" => inventory_metadata,
  "matrix_start" => positions.min,
  "matrix_end" => positions.max,
  "cops" => cops,
  "crashes" => crashes,
  "rubocop_errors" => rubocop_errors,
  "projects" => project_results,
  "combined_by_cop" => combined
}
Rustocop::ArtifactStore.write_json(options[:report], report)

rows = cops.map do |cop|
  row = combined.fetch(cop)
  counts = %w[rustocop rubocop exact].map { |key| row[key].nil? ? "—" : row.fetch(key).to_s }
  "| `#{cop}` | #{counts.join(' | ')} | #{row.fetch('classification')} |"
end
summary = combined.values.group_by { |row| row.fetch("classification") }.transform_values(&:length)
markdown = <<~MARKDOWN
  # #{projects.length}-project parity audit: positions #{positions.max}–#{positions.min}

  Generated against Rust source `#{report['rust_commit'] || "uncommitted native #{report.fetch('native_sha256')[0, 12]}"}` and RuboCop
  #{report.fetch('rubocop_version')} across the pinned project corpora.

  - Project-exact: #{summary.fetch('project_exact', 0)}
  - Exact but dormant: #{summary.fetch('dormant', 0)}
  - Mismatching: #{summary.fetch('mismatch', 0)}
  - Crashing: #{summary.fetch('crash', 0)}
  - RuboCop gate errors: #{summary.fetch('rubocop_error', 0)}
  - Exhaustive mismatch signatures: #{inventory.fetch('distinct_mismatches')}
  - Unmatched offense instances: #{inventory.fetch('unmatched_offenses')}

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
puts "Mismatch inventory: #{options[:mismatch_inventory]} " \
     "(#{inventory.fetch('distinct_mismatches')} signatures, " \
     "#{inventory.fetch('unmatched_offenses')} unmatched offenses)"
puts "Summary: #{options[:markdown]}"
