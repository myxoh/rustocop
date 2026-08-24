      assert_not @cache.fetch(SecureRandom.uuid) { }
