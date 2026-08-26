# frozen_string_literal: true

require "yaml"
require_relative "repository_layout"

module Rustocop
  class CompatibilityStatus
    DEFAULT_VERSION = "1.87.0"

    attr_reader :data, :hardening_data, :pending_data, :root, :version

    def self.load(root:, version: DEFAULT_VERSION)
      new(
        root:,
        version:,
        data: YAML.safe_load_file(status_path(root, version)),
        hardening_data: YAML.safe_load_file(hardening_path(root)),
        pending_data: YAML.safe_load_file(pending_path(root, version))
      ).tap(&:validate!)
    end

    def self.status_path(root, version)
      RepositoryLayout.new(root).upstream(version, "status.yml")
    end

    def self.hardening_path(root)
      RepositoryLayout.new(root).path("spec", "hardening", "status.yml")
    end

    def self.pending_path(root, version)
      RepositoryLayout.new(root).upstream(version, "intentionally_pending_cops.yml")
    end

    def initialize(root:, version:, data:, hardening_data: { "cops" => {} }, pending_data: nil)
      @root = root
      @version = version
      @data = data
      @hardening_data = hardening_data
      @pending_data = pending_data || {
        "version" => 1,
        "rubocop_version" => version,
        "cops" => []
      }
    end

    def verified_cops
      @verified_cops ||= data.fetch("fully_compatible_cops").freeze
    end

    def heuristic_cops
      @heuristic_cops ||= begin
        path = RepositoryLayout.new(root).upstream(version, "remaining_cops.yml")
        YAML.safe_load_file(path).fetch("cops").filter_map do |entry|
          entry.fetch("cop") if entry.fetch("state") == "heuristic"
        end.freeze
      end
    end

    def hardening_entries
      hardening_data.fetch("cops")
    end

    def hardened_cops
      @hardened_cops ||= hardening_entries.keys.freeze
    end

    def intentionally_pending_cops
      @intentionally_pending_cops ||= pending_data.fetch("cops").freeze
    end

    def intentionally_pending?(cop)
      intentionally_pending_cops.include?(cop)
    end

    def built_in_cops
      @built_in_cops ||= (verified_cops + heuristic_cops).uniq.freeze
    end

    def verified?(cop)
      verified_cops.include?(cop)
    end

    def heuristic?(cop)
      heuristic_cops.include?(cop)
    end

    def hardened?(cop)
      hardened_cops.include?(cop)
    end

    def validate_verified!(cops, label: "cop list")
      unverified = cops.uniq.reject { |cop| verified?(cop) }
      return if unverified.empty?

      raise ArgumentError, "#{label} contains non-verified cops: #{unverified.join(", ")}"
    end

    def validate_hardening!
      unless hardening_data.fetch("version") == 1
        raise ArgumentError, "unsupported hardening manifest version"
      end
      unless hardening_data.fetch("rubocop_version").to_s == version
        actual = hardening_data.fetch("rubocop_version")
        raise ArgumentError, "hardening manifest targets RuboCop #{actual}, expected #{version}"
      end
      validate_verified!(hardened_cops, label: "hardening manifest")
      required = hardening_data.fetch("required_categories").sort
      hardening_entries.each do |cop, evidence|
        missing = required - evidence.fetch("categories").sort
        raise ArgumentError, "#{cop} lacks hardening categories: #{missing.join(", ")}" unless missing.empty?

        paths = [evidence.fetch("fixture"), *evidence.fetch("evidence")].uniq
        absent = paths.reject { |path| File.file?(File.join(root, path)) }
        raise ArgumentError, "#{cop} has missing hardening evidence: #{absent.join(", ")}" unless absent.empty?
      end
      true
    end

    def validate_pending!
      unless pending_data.fetch("version") == 1
        raise ArgumentError, "unsupported intentionally-pending manifest version"
      end
      unless pending_data.fetch("rubocop_version").to_s == version
        actual = pending_data.fetch("rubocop_version")
        raise ArgumentError, "intentionally-pending manifest targets RuboCop #{actual}, expected #{version}"
      end
      unless intentionally_pending_cops == intentionally_pending_cops.sort.uniq
        raise ArgumentError, "intentionally-pending cops must be sorted and unique"
      end

      active = verified_cops | heuristic_cops
      overlap = intentionally_pending_cops & active
      unless overlap.empty?
        raise ArgumentError, "intentionally-pending cops remain active: #{overlap.join(', ')}"
      end
      true
    end

    def validate!
      validate_hardening!
      validate_pending!
    end
  end
end
