# frozen_string_literal: true

require "yaml"

module Rustocop
  module ConfigSerialization
    RUBY_SYMBOL_MAPS = %w[InverseMethods InverseBlocks].freeze

    module_function

    def rubocop_yaml(config)
      YAML.dump(restore_ruby_symbol_maps(config))
    end

    def restore_ruby_symbol_maps(value, parent_key = nil)
      case value
      when Hash
        symbolize = RUBY_SYMBOL_MAPS.include?(parent_key)
        value.to_h do |key, child|
          rendered_key = symbolize ? key.to_sym : key
          rendered_child = if symbolize && child.respond_to?(:to_sym)
                             child.to_sym
                           else
                             restore_ruby_symbol_maps(child, key.to_s)
                           end
          [rendered_key, rendered_child]
        end
      when Array
        value.map { |child| restore_ruby_symbol_maps(child, parent_key) }
      else
        value
      end
    end
  end
end
