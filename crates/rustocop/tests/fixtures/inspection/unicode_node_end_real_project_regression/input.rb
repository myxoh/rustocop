def test_unicode_column_name
  weird = Weird.create(なまえ: "たこ焼き仮面")
  assert_equal "たこ焼き仮面", weird.なまえ
end

enum :language, [:🇺🇸, :🇪🇸, :🇫🇷]
