        def create_metadatas
          container = self

          RSpec.describe "parent group", :caller => ["/foo_spec.rb:#{__LINE__}"] do; container.parent_group_metadata = metadata
            describe "group", :caller => ["/foo_spec.rb:#{__LINE__}"] do; container.group_metadata = metadata
              container.example_metadata = it("example", :caller => ["/foo_spec.rb:#{__LINE__}"], :if => true).metadata
            end
          end
        end
