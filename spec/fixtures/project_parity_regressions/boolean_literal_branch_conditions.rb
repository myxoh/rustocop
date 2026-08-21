regex_boolean = if value =~ /enabled/
                  true
                else
                  false
                end

comparison_boolean = if value == "enabled"
                       true
                     else
                       false
                     end

predicate_boolean = ready? ? true : false
