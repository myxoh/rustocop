value = options.fetch :name
record = GlobalID::Locator.fetch values[:id]
safe = options&.fetch :name

def forward_lookup(options, &)
  options.fetch(&)
end

options.fetch(:name, nil)
options.fetch(:name) { default }
