test "nested class" do
  class NestedCommand
    alias :foo :perform
    alias :bar :perform
  end
end

class Container
  module Behavior
    alias_method :new_name, :old_name
  end
end
