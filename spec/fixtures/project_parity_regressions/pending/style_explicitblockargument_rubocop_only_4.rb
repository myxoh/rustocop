      def with_redis
        Gitlab::Redis::SharedState.with { |redis| yield(redis) }
      end
