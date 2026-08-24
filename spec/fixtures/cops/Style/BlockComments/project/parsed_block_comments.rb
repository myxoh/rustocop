template = <<~RUBY
  =begin
  This is heredoc content, not a Ruby document comment.
  =end
RUBY

# The following tokens are part of an ordinary line comment: =begin and =end.

=begin
This is a real Ruby document comment.

It should be converted to line comments.
=end

puts template
