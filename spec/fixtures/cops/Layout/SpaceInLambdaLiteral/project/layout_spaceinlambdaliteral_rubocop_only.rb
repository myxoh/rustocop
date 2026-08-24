
      # Unsubscribes all streams associated with this channel from the pubsub queue.
      def stop_all_streams
        streams.each do |broadcasting, callback|
          pubsub.unsubscribe broadcasting, callback
          logger.info "#{self.class.name} stopped streaming from #{broadcasting}"
        end.clear
      end

      # Calls stream_for with the given `model` if it's present to start streaming,
      # otherwise rejects the subscription.
      def stream_or_reject_for(model)
        if model
          stream_for model
        else
          reject
        end
      end

      private
        delegate :pubsub, to: :connection

        def streams
          @_streams ||= {}
        end

        # Always wrap the outermost handler to invoke the user handler on the worker
        # pool rather than blocking the event loop.
        def worker_pool_stream_handler(broadcasting, user_handler, coder: nil)
          handler = stream_handler(broadcasting, user_handler, coder: coder)

          if user_handler
            -> message do
              connection.perform_work handler, :call, message
            end
          else
            handler
          end
        end

        # May be overridden to add instrumentation, logging, specialized error handling,
        # or other forms of handler decoration.
        #
        # TODO: Tests demonstrating this.
        def stream_handler(broadcasting, user_handler, coder: nil)
          if user_handler
            stream_decoder user_handler, coder: coder
          else
            default_stream_handler broadcasting, coder: coder
          end
        end

        # May be overridden to change the default stream handling behavior which decodes
        # JSON and transmits to the client.
        #
        # TODO: Tests demonstrating this.
        #
        # TODO: Room for optimization. Update transmit API to be coder-aware so we can
        # no-op when pubsub and connection are both JSON-encoded. Then we can skip
        # decode+encode if we're just proxying messages.
        def default_stream_handler(broadcasting, coder:)
          coder ||= ActiveSupport::JSON
          stream_transmitter stream_decoder(coder: coder), broadcasting: broadcasting
        end
