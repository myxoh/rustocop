        missing_objects = (expected.keys - actual.keys).map { |it| expected[it].slice(:title, :url) }
