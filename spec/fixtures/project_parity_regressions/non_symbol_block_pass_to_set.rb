def compare(values, values_block)
  values.map(&values_block).to_set
end
