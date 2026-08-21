Collection.new(select { |variable| !variable.value.nil? })

params.reject! { |key, value| key.to_sym == :order_by && !allowed_values.any?(value) }
