standalone = URI.regexp
interpolated = /\A#{URI.regexp}\z/
cleaned = text.gsub(URI.regexp(%w[http https]), "")

[standalone, interpolated, cleaned]
