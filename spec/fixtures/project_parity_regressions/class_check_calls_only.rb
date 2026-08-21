alias_method :kind_of?, :is_a?
alias :kind_of? :is_a?

# Documentation may mention value.kind_of?(Numeric).

ordinary = value.kind_of?(Numeric)
safe = value&.kind_of?(Numeric)

[ordinary, safe]
