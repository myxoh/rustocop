      private

      # Convert JSON string into Ruby through toggleable adapters.
      #
      # Must rescue adapter-specific errors and return `parser_error`, and
      # must also standardize the options hash to support each adapter as
      # they all take different options.
      #
      # @param string [String] the JSON string to convert to Ruby objects
      # @param opts [Hash] an options hash in the standard JSON gem format
      # @return [Boolean, String, Array, Hash]
      # @raise [JSON::ParserError]
      def adapter_load(string, *_args, **opts)
        opts = standardize_opts(opts)

        Oj.load(string, opts)
      rescue Oj::ParseError, EncodingError, Encoding::UndefinedConversionError, JSON::GeneratorError => e
        raise parser_error, e
      end

      def validate!(string, parse_limits)
        Gitlab::Json::StreamValidator.new(parse_limits).validate!(string)
      rescue Oj::ParseError, EncodingError => e
        raise parser_error, e
      rescue ::Gitlab::Json::StreamValidator::LimitExceededError => e
        log_exceeded_json(e, parse_limits)
        message = ::Gitlab::Json::StreamValidator.user_facing_error_message(e)
        raise parser_error, message
      end

      def log_exceeded_json(exception, parse_limits)
        on_limit_exceeded&.call(exception, parse_limits)
      end
