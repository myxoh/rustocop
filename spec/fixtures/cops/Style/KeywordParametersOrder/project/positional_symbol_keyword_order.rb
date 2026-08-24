def initialize(path, fallback = :default, required:, optional: 1)
  [path, fallback, required, optional]
end

def reordered(path, optional: :default, required:)
  [path, optional, required]
end
