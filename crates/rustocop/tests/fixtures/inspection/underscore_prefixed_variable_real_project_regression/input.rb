def convert(values)
  values.map { |_| transform(_) }
end

def define_attribute(attr_name, _owner: generated_methods, as: attr_name)
  batch(_owner)
end

def begin_transaction(_lazy: true)
  materialize if _lazy
end

def intentionally_unused(_value)
end

def same_name_but_used_elsewhere(_value)
  consume(_value)
end
