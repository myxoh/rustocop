  self.includes = %i[rubygem version]

  self.index_query = lambda {
    query.order(count: :desc)
  }
