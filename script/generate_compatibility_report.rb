# frozen_string_literal: true

require "fileutils"
require "json"
require "open3"
require "optparse"
require "rbconfig"
require "time"
require_relative "../lib/rustocop/compatibility_status"

ROOT = File.expand_path("..", __dir__)
EVIDENCE_ROOT = File.join(ROOT, "spec", "compatibility_evidence")
DEFAULT_FIXTURE_SNAPSHOT = File.join(EVIDENCE_ROOT, "fixtures.json")
DEFAULT_PROJECT_SNAPSHOT = File.join(EVIDENCE_ROOT, "projects.json")
DEFAULT_OUTPUT = File.join(ROOT, "docs", "compatibility.md")
RUST_COP_ROOT = File.join(ROOT, "crates", "rustocop", "src", "cops")
ACTIVE_COPS = Rustocop::CompatibilityStatus.load(root: ROOT).built_in_cops.sort.freeze

options = {
  fixture_snapshot: DEFAULT_FIXTURE_SNAPSHOT,
  project_snapshot: DEFAULT_PROJECT_SNAPSHOT,
  output: DEFAULT_OUTPUT,
  native: File.join(ROOT, "crates", "rustocop", "target", "release", "rustocop"),
  jobs: 8,
  check: false
}

OptionParser.new do |parser|
  parser.banner = "Usage: ruby script/generate_compatibility_report.rb [options]"
  parser.on("--fixture-report PATH", "import a fresh full fixture comparison") do |path|
    options[:fixture_report] = File.expand_path(path)
  end
  parser.on("--project-report PATH", "import a fresh complete project audit") do |path|
    options[:project_report] = File.expand_path(path)
  end
  parser.on("--refresh-fixtures", "run the fixture differential before generating") do
    options[:refresh_fixtures] = true
  end
  parser.on("--refresh-projects", "run the expensive complete project audit before generating") do
    options[:refresh_projects] = true
  end
  parser.on("--refresh-rubocop-reference", "replace the stored RuboCop reference during project refresh") do
    options[:refresh_rubocop_reference] = true
  end
  parser.on("--native PATH") { |path| options[:native] = File.expand_path(path) }
  parser.on("--jobs COUNT", Integer) { |count| options[:jobs] = count }
  parser.on("--fixture-snapshot PATH") { |path| options[:fixture_snapshot] = File.expand_path(path) }
  parser.on("--project-snapshot PATH") { |path| options[:project_snapshot] = File.expand_path(path) }
  parser.on("--output PATH") { |path| options[:output] = File.expand_path(path) }
  parser.on("--check", "fail instead of writing when snapshots or output are stale") do
    options[:check] = true
  end
end.parse!

abort "refresh options cannot be combined with --check" if options[:check] &&
  (options[:refresh_fixtures] || options[:refresh_projects] || options[:refresh_rubocop_reference])
abort "--refresh-rubocop-reference requires --refresh-projects" if
  options[:refresh_rubocop_reference] && !options[:refresh_projects]

if options[:refresh_projects]
  project_report = File.join(ROOT, "tmp", "project-parity", "all-cops-current.json")
  project_markdown = project_report.sub(/\.json\z/, ".md")
  command = [
    RbConfig.ruby, File.join(ROOT, "script", "audit_project_parity.rb"),
    "--active", "--jobs", options[:jobs].to_s,
    "--report", project_report, "--markdown", project_markdown
  ]
  command << "--refresh-rubocop-reference" if options[:refresh_rubocop_reference]
  abort "complete project audit failed" unless system(*command, chdir: ROOT)
  options[:project_report] = project_report
end

if options[:refresh_fixtures]
  unless options[:refresh_projects]
    build = [
      "cargo", "build", "--release", "--manifest-path",
      File.join(ROOT, "crates", "rustocop", "Cargo.toml")
    ]
    abort "Rust release build failed" unless system(*build, chdir: ROOT)
  end

  fixture_report = File.join(ROOT, "tmp", "compatibility", "fixtures-current.json")
  FileUtils.mkdir_p(File.dirname(fixture_report))
  FileUtils.rm_f(fixture_report)
  command = [
    RbConfig.ruby, File.join(ROOT, "script", "compare_upstream_cop_specs.rb"),
    "--jobs", options[:jobs].to_s, "--report", fixture_report
  ]
  system(
    { "RUSTOCOP_NATIVE_PATH" => options[:native] },
    *command,
    chdir: ROOT
  )
  exitstatus = Process.last_status.exitstatus
  abort "fixture differential failed" unless [0, 1].include?(exitstatus) && File.file?(fixture_report)
  options[:fixture_report] = fixture_report
end

