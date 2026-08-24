    temp_dir = Rails.root.join('tmp/uploads', "telegram-#{attachment.message_id}")
    FileUtils.mkdir_p(temp_dir)
    temp_file_path = File.join(temp_dir, attachment.file.filename.to_s)

    File.open(temp_file_path, 'wb') do |file|
      attachment.file.blob.open do |blob_file|
        IO.copy_stream(blob_file, file)
      end
    end
