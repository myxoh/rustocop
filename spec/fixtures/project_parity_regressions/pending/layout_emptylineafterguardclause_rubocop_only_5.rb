
        ##
        ## ==== <tt>rack.multipart.tempfile_factory</tt>
        ##
        ## An optional object for constructing temporary files for multipart form data. The factory must implement:
        if rack_multipart_tempfile_factory = env[RACK_MULTIPART_TEMPFILE_FACTORY]
          ## * <tt>call(filename, content_type)</tt> to create a temporary file for a multipart form field.
          unless rack_multipart_tempfile_factory.respond_to?(:call)
            raise LintError, "rack.multipart.tempfile_factory must respond to #call"
          end

          ## The factory must return an +IO+-like object that responds to <tt><<</tt> and optionally <tt>rewind</tt>.
          env[RACK_MULTIPART_TEMPFILE_FACTORY] = lambda do |filename, content_type|
            io = rack_multipart_tempfile_factory.call(filename, content_type)
            unless io.respond_to?(:<<)
              raise LintError, "rack.multipart.tempfile_factory return value must respond to #<<"
            end
            io
          end
        end

        ##
        ## ==== <tt>rack.hijack?</tt>
        ##
        ## If present and truthy, indicates that the server supports partial hijacking. See the section below on hijacking for more information.
        #
        # N.B. There is no specific validation here. If the user provides a partial hijack response, we will confirm this value is truthy in `check_hijack_response`.

        ##
        ## ==== <tt>rack.hijack</tt>
        ##
        ## If present, an object responding to +call+ that is used to perform a full hijack. See the section below on hijacking for more information.
        check_hijack(env)
