# frozen_string_literal: true

module Rustocop
  module CompatibilityBaseline
    module_function

    def errors(summary, status)
      baseline = status.fetch("diagnostic_baseline")
      problems = []
      compare_exact(problems, "captured cases", status.fetch("captured_cases"), summary.fetch("cases"))
      compare_exact(problems, "total cops", baseline.fetch("total_cops"), summary.fetch("cops"))
      compare_floor(problems, "passing cases", baseline.fetch("passed_cases"), summary.fetch("passed_cases"))
      compare_floor(problems, "passing cops", baseline.fetch("passing_cops"), summary.fetch("passing_cops"))

      results = summary.fetch("results")
      status.fetch("fully_compatible_cops").each do |cop|
        result = results[cop]
        if result.nil?
          problems << "verified cop missing from report: #{cop}"
        elsif result.fetch("status") != "passing"
          problems << "verified cop regressed: #{cop} (#{result.fetch("passed")}/#{result.fetch("total")})"
        end
      end
      problems
    end

    def compare_exact(problems, label, expected, actual)
      return if expected == actual

      problems << "#{label}: expected #{expected}, got #{actual}"
    end

    def compare_floor(problems, label, minimum, actual)
      return if actual >= minimum

      problems << "#{label}: expected at least #{minimum}, got #{actual}"
    end
    private_class_method :compare_exact, :compare_floor
  end
end
