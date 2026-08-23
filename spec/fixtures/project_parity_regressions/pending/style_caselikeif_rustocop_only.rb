def duration_value(other)
  if Duration === other
    other.value
  elsif Numeric === other
    other
  elsif Scalar === other
    other.value
  end
end
