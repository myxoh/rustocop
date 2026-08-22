template = <<~RUBY
  class Phantom
  end
RUBY

class First
end

class Second < First
end

if condition?
  class ConditionalDefinition
  end
end
