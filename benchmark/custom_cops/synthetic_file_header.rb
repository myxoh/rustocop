# frozen_string_literal: true

module RuboCop
  module Cop
    module Custom
      # A deliberately tiny custom cop used to measure delegation overhead.
      class SyntheticFileHeader < Base
        MSG = "Synthetic custom-cop offense."

        def on_new_investigation
          return if processed_source.raw_source.empty?

          add_offense(Parser::Source::Range.new(processed_source.buffer, 0, 1))
        end
      end
    end
  end
end
