def sanitize
  @query = @query.delete("\u0000")
end
