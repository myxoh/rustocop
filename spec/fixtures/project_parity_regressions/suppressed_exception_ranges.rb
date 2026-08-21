def remove_images(ids)
  ids.each do |id|
    OptimizedImage.find(id).destroy!
  rescue ActiveRecord::RecordNotFound
  rescue => error
    warn error
  end
end

def ignore_missing_file
  File.unlink(path)
rescue Errno::ENOENT
  # The file is already gone.
end

def patched_files
  Open3.capture2(command)
rescue # rubocop:disable Style/RescueStandardError
end

begin
  perform_work
rescue;
end
