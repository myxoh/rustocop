module Cells
  module Mailroom
    class Processor
      def initialize(wildcard_address:, gitlab_host:, forwarder:,
        logger:)
        @options = {
          wildcard_address: wildcard_address,
          gitlab_host: gitlab_host,
          forwarder: forwarder,
          logger: logger,
        }
      end

      def ready(first,
                second)
        [first, second]
      end

      def one_line(first, second)
        [first, second]
      end
    end
  end
end
