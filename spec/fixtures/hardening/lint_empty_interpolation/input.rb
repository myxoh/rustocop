single_quoted = '#{}'
# "commented #{}"
literal_heredoc = <<~'TEXT'
  #{}
TEXT
character_class = /[\#{}]/
escaped_marker = "literal \#{}"
percent_words = %W[#{''} one]
percent_symbols = %I[#{''} one]
unrelated_percent_words = %W[one two]; same_line = "remove #{nil}"
actual = "remove #{}"
nil_value = "remove #{nil}"
