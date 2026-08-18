# frozen_string_literal: true

# Run the vendored RuboCop cop specs against the exact RuboCop version in the
# bundle without loading RuboCop's development-only spec dependencies.
ENV["RUBOCOP_CORE_DEVELOPMENT"] = "true"

require "rubocop"
require "rubocop/cop/internal_affairs"
require "rubocop/rspec/support"
require_relative "../upstream/rubocop-1.87.0/spec/core_ext/string"

if ENV["RUSTOCOP_UPSTREAM_CAPTURE"]
  require_relative "upstream_cop_capture"
  RuboCop::RSpec::ExpectOffense.prepend(UpstreamCopCapture)
end

upstream_support = File.expand_path("../upstream/rubocop-1.87.0/spec/support", __dir__)
unused_helpers = %w[lsp_helper.rb mcp_helper.rb strict_warnings.rb]
Dir[File.join(upstream_support, "**/*.rb")].sort.each do |path|
  require path unless unused_helpers.include?(File.basename(path))
end

RSpec.configure do |config|
  config.order = :defined
  config.disable_monkey_patching!
  config.after do |example|
    rustocop_flush_capture(example) if respond_to?(:rustocop_flush_capture)
  end
end
