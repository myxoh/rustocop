
  def branches_sorted_by(sort_by, pagination_params = nil)
    raw_repository.local_branches(sort_by: sort_by, pagination_params: pagination_params)
  end

  def tags_sorted_by(value, pagination_params = nil)
    raw_repository.tags(sort_by: value, pagination_params: pagination_params)
  end

  # Params:
  #
  # order_by: name|email|commits
  # sort: asc|desc default: 'asc'
  def contributors(ref: nil, order_by: nil, sort: 'asc')
    commits = self.commits(ref, limit: 2000, offset: 0, skip_merges: true)

    commits = commits.group_by(&:author_email).map do |email, commits|
      contributor = Gitlab::Contributor.new
      contributor.email = email

      commits.each do |commit|
        if contributor.name.blank?
          contributor.name = commit.author_name
        end

        contributor.commits += 1
      end

      contributor
    end
    Commit.order_by(collection: commits, order_by: order_by, sort: sort)
  end
