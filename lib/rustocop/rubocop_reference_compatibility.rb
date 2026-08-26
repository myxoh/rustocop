# frozen_string_literal: true

# Narrow compatibility fixes for the pinned RuboCop reference engine. These
# keep project-parity evidence runnable without changing the gem installation.
require "rubocop"
require "rubocop/cop/style/class_and_module_children"
require "rubocop/cop/style/file_write"
require "rubocop/options"

module Rustocop
  module RubocopReferenceCompatibility
    module ClassAndModuleChildren
      private

      def replace_namespace_keyword(corrector, node)
        sibling = node.left_sibling
        class_definition = if sibling.respond_to?(:each_node)
                             sibling.each_node(:class).find do |class_node|
                               class_node.identifier == node.identifier.namespace
                             end
                           end
        namespace_keyword = class_definition ? "class" : "module"

        corrector.replace(node.loc.keyword, namespace_keyword)
      end
    end

    module FileWrite
      private

      def find_heredoc(node)
        return unless node

        super
      end
    end

    module OptionsValidator
      private

      # Project parity deliberately selects one cop at a time. RuboCop blocks
      # this cop at the CLI boundary even though its internal orchestration can
      # still provide deterministic reference behavior for that selection.
      def only_includes_redundant_disable?
        false
      end
    end
  end
end

RuboCop::Cop::Style::ClassAndModuleChildren.prepend(
  Rustocop::RubocopReferenceCompatibility::ClassAndModuleChildren
)
RuboCop::Cop::Style::FileWrite.prepend(
  Rustocop::RubocopReferenceCompatibility::FileWrite
)
RuboCop::OptionsValidator.prepend(Rustocop::RubocopReferenceCompatibility::OptionsValidator)
