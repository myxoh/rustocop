scope = ->(file_types) do
  self.file_types.select { |file_type| file_types.include?(file_type) }
end
