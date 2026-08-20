# frozen_string_literal: true

require "optparse"
require "rubocop"
require_relative "../lib/rustocop/generated_section"
require_relative "../lib/rustocop/qualification_progress"

ROOT = File.expand_path("..", __dir__)
README = File.join(ROOT, "README.md")
DETAIL = File.join(ROOT, "docs/qualification-progress.md")

check = false
OptionParser.new do |parser|
  parser.on("--check", "fail when generated qualification documentation is stale") { check = true }
end.parse!

rubocop_root = Gem::Specification.find_by_name("rubocop", "1.87.0").full_gem_path
ledger = Rustocop::QualificationProgress.new(root: ROOT, rubocop_root: rubocop_root)
registry = RuboCop::Cop::Registry.global.map(&:cop_name)
department_totals = registry.group_by { |cop| cop.split("/").first }.transform_values(&:length)

percent = lambda { |count| format("%.1f%%", count.fdiv(Rustocop::QualificationProgress::TOTAL_COPS) * 100) }
short_sha = lambda { |sha| "`#{sha.to_s[0, 8]}`" }

check_rows = Rustocop::QualificationProgress::CHECKS.map do |number, name|
  recorded = ledger.recorded_count(number)
  current = ledger.current_count(number)
  "| #{number}. #{name} | #{recorded} / 606 | #{current} / 606 | #{percent.call(current)} |"
end

department_rows = department_totals.sort.map do |department, total|
  records = ledger.records.select { |record| record.fetch("cop").start_with?("#{department}/") }
  recorded = records.count { |record| ledger.evidence_complete?(record) }
  current = records.count { |record| ledger.fully_qualified?(record) }
  stale = records.count { |record| ledger.evidence_complete?(record) && !ledger.source_current?(record) }
  "| `#{department}` | #{current} / #{total} | #{recorded} | #{stale} |"
end

batch_rows = ledger.documents.sort_by { |document| -document.fetch("matrix_end") }.map do |document|
  records = ledger.records.select { |record| record.fetch("batch") == document.fetch("batch") }
  recorded = records.count { |record| ledger.evidence_complete?(record) }
  current = records.count { |record| ledger.source_current?(record) }
  qualified = records.count { |record| ledger.fully_qualified?(record) }
  range = "#{document.fetch("matrix_end")}–#{document.fetch("matrix_start")}"
  "| `#{document.fetch("batch")}` | #{range} | #{recorded} / #{records.length} | " \
    "#{current} / #{records.length} | #{qualified} / #{records.length} | #{short_sha.call(document.fetch("rustocop_commit"))} |"
end

evidence_complete = ledger.evidence_complete_count
qualified = ledger.fully_qualified_count
stale = ledger.stale_records.length
rubocop_commits = ledger.documents.map { |document| document.fetch("rubocop_commit") }.uniq
raise "qualification records use multiple RuboCop commits" unless rubocop_commits.one?

rubocop_commit = rubocop_commits.first
current_rust_commit = ledger.current_rust_commit

readme_body = <<~MARKDOWN
  Qualification restarted from zero on 2026-08-19; the table now reflects the
  authoritative records under `qualification/work/`. "Recorded evidence" means
  a record contains all required evidence. "Current-source credit" additionally
  requires the recorded Rust files to be unchanged from that record's pinned SHA.

  | Check | Recorded evidence | Current-source credit | Current progress |
  | --- | ---: | ---: | ---: |
  #{check_rows.join("\n")}
  | **Fully qualified** | **#{evidence_complete} / 606** | **#{qualified} / 606** | **#{percent.call(qualified)}** |

  #{evidence_complete} cops have complete five-check records. #{stale} of those
  records are currently invalidated by later changes to their Rust source, leaving
  **#{qualified} currently qualified cops**. The RuboCop reference is
  `#{rubocop_commit}`; the current native Rust source is `#{current_rust_commit}`.

  | Department | Currently qualified | Complete records | Stale records |
  | --- | ---: | ---: | ---: |
  #{department_rows.join("\n")}

  See [the detailed qualification ledger](docs/qualification-progress.md) for
  batch totals, every recorded cop, pinned SHAs, and the records needing revalidation.
MARKDOWN

detail_rows = ledger.records.sort_by { |record| -record.fetch("matrix_position") }.map do |record|
  checks = Rustocop::QualificationProgress::CHECKS.map do |number, _name|
    ledger.check_pass?(record, number) ? "✓" : "—"
  end
  source = ledger.source_current?(record) ? "Current" : "**Stale**"
  qualified_state = ledger.fully_qualified?(record) ? "✓" : "—"
  "| #{record.fetch("matrix_position")} | `#{record.fetch("cop")}` | #{checks.join(" | ")} | " \
    "#{source} | #{qualified_state} | `#{record.fetch("batch")}` | #{short_sha.call(record.fetch("rustocop_commit"))} |"
end

stale_rows = ledger.stale_records.sort_by { |record| -record.fetch("matrix_position") }.map do |record|
  changed = Array(record.dig("sources", "rustocop")).map { |path| "`#{path}`" }.join("<br>")
  "| #{record.fetch("matrix_position")} | `#{record.fetch("cop")}` | `#{record.fetch("batch")}` | " \
    "#{short_sha.call(record.fetch("rustocop_commit"))} | #{changed} |"
end

detail_body = <<~MARKDOWN
  # Cop qualification progress

  Generated by `bundle exec ruby script/generate_qualification_progress.rb` from
  the authoritative records in `qualification/work/`.

  ## References

  - RuboCop 1.87.0: `#{rubocop_commit}`
  - Current native Rust source: `#{current_rust_commit}`
  - Complete five-check records: #{evidence_complete} / 606
  - Currently qualified: #{qualified} / 606 (#{percent.call(qualified)})
  - Complete records requiring revalidation: #{stale}

  ## By department

  | Department | Currently qualified | Complete records | Stale records |
  | --- | ---: | ---: | ---: |
  #{department_rows.join("\n")}

  ## By batch

  | Batch | Matrix positions | Complete evidence | Source current | Qualified | Reviewed Rust SHA |
  | --- | ---: | ---: | ---: | ---: | --- |
  #{batch_rows.join("\n")}

  ## Records requiring revalidation

  | Position | Cop | Batch | Reviewed Rust SHA | Changed Rust source |
  | ---: | --- | --- | --- | --- |
  #{stale_rows.empty? ? "| — | None | — | — | — |" : stale_rows.join("\n")}

  ## Per-cop ledger

  A checkmark in columns 1–5 means the recorded evidence is structurally
  complete. A cop receives current qualification credit only when all five are
  complete and its Rust source still matches the reviewed SHA.

  | Position | Cop | 1 | 2 | 3 | 4 | 5 | Rust source | Qualified | Batch | Rust SHA |
  | ---: | --- | :-: | :-: | :-: | :-: | :-: | --- | :-: | --- | --- |
  #{detail_rows.join("\n")}
MARKDOWN

if check
  source = File.read(README)
  match = source.match(/<!-- generated:qualification-progress:start -->\n(.*?)\n<!-- generated:qualification-progress:end -->/m)
  abort "README qualification progress is stale" unless match && match[1] == readme_body.rstrip
  abort "detailed qualification progress is stale" unless File.file?(DETAIL) && File.read(DETAIL) == detail_body
  puts "Qualification progress is current: #{qualified}/606 qualified, #{stale} stale records."
  exit 0
end

Rustocop::GeneratedSection.replace(README, "qualification-progress", readme_body)
File.write(DETAIL, detail_body)
puts "Updated README.md and docs/qualification-progress.md: #{qualified}/606 qualified, #{stale} stale records."
