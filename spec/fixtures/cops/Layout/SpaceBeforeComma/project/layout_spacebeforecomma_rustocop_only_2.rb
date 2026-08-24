        tags_string = tags.map { |k, v| "#{k}=#{v.to_s.gsub(/[ ,=]/) { |char| "\\#{char}" }}" }
