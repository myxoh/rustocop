#!/usr/bin/env ruby
# frozen_string_literal: true

require "json"
require_relative "../lib/rustocop/structural_parity"

workflow = Rustocop::StructuralParity.new(root: File.expand_path("..", __dir__))
command = ARGV.shift || "status"

case command
when "status"
  puts JSON.pretty_generate(workflow.status)
when "next"
  result = workflow.next_cop(ARGV.shift || "advance")
  abort "No eligible cop" unless result
  puts({"role" => result[0], "cop" => result[1]}.to_json)
when "init"
  cop = ARGV.shift or abort "Usage: structural_parity.rb init COP"
  puts workflow.init_dossier(cop)
when "transition"
  cop, target = ARGV.shift(2)
  abort "Usage: structural_parity.rb transition COP STATE" unless cop && target
  workflow.transition(cop, target)
  puts "#{cop}: #{workflow.state(cop)}"
when "validate"
  cop = ARGV.shift or abort "Usage: structural_parity.rb validate COP"
  errors = workflow.state(cop) == "accepted" ? workflow.validate_attestation(cop) : workflow.validate_dossier(cop)
  abort errors.join("\n") unless errors.empty?
  puts "#{cop}: valid"
when "attestation-template"
  cop, reviewer = ARGV.shift(2)
  abort "Usage: structural_parity.rb attestation-template COP REVIEWER" unless cop && reviewer
  puts JSON.pretty_generate(workflow.attestation_template(cop, reviewer))
when "check"
  invalid = workflow.cops.select { |cop| workflow.state(cop) == "invalidated" }
  abort "Invalidated attestations: #{invalid.join(', ')}" unless invalid.empty?
  puts JSON.generate(workflow.status)
else
  abort "Unknown command: #{command}"
end

