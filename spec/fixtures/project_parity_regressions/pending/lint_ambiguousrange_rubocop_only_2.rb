  scope :released_within_2hrs, -> { where(released_at: Time.zone.now - 1.hour..Time.zone.now + 1.hour) }
