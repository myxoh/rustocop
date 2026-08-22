        begin
          eval "broke_syntax ="
        rescue Exception
          raise ActionView::Template::Error.new(template)
        end
