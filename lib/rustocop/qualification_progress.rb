# frozen_string_literal: true

require "open3"
require "yaml"

module Rustocop
  # Reads the qualification ledger and separates recorded evidence from evidence
  # that still applies to the current native source tree.
  class QualificationProgress
    TOTAL_COPS = 606
    CHECKS = [
      [1, "Manual source verification"],
      [2, "Ported upstream unit tests"],
      [3, "Edge-case fixtures"],
      [4, "Real-world true positives"],
      [5, "Real-world true negatives"]
    ].freeze

    attr_reader :documents, :records

    def initialize(root:, rubocop_root:, source_current: nil)
      @root = root
      @rubocop_root = rubocop_root
      @source_current = source_current || method(:git_source_current?)
      @documents = load_documents
      @records = load_records
    end

    def recorded_count(check)
      records.count { |record| check_pass?(record, check) }
    end

    def current_count(check)
      records.count { |record| check_pass?(record, check) && source_current?(record) }
    end

    def evidence_complete?(record)
      CHECKS.all? { |check, _name| check_pass?(record, check) }
    end

    def fully_qualified?(record)
      evidence_complete?(record) && source_current?(record)
    end

    def evidence_complete_count
      records.count { |record| evidence_complete?(record) }
    end

    def fully_qualified_count
      records.count { |record| fully_qualified?(record) }
    end

    def stale_records
      records.select { |record| evidence_complete?(record) && !source_current?(record) }
    end

    def current_rust_commit
      output, error, status = Open3.capture3(
        "git", "log", "-1", "--format=%H", "--", "crates/rustocop", chdir: @root
      )
      raise "could not determine current Rust source commit: #{error}" unless status.success?

      output.strip
    end

    def check_pass?(record, check)
      case check
      when 1 then manual_pass?(record)
      when 2 then upstream_pass?(record)
      when 3 then edge_cases_pass?(record)
      when 4 then real_world_pass?(record, "positives")
      when 5 then real_world_pass?(record, "negatives")
      else raise ArgumentError, "unknown qualification check #{check}"
      end
    end

    def source_current?(record)
      return record["source_current"] if record.key?("source_current")

      record["source_current"] = @source_current.call(record)
    end

    private

    def load_documents
      paths = Dir[File.join(@root, "qualification/work/*.yml")].sort
      raise "no qualification work records found" if paths.empty?

      paths.map do |path|
        document = YAML.safe_load_file(path)
        raise "unsupported qualification schema in #{path}" unless document["schema"] == 1

        document.merge("record_file" => path)
      end
    end

    def load_records
      seen = {}
      documents.flat_map do |document|
        document.fetch("cops").map do |cop, record|
          raise "duplicate qualification record for #{cop}" if seen[cop]

          seen[cop] = true
          record.merge(
            "cop" => cop,
            "batch" => document.fetch("batch"),
            "rubocop_commit" => document.fetch("rubocop_commit"),
            "rustocop_commit" => document.fetch("rustocop_commit"),
            "record_file" => document.fetch("record_file")
          )
        end
      end
    end

    def manual_pass?(record)
      review = record.fetch("manual_review", {})
      ruby_source = File.join(@rubocop_root, record.dig("sources", "rubocop").to_s)
      rust_sources = Array(record.dig("sources", "rustocop"))
      review["status"] == "passed" && Array(review["notes"]).length >= 2 &&
        File.file?(ruby_source) && !rust_sources.empty? &&
        rust_sources.all? { |path| File.file?(File.join(@root, path)) }
    end

    def upstream_pass?(record)
      upstream = record.fetch("upstream_tests", {})
      upstream["status"] == "passed" && upstream["corrections"] == true &&
        upstream["total"].to_i.positive? && upstream["passed"] == upstream["total"]
    end

    def edge_cases_pass?(record)
      cases = Array(record["edge_cases"])
      cases.length >= 4 && cases.map { |item| item["id"] }.uniq.length == cases.length
    end

    def real_world_pass?(record, kind)
      examples = Array(record.dig("real_world", kind))
      origins = examples.map { |item| [item["repository"], item["revision"], item["path"], item["line"]] }
      examples.length >= 2 && origins.uniq.length == examples.length && examples.all? do |item|
        item["repository"].to_s.match?(%r{\A[^/]+/[^/]+\z}) &&
          item["revision"].to_s.match?(/\A[0-9a-f]{40}\z/) &&
          !item["path"].to_s.empty? && item["line"].to_i.positive? && !item["source"].to_s.empty?
      end
    end

    def git_source_current?(record)
      commit = record.fetch("rustocop_commit").to_s
      paths = Array(record.dig("sources", "rustocop"))
      return false unless commit.match?(/\A[0-9a-f]{40}\z/) && !paths.empty?

      _output, _error, exists = Open3.capture3("git", "cat-file", "-e", "#{commit}^{commit}", chdir: @root)
      return false unless exists.success?

      _output, _error, unchanged = Open3.capture3(
        "git", "diff", "--quiet", commit, "--", *paths, chdir: @root
      )
      unchanged.success?
    end
  end
end
