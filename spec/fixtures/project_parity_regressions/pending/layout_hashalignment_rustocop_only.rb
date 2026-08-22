    :parse_gem_dependency

  validates :requirements, length: { maximum: Gemcutter::MAX_FIELD_LENGTH }, gem_requirements: true, presence: true
  validates :unresolved_name, length: { maximum: Gemcutter::MAX_FIELD_LENGTH }, allow_blank: true
  validates :unresolved_name, name_format: true, allow_blank: true, on: :create
  validates :scope, inclusion: { in: %w[development runtime] }

  attr_accessor :gem_dependency

  def self.unresolved(rubygem)
    where(unresolved_name: nil, rubygem_id: rubygem.id)
  end

  def self.mark_unresolved_for(rubygem)
    unresolved(rubygem).update_all(unresolved_name: rubygem.name,
                                   rubygem_id: nil)
  end

  def self.development
    where(scope: "development")
  end

  def self.runtime
    where(scope: "runtime")
  end

  def name
    unresolved_name || rubygem&.name
  end

  def payload
    {
      "name"         => name,
      "requirements" => clean_requirements
    }
  end

  delegate :as_json, :to_yaml, to: :payload

  def to_xml(options = {})
    payload.to_xml(options.merge(root: "dependency"))
  end

  def encode_with(coder)
    coder.tag = nil
    coder.implicit = true
    coder.map = payload
  end

  def to_s
    "#{name} #{clean_requirements}"
  end

  def clean_requirements(reqs = requirements)
    reqs.gsub(/#<YAML::Syck::DefaultKey[^>]*>/, "=")
  end

  def update_resolved(rubygem)
    self.rubygem = rubygem
    self.unresolved_name = nil
    save!
  end

  private
