        @ast = RuboCop::AST::ProcessedSource.new(file_content, RUBY_VERSION.to_f).ast
