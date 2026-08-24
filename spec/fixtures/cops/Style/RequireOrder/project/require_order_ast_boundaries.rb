require 'rainbow'

require_relative '../qa'

require_relative 'scenario_shared_examples'
require_relative('../../jh/qa/spec/spec_helper') if GitlabEdition.jh?

frameworks = <<~RUBY
  require "rails"
  require "active_model/railtie"
RUBY

example = "
  require './lib/lib_mod'
  require './app/app_mod'
"

begin
  require 'mocha/api'
rescue LoadError
  require 'mocha/standalone'
  require 'mocha/object'
end
