
    # Marks a TODO YAML file for inspection by renaming the original TODO YAML
    # and appending the suffix +.inspect+ to it.
    #
    # @return [Boolean] +true+ a file was marked for inspection successfully.
    def inspect(cop_name)
      path = path_for(cop_name)

      if File.exist?(path)
        FileUtils.mv(path, "#{path}#{SUFFIX_INSPECT}")
        true
      else
        false
      end
    end

    # Marks all TODO YAML files for inspection.
