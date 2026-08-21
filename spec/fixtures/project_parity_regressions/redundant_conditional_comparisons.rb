regex_value = if input =~ /enabled/
                true
              else
                false
              end

ordering = input <=> expected ? true : false
comparison = input == expected ? true : false
