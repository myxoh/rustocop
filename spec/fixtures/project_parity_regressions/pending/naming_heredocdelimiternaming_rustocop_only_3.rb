
    def write_gem_lines(io, gems)
      gems.each do |gem|
        version_numbers = gem.versions.map(&:number_and_platform).join(",")
        io << gem.name <<
          " " << version_numbers <<
          " #{gem.versions.last.info_checksum}\n"
      end
    end
