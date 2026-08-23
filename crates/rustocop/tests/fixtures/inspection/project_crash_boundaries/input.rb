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

values.each do |value|
  ->(item) { item || value }
end

values.any? { |value| value }
values << +"text"
options = { spread_interval: -10 }

instance = Object.new
def (instance.foo).kw(a:)
  a
end
