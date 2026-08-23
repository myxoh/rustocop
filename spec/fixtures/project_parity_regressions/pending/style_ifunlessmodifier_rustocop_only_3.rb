    described_class.new(shutdown_timeout_seconds, sleep_time).tap do |instance|
      # We need to defuse `sleep` and stop the  handler after n iteration
      iterations = 0
      allow(instance).to receive(:sleep) do
        if (iterations += 1) > handler_iterations
          instance.stop
        end
      end
    end
