    def placeholder(*args)
      if args.present?
        @placeholders << args[0]
      elsif block_given?
        @placeholders =
          @placeholders.concat(Array(yield(@automation.serialized_fields, @automation)))
      end
    end