def read_json(path, label)
  abort "#{label} not found: #{path}" unless File.file?(path)

  JSON.parse(File.read(path))
rescue JSON::ParserError => e
  abort "invalid #{label} #{path}: #{e.message}"
end

def evidence_time(report, path)
  value = report.fetch("generated_at", File.mtime(path).iso8601)
  parsed = Time.iso8601(value)
  abort "evidence timestamp must include a time and UTC offset: #{value.inspect}" unless
    value.match?(/T\d{2}:\d{2}:\d{2}(?:\.\d+)?(?:Z|[+-]\d{2}:\d{2})\z/)

  parsed.iso8601
rescue ArgumentError
  abort "evidence timestamp must be ISO 8601: #{value.inspect}"
end

def evidence_timestamp(snapshot, label)
  value = snapshot.fetch("updated_at")
  Time.iso8601(value)
  abort "#{label} updated_at must include a time and UTC offset: #{value.inspect}" unless
    value.match?(/T\d{2}:\d{2}:\d{2}(?:\.\d+)?(?:Z|[+-]\d{2}:\d{2})\z/)

  value
rescue ArgumentError
  abort "#{label} updated_at must be ISO 8601: #{value.inspect}"
end

def fixture_snapshot(report, path)
  results = report.fetch("results").transform_values do |row|
    passed = row.fetch("passed")
    total = row.fetch("total")
    {
      "passed" => passed,
      "total" => total,
      "status" => passed == total && total.positive? ? "compatible" : "mismatch"
    }
  end
  {
    "version" => 1,
    "kind" => "fixture_compatibility",
    "updated_at" => evidence_time(report, path),
    "rust_commit" => report["rust_commit"],
    "native_sha256" => report["native_sha256"],
    "fixture_corpus_sha256" => report["fixture_corpus_sha256"],
    "rubocop_version" => report.fetch("rubocop_version"),
    "results" => results.sort.to_h
  }
end

def project_snapshot(report, path)
  results = report.fetch("combined_by_cop").transform_values do |row|
    row.slice("rustocop", "rubocop", "exact", "classification")
  end
  {
    "version" => 1,
    "kind" => "project_compatibility",
    "updated_at" => evidence_time(report, path),
    "rust_commit" => report["rust_commit"],
    "native_sha256" => report["native_sha256"],
    "rubocop_version" => report.fetch("rubocop_version"),
    "rubocop_reference" => report["rubocop_reference"],
    "project_count" => report.fetch("projects", {}).length,
    "ruby_files" => report.fetch("projects", {}).values.sum { |project| project.fetch("files", 0) },
    "results" => results.sort.to_h
  }
end

def formatted_json(value)
  "#{JSON.pretty_generate(value)}\n"
end

def update_file(path, content, check:, label:)
  if check
    abort "#{label} is stale: #{path}" unless File.file?(path) && File.read(path) == content
    return
  end

  FileUtils.mkdir_p(File.dirname(path))
  File.write(path, content)
end

if options[:fixture_report]
  imported = fixture_snapshot(
    read_json(options[:fixture_report], "fixture report"),
    options[:fixture_report]
  )
  update_file(
    options[:fixture_snapshot], formatted_json(imported),
    check: options[:check], label: "fixture snapshot"
  )
end

if options[:project_report]
  imported = project_snapshot(
    read_json(options[:project_report], "project report"),
    options[:project_report]
  )
  abort "project report must cover all #{ACTIVE_COPS.length} active cops" unless
    imported.fetch("results").keys.sort == ACTIVE_COPS
  update_file(
    options[:project_snapshot], formatted_json(imported),
    check: options[:check], label: "project snapshot"
  )
end

fixtures = read_json(options[:fixture_snapshot], "fixture snapshot")
projects = read_json(options[:project_snapshot], "project snapshot")
fixture_timestamp = evidence_timestamp(fixtures, "fixture snapshot")
project_timestamp = evidence_timestamp(projects, "project snapshot")
fixture_results = fixtures.fetch("results")
project_results = projects.fetch("results")
cops = (fixture_results.keys | project_results.keys).sort
abort "compatibility evidence must cover the same cops" unless fixture_results.keys.sort == project_results.keys.sort
abort "compatibility evidence must cover the #{ACTIVE_COPS.length} active cops" unless cops == ACTIVE_COPS
abort "fixture and project evidence use different RuboCop versions" unless
  fixtures.fetch("rubocop_version") == projects.fetch("rubocop_version")

rust_files = Dir[File.join(RUST_COP_ROOT, "**", "*.rs")].reject do |path|
  path.include?("/tests/") || path.include?("/framework/") || path.include?("/runtime/")
end
rust_sources = rust_files.to_h { |path| [path, File.read(path)] }

