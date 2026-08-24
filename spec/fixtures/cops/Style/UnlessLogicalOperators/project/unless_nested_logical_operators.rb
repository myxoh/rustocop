unless caller_locations.any? { |location| location.path == __FILE__ && (location.lineno == get_line || location.lineno == post_line) }
  raise UnexpectedCall
end

return unless first || second && third
