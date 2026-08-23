values = [
  {
  }, {
  }
]

title&.strip&.split(/\s*[|\-–—·:]+\s*/)&.first

warning = <<~MARKDOWN
  ⚠️ **Data deletion detected**
MARKDOWN

table = "key？"

instance = Object.new
def (instance.foo).kw(a:)
  a
end
