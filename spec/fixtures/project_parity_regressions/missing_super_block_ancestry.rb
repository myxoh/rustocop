module WorkerCallbacks
  class_methods do
    def inherited(subclass)
      subclass.configure
    end
  end
end

RSpec.describe "a nested test class" do
  class Tester < ApplicationController
    def initialize(env = {})
      @request = env
    end
  end
end

Class.new(Parent) do
  def initialize(value)
    @value = value
  end
end
