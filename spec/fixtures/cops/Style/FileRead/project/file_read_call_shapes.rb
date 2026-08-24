contents = File.open("document.txt").read
binary = File.open(path, "rb").read
config = File.open(config_file, &:read)
File.open(path, "rt") { |file| file.read }

File.open(path, "a").read
File.open(path) { |file| file.read(1) }
