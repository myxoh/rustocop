class DrilldownBuilder
  def self.supported_dimension_type?(type) = type.in?(%w[agent label])
end
