# frozen_string_literal: true

RSpec.describe "Prism cop compatibility" do
  cops = {
    "security_eval" => "Security/Eval",
    "security_json_load" => "Security/JSONLoad",
    "security_marshal_load" => "Security/MarshalLoad",
    "security_open" => "Security/Open",
    "security_io_methods" => "Security/IoMethods"
  }

  examples = cops.flat_map do |directory, cop|
    Dir[File.join(ROOT, "spec", "fixtures", "prism_examples", directory, "*.rb")].sort.map do |path|
      [cop, path]
    end
  end

  raise "expected exactly 25 Prism examples, got #{examples.length}" unless examples.length == 25

  examples.each do |cop, path|
    relative_path = path.delete_prefix("#{ROOT}/")

    it "matches RuboCop for #{cop} on #{relative_path}" do
      rubocop = run_rubocop("--cache", "false", "--format", "json", "--only", cop, path)
      rustocop = run_rustocop("--cache", "false", "--format", "json", "--only", cop, path)

      expect(rustocop.stderr).to eq("")
      expect(rustocop.status.exitstatus).to eq(rubocop.status.exitstatus)
      expect(normalize_rubocop_report(parsed_json(rustocop))).to eq(
        normalize_rubocop_report(parsed_json(rubocop))
      )
    end
  end

  it "matches RuboCop across all 25 files as one Security department run" do
    paths = examples.map(&:last).sort
    rubocop = run_rubocop("--cache", "false", "--format", "json", "--only", "Security", *paths)
    rustocop = run_rustocop("--cache", "false", "--format", "json", "--only", "Security", *paths)

    expect(rustocop.stderr).to eq("")
    expect(rustocop.status.exitstatus).to eq(rubocop.status.exitstatus)
    expect(normalize_rubocop_report(parsed_json(rustocop))).to eq(
      normalize_rubocop_report(parsed_json(rubocop))
    )
  end

  {
    "Security/JSONLoad" => ["JSON.load(payload)\n", "JSON.parse(payload)\n"],
    "Security/IoMethods" => ["IO.read(path)\n", "File.read(path)\n"]
  }.each do |cop, (source, corrected_source)|
    it "matches RuboCop autocorrection for #{cop}" do
      Dir.mktmpdir do |dir|
        rubocop_dir = File.join(dir, "rubocop")
        rustocop_dir = File.join(dir, "rustocop")
        FileUtils.mkdir_p([rubocop_dir, rustocop_dir])
        rubocop_path = File.join(rubocop_dir, "sample.rb")
        rustocop_path = File.join(rustocop_dir, "sample.rb")
        File.write(rubocop_path, source)
        File.write(rustocop_path, source)

        rubocop = run_rubocop("--cache", "false", "-A", "--format", "json", "--only", cop, rubocop_path)
        rustocop = run_rustocop("--cache", "false", "-A", "--format", "json", "--only", cop, rustocop_path)

        expect(rustocop.stderr).to eq("")
        expect(rustocop.status.exitstatus).to eq(rubocop.status.exitstatus)
        expect(File.read(rustocop_path)).to eq(corrected_source)
        expect(File.read(rustocop_path)).to eq(File.read(rubocop_path))
        expect(normalize_rubocop_report(parsed_json(rustocop))).to eq(
          normalize_rubocop_report(parsed_json(rubocop))
        )
      end
    end
  end
end
