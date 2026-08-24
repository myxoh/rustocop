result = run_test_command("") .tap(&:to_s)
content = lines.map(&:to_s) .join("\n")
