def index_with(default = (no_default = true))
  to_enum(:index_with) { size }
end

def transform_keys!(hash = NOT_GIVEN, &block)
  to_enum(:transform_keys!)
end
