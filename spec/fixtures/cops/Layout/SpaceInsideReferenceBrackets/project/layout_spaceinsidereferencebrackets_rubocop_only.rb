          Hash[params.map { |k, v| [k.to_s.tr("-", "_"), normalize_keys(v)] } ]
