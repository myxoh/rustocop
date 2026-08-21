private def hidden
end

module_function def helper
end

public false
private :hidden
private attr_reader :value
private(*delegate(:street, to: :place))
private :"before_save_#{name}"

def declare(name)
  private :"before_save_#{name}"
end

proc { private method_name }

source = <<~RUBY
  private def generated
  end
RUBY
