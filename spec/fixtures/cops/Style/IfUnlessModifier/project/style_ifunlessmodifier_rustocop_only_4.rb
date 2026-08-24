            prohibited = prohibited_key?(key)

            unless prohibited
              result[key] = value.is_a?(Hash) ? transform(context, value) : value
            end
