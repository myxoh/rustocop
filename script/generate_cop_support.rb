# frozen_string_literal: true

require "rbconfig"

generator = File.expand_path("generate_project_parity_docs.rb", __dir__)
warn "generate_cop_support.rb now uses complete project-parity evidence"
exec(RbConfig.ruby, generator, *ARGV)
