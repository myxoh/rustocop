case first
in Pathname
  first.to_s
end

case second
in [Pathname, String, Integer] | [Pathname, Integer, Integer]
  second
in Pathname
  second.to_s
end

message = <<~MESSAGE
  in #222
  in #222
MESSAGE
