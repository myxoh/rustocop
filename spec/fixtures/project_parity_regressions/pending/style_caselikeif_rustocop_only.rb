    # Returns a copy of the parts hash that defines the duration.
    #
    # ```
    # 5.minutes.parts # => {:minutes=>5}
    # 3.years.parts # => {:years=>3}
    # ```
    def parts
      @parts.dup
    end

    def coerce(other) # :nodoc:
      case other
      when Scalar
        [other, self]
      when Duration
        [Scalar.new(other.value), self]
      else
        [Scalar.new(other), self]
      end
    end

    # Compares one Duration with another or a Numeric to this Duration.
    # Numeric values are treated as seconds.
    def <=>(other)
      if Duration === other
        value <=> other.value
      elsif Numeric === other
        value <=> other
      end
    end

    # Adds another Duration or a Numeric to this Duration. Numeric values
    # are treated as seconds.
    def +(other)
      if Duration === other
        parts = @parts.merge(other._parts) do |_key, value, other_value|
          value + other_value
        end
        Duration.new(value + other.value, parts, @variable || other.variable?)
      else
        seconds = @parts.fetch(:seconds, 0) + other
        Duration.new(value + other, @parts.merge(seconds: seconds), @variable)
      end
    end

    # Subtracts another Duration or a Numeric from this Duration. Numeric
    # values are treated as seconds.
    def -(other)
      self + (-other)
    end

    # Multiplies this Duration by a Numeric and returns a new Duration.
    def *(other)
      if Scalar === other || Duration === other
        Duration.new(value * other.value, @parts.transform_values { |number| number * other.value }, @variable || other.variable?)
      elsif Numeric === other
        Duration.new(value * other, @parts.transform_values { |number| number * other }, @variable)
      else
        raise_type_error(other)
      end
    end

    # Divides this Duration by a Numeric and returns a new Duration.
    def /(other)
      if Scalar === other
        Duration.new(value / other.value, @parts.transform_values { |number| number / other.value }, @variable)
      elsif Duration === other
        value / other.value
      elsif Numeric === other
        Duration.new(value / other, @parts.transform_values { |number| number / other }, @variable)
      else
        raise_type_error(other)
      end
    end

    # Returns the modulo of this Duration by another Duration or Numeric.
    # Numeric values are treated as seconds.
    def %(other)
      if Duration === other || Scalar === other
        Duration.build(value % other.value)
      elsif Numeric === other
        Duration.build(value % other)
      else
        raise_type_error(other)
      end
    end

    def -@ # :nodoc:
      Duration.new(-value, @parts.transform_values(&:-@), @variable)
    end

    def +@ # :nodoc:
      self
    end

    def is_a?(klass) # :nodoc:
      Duration == klass || value.is_a?(klass)
    end
    alias :kind_of? :is_a?

    def instance_of?(klass) # :nodoc:
      Duration == klass || value.instance_of?(klass)
    end

    # Returns `true` if `other` is also a Duration instance with the
    # same `value`, or if `other == value`.
    def ==(other)
      if Duration === other
        other.value == value
      else
        other == value
      end
    end

    # Returns the amount of seconds a duration covers as a string.
    # For more information check to_i method.
    #
    # ```
    # 1.day.to_s # => "86400"
    # ```
    def to_s
      @value.to_s
    end

    # Returns the number of seconds that this Duration represents.
    #
    # ```
    # 1.minute.to_i   # => 60
    # 1.hour.to_i     # => 3600
