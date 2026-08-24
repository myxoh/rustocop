results.any? { |removed| removed.respond_to?(:to_i) && removed.to_i.positive? }
params[:page].respond_to?(:to_i) && params[:page].to_i.between?(1, max_page)
value = value.to_i if value.respond_to?(:to_i)
should_respond_to(:text, &:to_i)
