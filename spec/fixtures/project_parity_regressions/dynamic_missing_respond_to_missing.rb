module_with_missing = Module.new do
  def method_missing(name, ...)
    super
  end
end

struct_with_missing = Struct.new(:model) do
  def self.method_missing(method, ...)
    super
  end

  def method_missing(...)
    model.send(...)
  end
end

data_with_missing = Data.define(:value) do
  def method_missing(*)
    nil
  end
end

complete_class = Class.new do
  def method_missing(...)
    super
  end

  def respond_to_missing?(name, include_private = false)
    super
  end
end
