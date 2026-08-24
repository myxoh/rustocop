report = JSON.dump({ __FILE__ => 123.456 })
invalid = { __FILE__ => "123.456" }.to_s
payload = { __FILE__ => value, "ordinary" => value }
