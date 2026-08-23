      delegate :pubsub, :executor, :config, :broadcast, to: :server
      delegate :env, :request, :protocol, :perform_work, to: :socket, allow_nil: true

      def initialize(server, socket)
        @server = server
        @socket = socket

        @logger = socket.logger
        @subscriptions  = Subscriptions.new(self)

        @_internal_subscriptions = nil

        @started_at = Time.now
      end

      # This method is called every time an Action Cable client establishes an underlying connection.
      # Override it in your class to define authentication logic and
