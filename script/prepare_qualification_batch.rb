# frozen_string_literal: true

require "fileutils"
require "json"
require "open3"
require "optparse"
require "rbconfig"
require "rubocop"
require "yaml"
require_relative "../lib/rustocop/config_serialization"
require_relative "../lib/rustocop/qualification_batch"

ROOT = File.expand_path("..", __dir__)
NATIVE = File.join(ROOT, "crates/rustocop/target/debug/rustocop")
DEFAULT_CORPUS = File.join(ROOT, "tmp/rubocop-1.87.0-cop-cases.jsonl")
DEFAULT_OUTPUT = File.join(ROOT, "tmp/qualification/prepared-batch.yml")

options = {
  cops: nil,
  count: 10,
  from_position: nil,
  corpus: DEFAULT_CORPUS,
  output: DEFAULT_OUTPUT,
  jobs: 8,
  real_world: true,
  verify_upstream: true,
  dry_run: false
}

OptionParser.new do |parser|
  parser.banner = "Usage: ruby script/prepare_qualification_batch.rb [options]"
  parser.on("--cops NAMES", "comma-separated cop names") { |value| options[:cops] = value.split(",").map(&:strip) }
  parser.on("--count COUNT", Integer, "number of unrecorded reverse-order cops (default: 10)") { |value| options[:count] = value }
  parser.on("--from-position POSITION", Integer, "start at this one-based sorted matrix position") { |value| options[:from_position] = value }
  parser.on("--corpus PATH") { |value| options[:corpus] = File.expand_path(value) }
  parser.on("--output PATH") { |value| options[:output] = File.expand_path(value) }
  parser.on("--jobs COUNT", Integer) { |value| options[:jobs] = value }
  parser.on("--[no-]real-world", "scan pinned projects for real-world candidates") { |value| options[:real_world] = value }
  parser.on("--[no-]verify-upstream", "run the complete captured contract first") { |value| options[:verify_upstream] = value }
  parser.on("--dry-run", "print the pending record without builds, scans, or writes") { options[:dry_run] = true }
end.parse!

abort "captured corpus not found: #{options[:corpus]}" unless File.file?(options[:corpus])

matrix = RuboCop::Cop::Registry.global.map(&:cop_name).sort
existing = Dir[File.join(ROOT, "qualification/work/*.yml")].each_with_object({}) do |path, records|
  document = YAML.safe_load_file(path)
  document.fetch("cops", {}).each do |cop, record|
    records[cop] = record.merge(
      "record_file" => path,
      "rustocop_commit" => document.fetch("rustocop_commit", "pending")
    )
  end
end

cops = options[:cops]
unless cops
  start = options[:from_position] || matrix.length
  cops = start.downto(1).map { |position| matrix.fetch(position - 1) }
              .reject { |cop| existing.key?(cop) }.first(options[:count])
end
unknown = cops - matrix
abort "unknown cops: #{unknown.join(', ')}" unless unknown.empty?
abort "no cops selected" if cops.empty?

rust_status, status_error, status_ok = Open3.capture3(
  "git", "status", "--porcelain", "--", "crates/rustocop", chdir: ROOT
)
abort "could not inspect native Rust source: #{status_error}" unless status_ok.success?
abort "native Rust source has uncommitted changes; commit it before preparing qualification evidence" unless rust_status.empty?
rust_head, git_error, git_status = Open3.capture3(
  "git", "log", "-1", "--format=%H", "--", "crates/rustocop", chdir: ROOT
)
abort "could not determine the native Rust source commit: #{git_error}" unless git_status.success?
rust_commit = rust_head.strip
rubocop_root = Gem::Specification.find_by_name("rubocop", Rustocop::QualificationBatch::RUBOCOP_VERSION).full_gem_path
corpus = Rustocop::QualificationBatch::Corpus.new(options[:corpus])
cases = corpus.cases_for(cops)
missing_cases = cases.select { |_cop, items| items.empty? }.keys
abort "captured corpus has no cases for: #{missing_cases.join(', ')}" unless missing_cases.empty?

inventory_reader = Rustocop::QualificationBatch::SourceInventory.new(root: ROOT, rubocop_root: rubocop_root)
inventories = cops.to_h { |cop| [cop, inventory_reader.for(cop)] }
upstream_results = cops.to_h { |cop| [cop, { "passed" => 0, "total" => cases.fetch(cop).length, "status" => "pending" }] }

unless options[:dry_run] || !options[:verify_upstream]
  build_stdout, build_stderr, built = Open3.capture3("cargo", "build", "--manifest-path", File.join(ROOT, "crates/rustocop/Cargo.toml"))
  abort "Rust build failed:\n#{build_stdout}\n#{build_stderr}" unless built.success?

  report_path = File.join(ROOT, "tmp/qualification/prepared-upstream.json")
  FileUtils.mkdir_p(File.dirname(report_path))
  command = [RbConfig.ruby, File.join(ROOT, "script/compare_upstream_cop_specs.rb"),
             "--only", cops.join(","), "--corrections", "--jobs", options[:jobs].to_s,
             "--corpus", options[:corpus], "--report", report_path]
  _stdout, stderr, status = Open3.capture3({ "RUSTOCOP_NATIVE_PATH" => NATIVE }, *command, chdir: ROOT)
  abort "upstream comparison did not produce a report: #{stderr}" unless File.file?(report_path)
  report = JSON.parse(File.read(report_path))
  cops.each do |cop|
    result = report.fetch("results").fetch(cop)
    upstream_results[cop] = result.slice("passed", "total").merge("status" => result.fetch("status"))
  end
  warn stderr unless status.success? || stderr.empty?
end

