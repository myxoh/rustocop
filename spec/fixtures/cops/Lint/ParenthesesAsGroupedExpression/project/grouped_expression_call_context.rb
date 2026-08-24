assert_not (value =~ /pattern/)
result = convert (value)

comparison = Time.now > (Time.now + 1)
chained = convert (value).to_s

[result, comparison, chained]
