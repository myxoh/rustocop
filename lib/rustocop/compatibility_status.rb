# frozen_string_literal: true

require "yaml"

module Rustocop
  class CompatibilityStatus
    DEFAULT_VERSION = "1.87.0"

    attr_reader :data, :hardening_data, :root, :version

    def self.load(root:, version: DEFAULT_VERSION)
      new(
        root:,
        version:,
        data: YAML.safe_load_file(status_path(root, version)),
        hardening_data: YAML.safe_load_file(hardening_path(root))
      ).tap(&:validate_hardening!)
    end

    def self.status_path(root, version)
      File.join(root, "spec/upstream/rubocop-#{version}/status.yml")
    end

    def self.hardening_path(root)
      File.join(root, "spec/hardening/status.yml")
    end

    def initialize(root:, version:, data:, hardening_data: { "cops" => {} })
      @root = root
      @version = version
      @data = data
      @hardening_data = hardening_data
    end

    def verified_cops
      @verified_cops ||= data.fetch("fully_compatible_cops").freeze
    end

    def heuristic_cops
      @heuristic_cops ||= begin
        path = File.join(root, "spec/upstream/rubocop-#{version}/remaining_cops.yml")
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
  end
end
