          migration_class = filename.match(ActiveRecord::Migration::MigrationFilenameRegexp)[2].camelize

          File.open(path, "w+") do |file|
            file << <<~MIGRATION
          class #{migration_class} < ActiveRecord::Migration::Current
            def change; end
          end
            MIGRATION
          end
