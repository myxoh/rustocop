      #     <% if logged_in? -%>Welcome, <%= current_user.name %><% end -%>
      #
      # #### Parameters
      # *   `method[, method]` - A name or names of a method on the controller to be
      #     made available on the view.
      def helper_method(*methods)
        methods.flatten!
        self._helper_methods = (_helper_methods + methods).freeze

        location = caller_locations(1, 1).first
        file, line = location.path, location.lineno

        methods.each do |method|
          # def current_user(...)
          #   controller.send(:'current_user', ...)
          # end
          _helpers_for_modification.class_eval <<~ruby_eval.lines.map(&:strip).join(";"), file, line
            def #{method}(...)
              controller.send(:'#{method}', ...)
            end
          ruby_eval
        end
      end

      # Includes the given modules in the template class.
      #
      # Modules can be specified in different ways. All of the following calls include
      # `FooHelper`:
      #
      #     # Module, recommended.
      #     helper FooHelper
      #
      #     # String/symbol without the "helper" suffix, camel or snake case.
