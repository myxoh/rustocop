url = nil if url&.empty?
errors.add(:base, message) if @permissions&.empty?
notify if upload&.errors&.empty?

return if object.value&.empty?
