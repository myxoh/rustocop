class ProjectSeeder
  def initialize(organization:)
    @organization = organization
  end

  def single_line; seed!; end

  private def modifier
    seed!
  end

  def endless = seed!
end
