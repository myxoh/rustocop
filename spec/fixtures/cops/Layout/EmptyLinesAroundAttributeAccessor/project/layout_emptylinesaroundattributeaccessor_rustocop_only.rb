
      def initialize(attr:, user:, token:)
        super(attr: attr, user: user)

        @token = token
      end

      def execute
        return failure(:rate_limited) if verification_rate_limited?
        return failure(:invalid) unless valid?
        return failure(:expired) if expired_token?

        success
      end

      def expired_token?
        generated_at = case attr
                       when :unlock_token then user.locked_at
                       when :confirmation_token then user.confirmation_sent_at
                       when :email_otp then user.email_otp_last_sent_at
                       end

        generated_at.nil? ||
          generated_at < TOKEN_VALID_FOR_MINUTES.minutes.ago
      end

      private

      attr_reader :user

      def verification_rate_limited?
        Gitlab::ApplicationRateLimiter.throttled?(:email_verification, scope: attr_value || :global)
      end
