# frozen_string_literal: true

RSpec.describe "RuboCop compatibility" do
  it "matches RuboCop JSON for Layout/TrailingWhitespace file inspection" do
    Dir.mktmpdir do |dir|
      path = File.join(dir, "sample.rb")
      File.write(path, "value = 1  \n")

      rubocop = run_rubocop("--format", "json", "--only", "Layout/TrailingWhitespace", path)
      rustocop = run_rustocop("--format", "json", "--only", "Layout/TrailingWhitespace", path)

      expect(rustocop.stderr).to eq("")
      expect(rustocop.status.exitstatus).to eq(rubocop.status.exitstatus)
      expect(normalize_rubocop_report(parsed_json(rustocop))).to eq(normalize_rubocop_report(parsed_json(rubocop)))
    end
  end

  it "matches RuboCop JSON for clean files" do
    Dir.mktmpdir do |dir|
      path = File.join(dir, "clean.rb")
      File.write(path, "value = 1\n")

      rubocop = run_rubocop("--format", "json", "--only", "Layout/TrailingWhitespace", path)
      rustocop = run_rustocop("--format", "json", "--only", "Layout/TrailingWhitespace", path)

      expect(rustocop.stderr).to eq("")
      expect(rustocop.status.exitstatus).to eq(rubocop.status.exitstatus)
      expect(normalize_rubocop_report(parsed_json(rustocop))).to eq(normalize_rubocop_report(parsed_json(rubocop)))
    end
  end

  it "matches RuboCop JSON for stdin inspection" do
    Dir.mktmpdir do |dir|
      path = File.join(dir, "stdin_sample.rb")
      source = "value = 1  \n"

      rubocop = run_rubocop("--format", "json", "--only", "Layout/TrailingWhitespace", "--stdin", path, stdin: source)
      rustocop = run_rustocop("--format", "json", "--only", "Layout/TrailingWhitespace", "--stdin", path, stdin: source)

      expect(rustocop.stderr).to eq("")
      expect(rustocop.status.exitstatus).to eq(rubocop.status.exitstatus)
      expect(normalize_rubocop_report(parsed_json(rustocop))).to eq(normalize_rubocop_report(parsed_json(rubocop)))
    end
  end

  it "matches RuboCop JSON when autocorrecting Layout/TrailingWhitespace" do
    Dir.mktmpdir do |dir|
      rubocop_dir = File.join(dir, "rubocop")
      rustocop_dir = File.join(dir, "rustocop")
      FileUtils.mkdir_p([rubocop_dir, rustocop_dir])

      rubocop_path = File.join(rubocop_dir, "sample.rb")
      rustocop_path = File.join(rustocop_dir, "sample.rb")
      File.write(rubocop_path, "value = 1  \n")
      File.write(rustocop_path, "value = 1  \n")

      rubocop = run_rubocop("-A", "--format", "json", "--only", "Layout/TrailingWhitespace", rubocop_path)
      rustocop = run_rustocop("-A", "--format", "json", "--only", "Layout/TrailingWhitespace", rustocop_path)

      expect(rustocop.stderr).to eq("")
      expect(rustocop.status.exitstatus).to eq(rubocop.status.exitstatus)
      expect(File.read(rustocop_path)).to eq(File.read(rubocop_path))
      expect(normalize_rubocop_report(parsed_json(rustocop))).to eq(normalize_rubocop_report(parsed_json(rubocop)))
    end
  end

  it "autocorrects a dot directory target like RuboCop" do
    Dir.mktmpdir do |dir|
      path = File.join(dir, "sample.rb")
      File.write(path, "value = 1  \n")

      rustocop = run_rustocop("-A", ".", chdir: dir)

      expect(rustocop.stderr).to eq("")
      expect(rustocop.status.exitstatus).to eq(0)
      expect(File.read(path)).to eq("value = 1\n")
    end
  end
end
