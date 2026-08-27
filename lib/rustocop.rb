# frozen_string_literal: true

require_relative "rustocop/artifact_store"
require_relative "rustocop/compatibility_status"
require_relative "rustocop/diagnostic_signatures"
require_relative "rustocop/native_configuration"
require_relative "rustocop/process_runner"
require_relative "rustocop/project_mismatch_inventory"
require_relative "rustocop/repository_layout"
require_relative "rustocop/runner"
require_relative "rustocop/source_fingerprint"
require_relative "rustocop/version"

module Rustocop
  autoload :ConfigurationCompiler, File.expand_path("rustocop/configuration_compiler", __dir__)
  autoload :RubocopConfiguration, File.expand_path("rustocop/rubocop_configuration", __dir__)
end
