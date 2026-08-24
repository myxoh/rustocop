      PERCENT_ENCODED = /%[0-9a-fA-F]{2}/.freeze

    module_function

      # Not idempotent, as '%' is escaped to '%25' every time
