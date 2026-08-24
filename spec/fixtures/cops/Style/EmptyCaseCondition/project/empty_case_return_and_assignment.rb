def classify(value)
  case
  when value.nil?
    return :missing
  when value.empty?
    :empty
  else
    :present
  end
end

precision = case
            when duration < 1 then 4
            when duration < 120 then 2
            else 0
            end

[classify("value"), precision]
