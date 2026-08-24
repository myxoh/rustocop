# Minimized from gitlabhq/gitlabhq@67a526442c20d20b6e80ebf916bd766b54018c5e
# qa/gems/gitlab-orchestrator/lib/gitlab/orchestrator/lib/instance/configurations/gitlab.rb.

source = <<~RUBY
  def ignored(BAD_PARAMETER)
  end
RUBY

def initialize(
  image:,
  ci:,
  gitlab_domain:
)
  super
end

def each(&bl)
  values.each(&bl)
end
