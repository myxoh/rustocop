      return if query_params.empty?

      query_params.each do |key, _|
        query_params[key] = ["masked_#{key}"] unless MaskHelper::QUERY_PARAMS_TO_NOT_MASK.include?(key)
      end
