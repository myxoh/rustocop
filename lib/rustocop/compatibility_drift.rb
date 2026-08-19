# frozen_string_literal: true

require "json"

module Rustocop
  module CompatibilityDrift
    module_function

    def analyze(report, status, correction_contracts: {})
      results = report.fetch("results")
      passing = results.filter_map { |cop, result| cop if result.fetch("status") == "passing" }
      verified = status.verified_cops
      {
        "passing_not_promoted" => (passing - verified).sort,
        "verified_regressions" => verified.select do |cop|
          !results.key?(cop) || results.dig(cop, "status") != "passing"
        end.sort,
        "passing_without_correction_assertions" => passing.select do |cop|
          contract = correction_contracts.fetch(cop, {})
          contract.fetch("correctable_cases", 0).positive? && contract.fetch("assertions", 0).zero?
        end.sort
      }
    end

    def correction_contracts(corpus_path)
      contracts = Hash.new { |hash, cop| hash[cop] = { "correctable_cases" => 0, "assertions" => 0 } }
      File.foreach(corpus_path) do |line|
        test_case = JSON.parse(line)
        contract = contracts[test_case.fetch("cop")]
        offenses = test_case.fetch("offenses", []) || []
        contract["correctable_cases"] += 1 if offenses.any? { |offense| offense.fetch("correctable", false) }
        contract["assertions"] += 1 if test_case.key?("correction")
      end
      contracts
    end
  end
end
