        @system_specs_base_folder  = system_specs_base_folder

        # Cannot be extracted to a constant, as it depends on a variable
        @first_js_folder_extract_regexp = %r{
          (?:.*/)?             # Skips the GitLab edition (e.g. ee/, jh/)
          #{@js_base_folder}/  # Most likely app/assets/javascripts/
          (?:pages/)?          # If under a pages folder, we capture the following folder
          ([\w-]*)             # Captures the first folder
        }x
