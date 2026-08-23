values = [
  {
  }, {
  }
]

title&.strip&.split(/\s*[|\-–—·:]+\s*/)&.first

instance = Object.new
def (instance.foo).kw(a:)
  a
end
