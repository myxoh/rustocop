  module_eval %w(raises_before raises_after raises_both no_raise no_action).map { |action| "def #{action}; default_action end" }.join("\n")
