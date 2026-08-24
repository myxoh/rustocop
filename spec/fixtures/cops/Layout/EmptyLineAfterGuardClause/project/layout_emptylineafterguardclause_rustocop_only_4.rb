      #  - :skip - do not show the section at all
      # return [Symbol]
      add_read_only_setting :pending_failure_output
      def pending_failure_output=(mode)
        raise ArgumentError,
              "`pending_failure_output` can be set to :full, :no_backtrace, " \
              "or :skip" unless [:full, :no_backtrace, :skip].include?(mode)
        @pending_failure_output = mode
      end
