      File.write("lib/version.rb", fake_version_rb("2025.12.0-latest"))
      versions =
        (1..12).each_with_object({}) do |month, hash|
          hash["2025.#{month}"] = { "released" => month <= 6, "esr" => [1, 7].include?(month) }
        end
