class DirectPublic
  class << self
    def public_method
    end
  end
end

class NestedDefinitions
  class << self
    module Patch
      def nested_method
      end
    end

    if enabled?
      def conditional_method
      end
    end
  end
end

class NamedPrivate
  class << self
    def hidden
    end
    private :hidden
  end
end