def registration_paths(cop, sources)
  quoted = Regexp.escape(cop)
  patterns = [
    /=>\s*"#{quoted}"\s*=>/m,
    /(?:custom|report|replace)\(\s*"#{quoted}"/m,
    /fn\s+name\s*\([^)]*\)[^{]*\{\s*"#{quoted}"\s*\}/m,
    /let\s+cop\s*=\s*"#{quoted}"/m
  ]
  paths = sources.filter_map { |path, source| path if patterns.any? { |pattern| source.match?(pattern) } }
  return paths.sort unless paths.empty?

  literal = %Q{"#{cop}"}
  paths = sources.filter_map { |path, source| path if source.include?(literal) }
  paths.reject! { |path| path.end_with?("/text/mod.rs") }
  text_paths = paths.select { |path| path.include?("/cops/text/") }
  (text_paths.empty? ? paths : text_paths).sort
end

implementation_paths = cops.to_h do |cop|
  paths = registration_paths(cop, rust_sources)
  abort "could not find implementation for #{cop}" if paths.empty?
  [cop, paths]
end

git_dates = {}
implementation_paths.values.flatten.uniq.each do |path|
  relative = path.delete_prefix("#{ROOT}/")
  stdout, stderr, status = Open3.capture3(
    "git", "log", "-1", "--format=%cI", "--", relative, chdir: ROOT
  )
  abort "could not date implementation #{relative}: #{stderr}" unless status.success?
  git_dates[path] = stdout.strip
end

def changed_implementation_paths(commit)
  return nil if commit.nil? || commit.empty?

  stdout, stderr, status = Open3.capture3(
    "git", "diff", "--name-only", commit, "--", "crates/rustocop/src/cops", chdir: ROOT
  )
  abort "could not compare implementation evidence with #{commit}: #{stderr}" unless status.success?
  stdout.lines.map(&:chomp).to_h { |path| [path, true] }
end

fixture_changes = changed_implementation_paths(fixtures["rust_commit"])
project_changes = changed_implementation_paths(projects["rust_commit"])
fixture_stale = implementation_paths.transform_values do |paths|
  fixture_changes.nil? || paths.any? { |path| fixture_changes[path.delete_prefix("#{ROOT}/")] }
end
project_stale = implementation_paths.transform_values do |paths|
  project_changes.nil? || paths.any? { |path| project_changes[path.delete_prefix("#{ROOT}/")] }
end

def percentage(numerator, denominator)
  return "—" if denominator.zero?

  format("%.1f%%", 100.0 * numerator / denominator)
end

def markdown_paths(paths, dates)
  paths.map do |path|
    relative = path.delete_prefix("#{ROOT}/")
    "[`#{relative}`](../#{relative})"
  end.join("<br>")
end

def latest_date(paths, dates)
  paths.filter_map { |path| dates.fetch(path, "").split("T", 2).first }.max || "—"
end

rows = cops.map do |cop|
  fixture = fixture_results.fetch(cop)
  project = project_results.fetch(cop)
  paths = implementation_paths.fetch(cop)
  passed = fixture.fetch("passed")
  fixture_total = fixture.fetch("total")
  rust = project["rustocop"]
  ruby = project["rubocop"]
  exact = project["exact"]
  union = rust && ruby && exact ? rust + ruby - exact : nil
  project_match = if union&.positive?
                    "#{exact}/#{union} (#{percentage(exact, union)})"
                  elsif union
                    "— (unexercised)"
                  else
                    "— (#{project.fetch('classification')})"
                  end
  fixture_match = "#{passed}/#{fixture_total} (#{percentage(passed, fixture_total)})"
  fixture_match += " ⚠ stale" if fixture_stale.fetch(cop)
  project_match += " ⚠ stale" if project_stale.fetch(cop)
  project_hits = if ruby.nil?
                   "—"
                 elsif project_stale.fetch(cop)
                   "#{ruby} ⚠"
                 else
                   ruby
                 end
  [
    "`#{cop}`",
    markdown_paths(paths, git_dates),
    latest_date(paths, git_dates),
    fixture_total,
    fixture_match,
    project_hits,
    project_match
  ]
end

fixture_cops_hit = fixture_results.count { |_cop, row| row.fetch("total").positive? }
fixture_cases = fixture_results.values.sum { |row| row.fetch("total") }
fixture_matches = fixture_results.values.sum { |row| row.fetch("passed") }
fixture_compatible = fixture_results.count do |cop, row|
  !fixture_stale.fetch(cop) && row.fetch("total").positive? && row.fetch("passed") == row.fetch("total")
end
project_cops_hit = project_results.count do |_cop, row|
  values = row.values_at("rustocop", "rubocop", "exact")
  values.none?(&:nil?) && (values.fetch(0) + values.fetch(1) - values.fetch(2)).positive?
end
project_compatible = project_results.count do |cop, row|
  !project_stale.fetch(cop) && row.fetch("classification") == "project_exact"
end
fully_compatible = cops.count do |cop|
  fixture = fixture_results.fetch(cop)
  !fixture_stale.fetch(cop) && !project_stale.fetch(cop) &&
    fixture.fetch("total").positive? && fixture.fetch("passed") == fixture.fetch("total") &&
    project_results.fetch(cop).fetch("classification") == "project_exact"
end
fixture_current = fixture_stale.count { |_cop, stale| !stale }
project_current = project_stale.count { |_cop, stale| !stale }

generated_at = [fixture_timestamp, project_timestamp].max_by { |value| Time.iso8601(value) }
summary_rows = [
  ["Cops with fixture coverage", fixture_cops_hit, cops.length],
  ["Cops with current fixture evidence", fixture_current, cops.length],
  ["Fixture cases matching", fixture_matches, fixture_cases],
  ["Cops matching every fixture", fixture_compatible, fixture_cops_hit],
  ["Cops exercised on projects", project_cops_hit, cops.length],
  ["Cops with current project evidence", project_current, cops.length],
  ["Project-exact cops among exercised cops", project_compatible, project_cops_hit],
  ["Cops compatible in both evidence sets", fully_compatible, cops.length]
].map do |label, numerator, denominator|
  "| #{label} | #{numerator}/#{denominator} | #{percentage(numerator, denominator)} |"
end

table_rows = rows.map { |row| "| #{row.join(' | ')} |" }
report = <<~MARKDOWN
  # RuboCop compatibility evidence

  Generated at `#{generated_at}` for RuboCop #{fixtures.fetch('rubocop_version')}.
  Compatibility is binary at the cop level: every exercised fixture must match,
  and project output must have no false positives, false negatives, or signature
  differences. Partial overlap is not classified as compatible.

  This table covers #{ACTIVE_COPS.length} active built-in cops. The 48 cops in
  [`intentionally_pending_cops.yml`](../spec/upstream/rubocop-#{fixtures.fetch('rubocop_version')}/intentionally_pending_cops.yml)
  are deliberately unregistered and excluded from both evidence corpora.

  Fixture evidence was updated at `#{fixture_timestamp}`. Project
  evidence was updated at `#{project_timestamp}` from
  #{projects.fetch('project_count')} projects and #{projects.fetch('ruby_files')} Ruby files.
  Fixture source: `#{fixtures.fetch('rust_commit', 'unknown')}`. Project source:
  `#{projects.fetch('rust_commit', 'unknown')}`.

  ## Overall

  | Measure | Result | Percent |
  | --- | ---: | ---: |
  #{summary_rows.join("\n")}

  “Project hits” is the number of RuboCop reference diagnostics. Project matching
  is exact shared signatures divided by the union of Rustocop and RuboCop
  signatures, so both extra and missing diagnostics reduce the percentage. A
  zero-hit row is unexercised, not 100% compatible.

  ## Updating

  Refresh fixture evidence while retaining the existing project columns:

  ```sh
  bundle exec ruby script/generate_compatibility_report.rb --refresh-fixtures
  ```

  Refresh both evidence sets only when the expensive legacy RuboCop project scan
  is intended:

  ```sh
  bundle exec ruby script/generate_compatibility_report.rb \\
    --refresh-fixtures --refresh-projects
  ```

  Without either refresh flag, the generator only renders the checked-in compact
  snapshots. Use `--check` in CI to verify that the table is current.

  A stale marker means one of that cop's implementation files changed after the
  relevant evidence commit. Stale rows remain visible but do not count as
  compatible in the overall totals.

  ## Per-cop evidence

  | Cop | Implementation file | Implementation updated | Fixture tests<br>(as of #{fixture_timestamp}) | Fixture matching<br>(as of #{fixture_timestamp}) | Project hits<br>(as of #{project_timestamp}) | Project matching<br>(as of #{project_timestamp}) |
  | --- | --- | --- | ---: | ---: | ---: | ---: |
  #{table_rows.join("\n")}
MARKDOWN

update_file(options[:output], report, check: options[:check], label: "compatibility report")
puts "Compatibility report: #{options[:output]}"
puts "Fixture evidence: #{fixture_compatible}/#{fixture_cops_hit} cops match every fixture"
puts "Project evidence: #{project_compatible}/#{project_cops_hit} exercised cops are project-exact"
puts "Combined: #{fully_compatible}/#{cops.length} cops satisfy both gates"
