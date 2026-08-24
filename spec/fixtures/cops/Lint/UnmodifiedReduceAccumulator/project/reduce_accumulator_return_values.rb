50.times.inject(1) { |value, _| { "a" => value } }

key.split(".").reduce(DEFAULTS) { |defaults, part| defaults[part.to_sym] }

(1...100).inject({}) { |hash, key| hash["key_#{key}"] = true; hash }

ids.inject(Hash.new { |hash, key| hash[key] = [] }) do |hash, id|
  file, id = parse_id(id)
  hash[file] << id
  hash
end

imports.map { |part| fragment(part).errors }.reduce(errors.to_set) { |left, right| left | right }

fbffs.flatten.inject { |left, right| left.merge(right) { |_, first, second| first + second } }
