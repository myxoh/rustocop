def prepare(app_path)
  Dir.chdir(app_path) do
    File.open("config/environments/staging.rb", "w")
    File.write("log/staging.log", "staging")
  end
end
