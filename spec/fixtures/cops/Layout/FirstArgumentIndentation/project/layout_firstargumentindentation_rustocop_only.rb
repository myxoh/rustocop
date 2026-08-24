      BUILT_IN_SCHEMES[scheme_name.to_sym]&.map { |name, hex| { name: name, hex: hex } } if params[
      :base_scheme_id
    ]
