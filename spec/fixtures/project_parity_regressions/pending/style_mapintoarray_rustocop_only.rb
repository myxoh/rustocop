      def files
        refs = []

        available_packages.each_batch do |relation|
          batch = relation.preload_files_and_file_metadatum
                          .preload_pypi_metadatum

          batch.each do |package|
            package.installable_package_files.each do |file|
              refs << file_entry(package, file)
            end
          end
        end

        refs
      end
