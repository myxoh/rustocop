# frozen_string_literal: true

require_relative "../lib/rustocop/compatibility_status"

root = File.expand_path("..", __dir__)
status = Rustocop::CompatibilityStatus.load(root: root)

puts "Hardening contracts passed: #{status.hardened_cops.length} hardened cops."
