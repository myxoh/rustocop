render(<<~TEXT
  top-level heredoc
TEXT
)

def nested
  render(<<~TEXT
    heredoc inside an end-keyword scope
  TEXT
  )
end