real_candidates = { "positives" => {}, "negatives" => {} }
if options[:real_world] && !options[:dry_run]
  project_root = File.join(ROOT, "tmp/project-benchmarks/sources")
  projects = Rustocop::QualificationBatch::PROJECTS.map do |project|
    source_root = File.join(project_root, "#{project.fetch("name")}-#{project.fetch("revision")}")
    abort "pinned project corpus missing: #{source_root}; run script/benchmark_projects.rb once" unless File.directory?(source_root)
    project.merge("source_root" => source_root)
  end
  config_path = File.join(ROOT, "tmp/qualification/project-rubocop.yml")
  FileUtils.mkdir_p(File.dirname(config_path))
  File.write(config_path, Rustocop::ConfigSerialization.rubocop_yaml(
    "AllCops" => {
      "DisabledByDefault" => true,
      "Exclude" => [],
      "NewCops" => "disable",
      "SuggestExtensions" => false,
      "TargetRubyVersion" => 3.4
    }
  ))
  rubocop = [RbConfig.ruby, Gem.bin_path("rubocop", "rubocop")]
  markers = cops.to_h do |cop|
    [cop, inventory_reader.markers(cop, cases.fetch(cop), inventories.fetch(cop))]
  end
  scanner = Rustocop::QualificationBatch::ProjectScanner.new(
    root: ROOT,
    projects: projects,
    rubocop: rubocop,
    config_path: config_path,
    cache_root: File.join(ROOT, "tmp/qualification/project-scans")
  )
  scanned = scanner.candidates(cops: cops, markers: markers)
  verifier = Rustocop::QualificationBatch::DifferentialVerifier.new(
    rubocop: rubocop,
    rustocop: NATIVE,
    cache_root: File.join(ROOT, "tmp/qualification/cache"),
    jobs: options[:jobs]
  )
  cops.each do |cop|
    real_candidates["positives"][cop] = verifier.filter(
      cop, scanned.fetch("positives").fetch(cop, []), positive: true
    )
    real_candidates["negatives"][cop] = verifier.filter(
      cop, scanned.fetch("negatives").fetch(cop, []), positive: false
    )
  end
end

records = cops.each_with_object({}) do |cop, output|
  inventory = inventories.fetch(cop)
  previous = existing[cop]
  rust_state = if previous.nil?
                 "new"
               else
                 old_commit = previous.fetch("rustocop_commit", "pending")
                 paths = Array(previous.dig("sources", "rustocop"))
                 if old_commit.match?(/\A[0-9a-f]{40}\z/) && !paths.empty?
                   _stdout, _stderr, unchanged = Open3.capture3(
                     "git", "diff", "--quiet", old_commit, rust_commit, "--", *paths, chdir: ROOT
                   )
                   unchanged.success? ? "unchanged" : "changed_since_#{old_commit[0, 8]}"
                 else
                   "unfinalized"
                 end
               end
  upstream = upstream_results.fetch(cop)
  action = upstream.fetch("status") == "passing" ? "audit existing implementation" :
           upstream.fetch("status") == "failing" ? "convert failing implementation to Ruby-shaped callbacks, then rerun" :
           "run upstream validation"
  output[cop] = {
    "matrix_position" => matrix.index(cop) + 1,
    "sources" => inventory.slice("rubocop", "rustocop"),
    "manual_review" => {
      "status" => "pending",
      "notes" => [
        "TODO: compare RuboCop callbacks and semantic branches with the Rust implementation.",
        "TODO: compare offense ranges, messages, configuration, and autocorrection behavior."
      ]
    },
    "upstream_tests" => {
      "status" => upstream.fetch("status") == "passing" ? "passed" : upstream.fetch("status"),
      "passed" => upstream.fetch("passed"),
      "total" => upstream.fetch("total"),
      "corrections" => upstream.fetch("status") == "passing"
    },
    "edge_cases" => corpus.select_edges(cases.fetch(cop)),
    "real_world" => {
      "positives" => real_candidates.fetch("positives").fetch(cop, []),
      "negatives" => real_candidates.fetch("negatives").fetch(cop, [])
    },
    "preparation" => {
      "action" => action,
      "rust_source_state" => rust_state,
      "internals" => {
        "ruby" => inventory.fetch("ruby_internals"),
        "rust" => inventory.fetch("rust_internals")
      }
    }
  }
end

positions = records.values.map { |record| record.fetch("matrix_position") }
document = {
  "schema" => 1,
  "batch" => "prepared_#{positions.max}_#{positions.min}",
  "matrix_order" => "descending",
  "matrix_start" => positions.min,
  "matrix_end" => positions.max,
  "rubocop_version" => Rustocop::QualificationBatch::RUBOCOP_VERSION,
  "rubocop_commit" => Rustocop::QualificationBatch::RUBOCOP_COMMIT,
  "rustocop_commit" => rust_commit,
  "cops" => records
}

yaml = YAML.dump(document)
if options[:dry_run]
  puts yaml
  exit 0
end

FileUtils.mkdir_p(File.dirname(options[:output]))
File.write(options[:output], yaml)
markdown_path = options[:output].sub(/\.ya?ml\z/, ".md")
File.write(markdown_path, Rustocop::QualificationBatch::ReviewPacket.new.render(document))
puts "Prepared #{cops.length} cops:"
puts "  #{options[:output]}"
puts "  #{markdown_path}"
records.each do |cop, record|
  puts "  #{cop}: #{record.dig("upstream_tests", "passed")}/#{record.dig("upstream_tests", "total")} upstream, " \
       "#{record.dig("real_world", "positives").length}/2 positives, " \
       "#{record.dig("real_world", "negatives").length}/2 negatives"
end
