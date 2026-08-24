object&.params&.uniq { |parameter| parameter.identifier }&.map(&:to_hash)

response.headers[HTTP::Headers::CONTENT_TYPE]&.split(';')&.map(&:strip)&.any? do |value|
  value.start_with?('profile=')
end

object&.user&.guardian&.moderator?(object&.topic&.category)
