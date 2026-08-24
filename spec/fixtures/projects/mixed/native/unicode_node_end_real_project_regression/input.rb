def test_unicode_column_name
  weird = Weird.create(なまえ: "たこ焼き仮面")
  assert_equal "たこ焼き仮面", weird.なまえ
end

enum :language, [:🇺🇸, :🇪🇸, :🇫🇷]

def format_deletion_date(deletion_date_str)
  Time.zone.parse(deletion_date_str).strftime('%B %d, %Y')
rescue StandardError
  'Unknown'
end
