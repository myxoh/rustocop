result = +""

result << if enabled? && visible?
  "visible"
else
  "hidden"
end

result << (included? "value" && fallback)

puts result
