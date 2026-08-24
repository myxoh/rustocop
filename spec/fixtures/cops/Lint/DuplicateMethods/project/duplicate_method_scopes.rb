module ErrorTrackingOpenAPI
  class Configuration
    attr_accessor :scheme

    def scheme=(scheme)
      @scheme = scheme
    end

    if RUBY_ENGINE == 'jruby'
      def platform_name
        'jruby'
      end
    else
      def platform_name
        'ruby'
      end
    end
  end

  class OtherConfiguration
    def scheme=(scheme)
      @scheme = scheme
    end
  end
end
