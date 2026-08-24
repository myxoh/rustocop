LINE_PAIRS_PATTERN = %r{
  # A comment containing an ordinary (parenthesized phrase)
  (?<del_ins>
    -
    \g<del_ins>?
    \+
  )
}x
