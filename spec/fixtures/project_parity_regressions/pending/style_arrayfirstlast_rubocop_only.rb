          next_traversal_ids_literal = "{#{validated_ids.dup.tap { |ids| ids[-1] += 1 }.join(',')}}"
          location = all_headers[0]&.[]("location")
