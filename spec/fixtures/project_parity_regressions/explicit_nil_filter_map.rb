def valid_relationships(relationships)
  relationships.filter_map do |relationship|
    relationship.valid? ? relationship : nil
  end
end
