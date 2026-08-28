# frozen_string_literal: true

require "digest"
require "fileutils"
require "json"
require "pathname"
require "time"

module Rustocop
  class StructuralParity
    FORMAT_VERSION = 1
    RUBRIC_VERSION = 1
    FACETS = %w[callbacks restrictions configuration lifecycle_state helpers predicates traversal offenses corrections mixins].freeze
    STATES = %w[obligations_extracted dossier_ready implementation_submitted review_rejected review_blocked].freeze
    TRANSITIONS = {
      "obligations_extracted" => %w[dossier_ready review_blocked],
      "dossier_ready" => %w[implementation_submitted review_blocked],
      "implementation_submitted" => %w[review_rejected review_blocked],
      "review_rejected" => %w[implementation_submitted review_blocked],
      "review_blocked" => %w[dossier_ready implementation_submitted]
    }.freeze

    def initialize(root:, legacy_manifest: nil)
      @root = File.expand_path(root)
      @base = File.join(@root, "compatibility", "structural")
      @manifest_path = legacy_manifest || File.join(@root, "crates", "rustocop", "rubocop-cop-migrations.json")
    end

    def cops = rows.keys.sort

    def next_cop(role = "advance")
      roles = role == "advance" ? %w[review remediate implement prepare] : [role]
      roles.each do |candidate_role|
        cop = cops.find { |name| eligible?(name, candidate_role) }
        return [candidate_role, cop] if cop
      end
      nil
    end

    def state(cop)
      dossier = optional_json(dossier_path(cop))
      return "legacy_unverified" unless dossier
      attestation = optional_json(attestation_path(cop))
      return dossier.fetch("state") unless attestation
      validate_attestation(cop, attestation: attestation).empty? ? "accepted" : "invalidated"
    end

    def status = cops.group_by { |cop| state(cop) }.transform_values(&:length).sort.to_h

    def dossier_path(cop) = File.join(@base, "dossiers", key(cop))
    def attestation_path(cop) = File.join(@base, "attestations", key(cop))

    def init_dossier(cop)
      row = rows.fetch(cop) { raise ArgumentError, "unknown cop: #{cop}" }
      raise ArgumentError, "dossier already exists" if File.exist?(dossier_path(cop))
      dossier = {
        "format_version" => FORMAT_VERSION, "rubric_version" => RUBRIC_VERSION,
        "cop" => cop, "state" => "obligations_extracted", "updated_at" => Time.now.iso8601,
        "fingerprints" => fingerprints(row),
        "sources" => {
          "upstream" => row.slice("upstream_source", "upstream_sha256", "upstream_callbacks", "upstream_mixins"),
          "rust" => row.fetch("implementations").map { |path| {"path" => path} }
        },
        "facets" => FACETS.to_h { |facet| [facet, {"status" => "unreviewed", "notes" => ""}] },
        "upstream_units" => [], "rust_units" => [], "correspondences" => [],
        "adaptations" => [], "gaps" => [],
        "behavioral_evidence" => row.slice("fixtures", "projects"),
        "legacy_claim" => row.slice("similarity_score", "structural_status", "migration_status", "structural_gaps", "documented_adaptations")
      }
      write_json(dossier_path(cop), dossier)
      dossier_path(cop)
    end

    def transition(cop, target)
      dossier = read_json(dossier_path(cop))
      current = dossier.fetch("state")
      raise ArgumentError, "invalid transition #{current} -> #{target}" unless TRANSITIONS.fetch(current, []).include?(target)
      errors = validate_dossier(cop, dossier: dossier)
      raise ArgumentError, "invalid dossier:\n- #{errors.join("\n- ")}" if %w[dossier_ready implementation_submitted].include?(target) && errors.any?
      dossier["state"] = target
      dossier["updated_at"] = Time.now.iso8601
      write_json(dossier_path(cop), dossier)
    end

    def validate_dossier(cop, dossier: nil, complete: false)
      dossier ||= read_json(dossier_path(cop))
      errors = []
      errors << "wrong format version" unless dossier["format_version"] == FORMAT_VERSION
      errors << "wrong rubric version" unless dossier["rubric_version"] == RUBRIC_VERSION
      errors << "cop name mismatch" unless dossier["cop"] == cop
      errors << "unknown state" unless STATES.include?(dossier["state"])
      errors << "fingerprints are stale" unless dossier["fingerprints"] == fingerprints(rows.fetch(cop))
      FACETS.each do |facet|
        value = dossier.dig("facets", facet)
        errors << "facet #{facet} must be present or not_applicable" unless %w[present not_applicable].include?(value&.dig("status"))
        errors << "facet #{facet} needs an explanation" if value&.dig("status") == "not_applicable" && value["notes"].to_s.strip.empty?
      end
      upstream = validate_units(dossier.fetch("upstream_units", []), "upstream", errors)
      rust = validate_units(dossier.fetch("rust_units", []), "rust", errors)
      covered_upstream = []
      covered_rust = []
      mappings = dossier.fetch("correspondences", [])
      mappings.each_with_index do |mapping, index|
        unless %w[direct justified_adapter missing unexplained_extra].include?(mapping["classification"])
          errors << "mapping #{index} has invalid classification"
        end
        errors << "mapping #{index} needs an id" if mapping["id"].to_s.strip.empty?
        errors << "mapping #{index} needs an invariant" if mapping["invariant"].to_s.strip.empty?
        Array(mapping["upstream_units"]).each { |id| upstream.include?(id) ? covered_upstream << id : errors << "mapping #{index} references unknown upstream unit #{id}" }
        Array(mapping["rust_units"]).each { |id| rust.include?(id) ? covered_rust << id : errors << "mapping #{index} references unknown Rust unit #{id}" }
      end
      mappings.select { |mapping| mapping["classification"] == "justified_adapter" }.each do |mapping|
        adaptation = dossier.fetch("adaptations", []).find { |item| item["mapping_id"] == mapping["id"] }
        errors << "adapter #{mapping['id']} lacks specific evidence" unless valid_adaptation?(adaptation)
      end
      if complete
        errors << "unmapped upstream units: #{(upstream - covered_upstream).join(', ')}" unless (upstream - covered_upstream).empty?
        errors << "unmapped Rust units: #{(rust - covered_rust).join(', ')}" unless (rust - covered_rust).empty?
        errors << "missing or unexplained mappings remain" if mappings.any? { |mapping| %w[missing unexplained_extra].include?(mapping["classification"]) }
        errors << "structural gaps remain" unless dossier.fetch("gaps", []).empty?
      end
      errors
    end

    def attestation_template(cop, reviewer)
      dossier = read_json(dossier_path(cop))
      {
        "format_version" => FORMAT_VERSION, "cop" => cop, "decision" => "accepted",
        "reviewer" => reviewer, "reviewed_at" => Time.now.iso8601,
        "fingerprints" => dossier.fetch("fingerprints"),
        "dossier_sha256" => digest_file(dossier_path(cop)),
        "statement" => "", "findings" => []
      }
    end

    def validate_attestation(cop, attestation: nil)
      attestation ||= read_json(attestation_path(cop))
      dossier = read_json(dossier_path(cop))
      errors = validate_dossier(cop, dossier: dossier, complete: true)
      errors << "only an implementation_submitted dossier can be accepted" unless dossier["state"] == "implementation_submitted"
      errors << "wrong attestation format version" unless attestation["format_version"] == FORMAT_VERSION
      errors << "attestation cop mismatch" unless attestation["cop"] == cop
      errors << "decision must be accepted" unless attestation["decision"] == "accepted"
      errors << "reviewer is required" if attestation["reviewer"].to_s.strip.empty?
      errors << "review statement is required" if attestation["statement"].to_s.strip.empty?
      errors << "attestation fingerprints are stale" unless attestation["fingerprints"] == dossier["fingerprints"]
      errors << "attestation dossier digest is stale" unless attestation["dossier_sha256"] == digest_file(dossier_path(cop))
      errors
    end

    private

    def eligible?(cop, role)
      case role
      when "prepare" then %w[legacy_unverified obligations_extracted].include?(state(cop))
      when "implement" then %w[dossier_ready review_rejected].include?(state(cop))
      when "review" then state(cop) == "implementation_submitted"
      when "remediate" then state(cop) == "review_rejected"
      when "audit" then state(cop) == "accepted"
      else raise ArgumentError, "unknown role: #{role}"
      end
    end

    def rows
      @rows ||= read_json(@manifest_path).fetch("cops").to_h { |row| [row.fetch("cop"), row] }
    end

    def fingerprints(row)
      files = row.fetch("implementations").map { |path| File.join(@root, "crates", "rustocop", path) }
      framework = Dir[File.join(@root, "crates", "rustocop", "src", "cops", "prism", "framework", "**", "*.rs")]
      result = {
        "upstream_sha256" => row.fetch("upstream_sha256"),
        "rust_sha256" => combined_digest(files),
        "shared_runtime_sha256" => combined_digest(framework),
        "behavioral_evidence_sha256" => Digest::SHA256.hexdigest(JSON.generate(row.slice("fixtures", "projects"))),
        "standard_sha256" => digest_file(File.join(@base, "standard.md"))
      }
      dependencies = Array(row["upstream_dependencies"])
      result["upstream_dependencies_sha256"] = Digest::SHA256.hexdigest(JSON.generate(dependencies)) if dependencies.any?
      result
    end

    def validate_units(units, side, errors)
      ids = units.map { |unit| unit["id"] }
      errors << "duplicate #{side} unit IDs" unless ids.compact.uniq.length == ids.length
      units.each_with_index do |unit, index|
        errors << "#{side} unit #{index} needs id" if unit["id"].to_s.strip.empty?
        errors << "#{side} unit #{index} has invalid facet" unless FACETS.include?(unit["facet"])
        errors << "#{side} unit #{index} needs path" if unit["path"].to_s.strip.empty?
        errors << "#{side} unit #{index} needs start_line" unless unit["start_line"].to_i.positive?
        errors << "#{side} unit #{index} needs end_line" unless unit["end_line"].to_i >= unit["start_line"].to_i
        errors << "#{side} unit #{index} needs responsibility" if unit["responsibility"].to_s.strip.empty?
      end
      ids.compact
    end

    def valid_adaptation?(item)
      item && %w[necessity invariant evidence].all? { |field| item[field].is_a?(Array) ? item[field].any? : !item[field].to_s.strip.empty? }
    end

    def combined_digest(paths)
      digest = Digest::SHA256.new
      paths.sort.each { |path| digest << path.delete_prefix(@root) << File.binread(path) }
      digest.hexdigest
    end

    def key(cop) = "#{cop.tr('/', '__')}.json"
    def digest_file(path) = Digest::SHA256.file(path).hexdigest
    def read_json(path) = JSON.parse(File.read(path))
    def optional_json(path)
      read_json(path) if File.file?(path)
    end
    def write_json(path, value)
      FileUtils.mkdir_p(File.dirname(path))
      File.write(path, "#{JSON.pretty_generate(value)}\n")
    end
  end
end
