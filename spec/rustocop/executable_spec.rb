# frozen_string_literal: true

RSpec.describe "rustocop executable" do
  it "uses the native binary by default" do
    result = run_rustocop("-V")

    expect(result.stderr).to eq("")
    expect(result.stdout).to include("rustocop native")
    expect(result.status.exitstatus).to eq(0)
  end

  it "uses the rustocop name and version" do
    result = run_rustocop("--version")

    expect(result.stderr).to eq("")
    expect(result.stdout).to eq("#{Rustocop::VERSION}\n")
    expect(result.status.exitstatus).to eq(0)
  end

  it "keeps the Ruby fallback explicit for development" do
    result = run_rustocop("-V", env: { "RUSTOCOP_DISABLE_NATIVE" => "1" })

    expect(result.stderr).to eq("")
    expect(result.stdout).to include("rustocop ruby fallback")
    expect(result.status.exitstatus).to eq(0)
  end

  it "advertises every requested non-Singulate cop" do
    expected_cops = %w[
      Bundler/OrderedGems
      Rails/DefaultScope
      Rails/FilePath
      Rails/ApplicationJob
      Rails/ReversibleMigration
      Layout/FirstHashElementIndentation
      Style/HashSyntax
      Style/KeywordParametersOrder
      Style/RedundantBegin
      Style/CaseLikeIf
      Style/ConditionalAssignment
      Style/EmptyCaseCondition
      Style/EmptyElse
      Style/GuardClause
      Style/HashLikeCase
      Style/ClassMethodsDefinitions
      Style/EndlessMethod
      Style/FrozenStringLiteralComment
      Style/TrailingCommaInArrayLiteral
      Style/TrailingCommaInArguments
      Style/TrailingCommaInHashLiteral
      Style/ItAssignment
      Style/NumberedParameters
      Style/StringLiterals
      Naming/PredicatePrefix
      Naming/AccessorMethodName
      Lint/MissingSuper
      Lint/EmptyBlock
      Lint/Debugger
      Security/Eval
      Security/JSONLoad
      Security/MarshalLoad
      Security/Open
      Security/IoMethods
      RSpec/NestedGroups
      RSpec/EmptyExampleGroup
      RSpec/MessageChain
      RSpec/MultipleExpectations
      RSpec/ExampleLength
      RSpec/VariableName
      RSpec/MultipleMemoizedHelpers
      RSpec/Focus
      RSpec/PendingWithoutReason
      RSpec/ScatteredSetup
      RSpec/SpecFilePathSuffix
      RSpec/SpecFilePathFormat
    ]

    result = run_rustocop("--show-cops")
    advertised_cops = result.stdout.lines.map(&:strip)

    expect(result.stderr).to eq("")
    expect(result.status.exitstatus).to eq(0)
    expect(advertised_cops).to include(*expected_cops)
    expect(advertised_cops.grep(%r{\ASingulateCops/})).to be_empty
  end

  it "preserves output order when inspecting files in parallel" do
    Dir.mktmpdir("rustocop-parallel-order-") do |directory|
      files = 10.times.map do |index|
        File.join(directory, format("%02d.rb", index)).tap { |path| File.write(path, "value_#{index} = ()\n") }
      end
      arguments = ["--format", "json", "--only", "Lint/EmptyExpression", *files]

      sequential = run_rustocop(*arguments)
      parallel = run_rustocop("--jobs", "4", *arguments)

      expect(parallel.stderr).to eq("")
      expect(parallel.status.exitstatus).to eq(sequential.status.exitstatus)
      expect(parallel.stdout).to eq(sequential.stdout)
    end
  end

  it "discovers RuboCop configuration and runs the same enabled base cops" do
    Dir.mktmpdir("rustocop-effective-config") do |directory|
      File.write(File.join(directory, ".rubocop.yml"), <<~YAML)
        AllCops:
          DisabledByDefault: true
          NewCops: disable

        Layout/TrailingWhitespace:
          Enabled: true

        Style/StringLiterals:
          Enabled: false
      YAML
      path = File.join(directory, "example.rb")
      File.write(path, "'example'  \n")

      rustocop = run_rustocop("--format", "json", path, chdir: directory)
      fallback = run_rustocop(
        "--format", "json", path, chdir: directory, env: { "RUSTOCOP_DISABLE_NATIVE" => "1" }
      )
      rubocop = run_rubocop("--format", "json", path, chdir: directory)

      expect(rustocop.stderr).to eq("")
      expect(rustocop.status.exitstatus).to eq(rubocop.status.exitstatus)
      expect(normalize_rubocop_report(parsed_json(rustocop))).to eq(normalize_rubocop_report(parsed_json(rubocop)))
      expect(normalize_rubocop_report(parsed_json(fallback))).to eq(normalize_rubocop_report(parsed_json(rubocop)))
    end
  end

  it "uses inherited effective cop settings for explicit native selections" do
    Dir.mktmpdir("rustocop-inherited-config") do |directory|
      File.write(File.join(directory, "base.yml"), <<~YAML)
        AllCops:
          DisabledByDefault: true
          NewCops: disable

        Style/StringLiterals:
          Enabled: true
          EnforcedStyle: double_quotes
      YAML
      File.write(File.join(directory, ".rubocop.yml"), <<~YAML)
        inherit_from: base.yml
      YAML
      path = File.join(directory, "example.rb")
      File.write(path, "'example'\n")

      rustocop = run_rustocop(
        "--format", "json", "--only", "Style/StringLiterals", path, chdir: directory
      )
      rubocop = run_rubocop(
        "--format", "json", "--only", "Style/StringLiterals", path, chdir: directory
      )

      expect(rustocop.stderr).to eq("")
      expect(rustocop.status.exitstatus).to eq(rubocop.status.exitstatus)
      expect(parsed_json(rustocop).dig("summary", "offense_count")).to eq(1)
      expect(parsed_json(rubocop).dig("summary", "offense_count")).to eq(1)
      expect(parsed_json(rustocop).fetch("files").flat_map { |file| file.fetch("offenses") })
        .to contain_exactly(include("cop_name" => "Style/StringLiterals"))
    end
  end

  it "matches DuplicatedGem path, range, and AST hash edge cases" do
    Dir.mktmpdir("rustocop-duplicated-gem-") do |directory|
      File.write(File.join(directory, ".rubocop.yml"), <<~YAML)
        AllCops:
          NewCops: enable
          TargetRubyVersion: 2.7

        Bundler/DuplicatedGem:
          Enabled: true
          Severity: error
          Include:
            - '**/*.{rb,rake}'
      YAML
      paths = {
        "brace.rb" => "gem 'rubocop'\ngem 'rubocop'\n",
        "multiline.rb" => "gem 'rubocop'\n          gem(\n            'rubocop'\n)\n",
        "heredoc.rb" => "gem <<~GEM\n  rubocop\nGEM\ngem \"rubocop\\n\"\n",
        "escaped_heredoc.rb" => "gem <<~FIRST\n  rubo\\x63op\nFIRST\ngem <<~SECOND\n  rubocop\nSECOND\n",
        "source_file.rb" => "gem __FILE__\ngem __FILE__\n",
        "replacement_character.rb" => "gem \"\\u{FFFD}\"\ngem \"�\"\n"
      }.map do |name, source|
        File.join(directory, name).tap { |path| File.write(path, source) }
      end
      binary_path = File.join(directory, "Gémefile.rb")
      File.binwrite(
        binary_path,
        "# encoding: ASCII-8BIT\ngem __FILE__\ngem '#{binary_path}'\n".b
      )
      paths << binary_path
      latin1_distinct_path = File.join(directory, "latin1_distinct.rb")
      File.binwrite(
        latin1_distinct_path,
        "# encoding: ISO-8859-1\ngem 'caf\xE9'\ngem 'caf\xF1'\n".b
      )
      paths << latin1_distinct_path
      latin1_duplicate_path = File.join(directory, "latin1_duplicate.rb")
      File.binwrite(
        latin1_duplicate_path,
        "# encoding: ISO-8859-1\ngem 'caf\xE9'\ngem 'caf\xE9'\n".b
      )
      paths << latin1_duplicate_path
      arguments = ["--format", "json", "--only", "Bundler/DuplicatedGem", *paths]

      rustocop = run_rustocop(
        *arguments,
        chdir: directory,
        env: { "RUSTOCOP_NATIVE_PATH" => File.join(ROOT, "crates/rustocop/target/debug/rustocop") }
      )
      rubocop = run_rubocop(*arguments, chdir: directory)

      expect(rustocop.stderr).to eq("")
      expect(rustocop.status.exitstatus).to eq(rubocop.status.exitstatus)
      expect(normalize_rubocop_report(parsed_json(rustocop)))
        .to eq(normalize_rubocop_report(parsed_json(rubocop)))

      [
        ["--format", "simple"],
        ["--format", "clang"],
        ["-f", "c"],
        ["-f", "j"],
        [],
        ["--format", "simple", "--format", "clang"],
        ["--format", "simple", "--format", "json"],
        ["--format", "json", "--format", "simple"]
      ].each do |formatter_arguments|
        arguments = [*formatter_arguments, "--only", "Bundler/DuplicatedGem", paths.first]
        rustocop = run_rustocop(
          *arguments,
          chdir: directory,
          env: { "RUSTOCOP_NATIVE_PATH" => File.join(ROOT, "crates/rustocop/target/debug/rustocop") }
        )
        rubocop = run_rubocop(*arguments, chdir: directory)

        expect(rustocop.status.exitstatus).to eq(rubocop.status.exitstatus)
        expect(rustocop.stderr).to eq(rubocop.stderr)
        normalize_versions = lambda do |output|
          output.gsub(/"rubocop_version":"[^"]+"/, '"rubocop_version":"normalized"')
        end
        expect(normalize_versions.call(rustocop.stdout)).to eq(normalize_versions.call(rubocop.stdout))
      end

      File.write(File.join(directory, ".rubocop.yml"), <<~YAML)
        AllCops:
          NewCops: enable
          DefaultFormatter: clang
        Bundler/DuplicatedGem:
          Enabled: true
          Include:
            - '**/*.rb'
      YAML
      default_arguments = ["--only", "Bundler/DuplicatedGem", paths.first]
      rustocop = run_rustocop(
        *default_arguments,
        chdir: directory,
        env: { "RUSTOCOP_NATIVE_PATH" => File.join(ROOT, "crates/rustocop/target/debug/rustocop") }
      )
      rubocop = run_rubocop(*default_arguments, chdir: directory)

      expect(rustocop.status.exitstatus).to eq(rubocop.status.exitstatus)
      expect(rustocop.stderr).to eq(rubocop.stderr)
      expect(rustocop.stdout).to eq(rubocop.stdout)

      [
        ["false", []],
        ["false", ["-D"]],
        ["true", ["--no-display-cop-names"]]
      ].each do |configured_display, display_arguments|
        File.write(File.join(directory, ".rubocop.yml"), <<~YAML)
          AllCops:
            NewCops: enable
            DisplayCopNames: #{configured_display}
          Bundler/DuplicatedGem:
            Enabled: true
            Severity: error
            Include:
              - '**/*.rb'
        YAML
        arguments = ["--format", "clang", *display_arguments, "--only", "Bundler/DuplicatedGem", paths.first]
        rustocop = run_rustocop(
          *arguments,
          chdir: directory,
          env: { "RUSTOCOP_NATIVE_PATH" => File.join(ROOT, "crates/rustocop/target/debug/rustocop") }
        )
        rubocop = run_rubocop(*arguments, chdir: directory)

        expect(rustocop.status.exitstatus).to eq(rubocop.status.exitstatus)
        expect(rustocop.stderr).to eq(rubocop.stderr)
        expect(rustocop.stdout).to eq(rubocop.stdout)
      end

      File.write(File.join(directory, ".rubocop.yml"), <<~YAML)
        AllCops:
          NewCops: enable
          StyleGuideBaseURL: https://example.test/style/
        Bundler/DuplicatedGem:
          Enabled: true
          Severity: error
          Details: Read the dependency declaration carefully.
          StyleGuide: duplicated-gem.html
          References:
            - https://example.test/reference
          Include:
            - '**/*.rb'
      YAML
      annotation_arguments = [
        "--format", "clang", "--extra-details", "--display-style-guide",
        "--only", "Bundler/DuplicatedGem", paths.first
      ]
      rustocop = run_rustocop(
        *annotation_arguments,
        chdir: directory,
        env: { "RUSTOCOP_NATIVE_PATH" => File.join(ROOT, "crates/rustocop/target/debug/rustocop") }
      )
      rubocop = run_rubocop(*annotation_arguments, chdir: directory)

      expect(rustocop.status.exitstatus).to eq(rubocop.status.exitstatus)
      expect(rustocop.stdout).to eq(rubocop.stdout)

      File.write(File.join(directory, ".rubocop.yml"), <<~YAML)
        AllCops:
          NewCops: enable
        Bundler/DuplicatedGem:
          Enabled: true
          Include: []
      YAML
      empty_arguments = ["--format", "json", "--only", "Bundler/DuplicatedGem", paths.first]
      rustocop = run_rustocop(
        *empty_arguments,
        chdir: directory,
        env: { "RUSTOCOP_NATIVE_PATH" => File.join(ROOT, "crates/rustocop/target/debug/rustocop") }
      )
      rubocop = run_rubocop(*empty_arguments, chdir: directory)

      expect(normalize_rubocop_report(parsed_json(rustocop)))
        .to eq(normalize_rubocop_report(parsed_json(rubocop)))

      File.write(File.join(directory, ".rubocop.yml"), <<~YAML)
        AllCops:
          NewCops: enable
        Bundler/DuplicatedGem:
          Enabled: true
          Severity: invalid
          Include:
            - '**/*.rb'
      YAML
      invalid_arguments = ["--format", "json", "--only", "Bundler/DuplicatedGem", paths.first]
      rustocop = run_rustocop(
        *invalid_arguments,
        chdir: directory,
        env: { "RUSTOCOP_NATIVE_PATH" => File.join(ROOT, "crates/rustocop/target/debug/rustocop") }
      )
      rubocop = run_rubocop(*invalid_arguments, chdir: directory)

      expect(normalize_rubocop_report(parsed_json(rustocop)))
        .to eq(normalize_rubocop_report(parsed_json(rubocop)))
      expect(rustocop.stderr).to include("Invalid severity 'invalid'")
      expect(rubocop.stderr).to include("Invalid severity 'invalid'")

      File.write(File.join(directory, ".rubocop.yml"), <<~YAML)
        AllCops:
          NewCops: enable
        Bundler/DuplicatedGem:
          Enabled: true
          Include:
            - '**/[Gg]emfile'
            - '**/[!a]emfile'
      YAML
      bracket_path = File.join(directory, "Gemfile")
      File.write(bracket_path, "gem 'rubocop'\ngem 'rubocop'\n")
      bracket_arguments = ["--format", "json", "--only", "Bundler/DuplicatedGem", bracket_path]
      rustocop = run_rustocop(
        *bracket_arguments,
        chdir: directory,
        env: { "RUSTOCOP_NATIVE_PATH" => File.join(ROOT, "crates/rustocop/target/debug/rustocop") }
      )
      rubocop = run_rubocop(*bracket_arguments, chdir: directory)

      expect(normalize_rubocop_report(parsed_json(rustocop)))
        .to eq(normalize_rubocop_report(parsed_json(rubocop)))

      File.write(File.join(directory, ".rubocop.yml"), <<~'YAML')
        AllCops:
          NewCops: enable
        Bundler/DuplicatedGem:
          Enabled: true
          Include:
            - '**/Gem\file'
      YAML
      escape_arguments = ["--format", "json", "--only", "Bundler/DuplicatedGem", bracket_path]
      rustocop = run_rustocop(
        *escape_arguments,
        chdir: directory,
        env: { "RUSTOCOP_NATIVE_PATH" => File.join(ROOT, "crates/rustocop/target/debug/rustocop") }
      )
      rubocop = run_rubocop(*escape_arguments, chdir: directory)

      expect(normalize_rubocop_report(parsed_json(rustocop)))
        .to eq(normalize_rubocop_report(parsed_json(rubocop)))

    end
  end

  it "preserves DuplicatedGem invalid message bytes through formatter failure" do
    Dir.mktmpdir("rustocop-duplicated-gem-invalid-byte-") do |directory|
      File.write(File.join(directory, ".rubocop.yml"), <<~YAML)
        AllCops:
          NewCops: enable
        Bundler/DuplicatedGem:
          Enabled: true
      YAML
      path = File.join(directory, "Gemfile")
      File.write(path, "gem \"\\xFF\"\ngem \"\\xFF\"\n")
      arguments = ["--format", "json", "--only", "Bundler/DuplicatedGem", path]

      rustocop = run_rustocop(
        *arguments,
        chdir: directory,
        env: { "RUSTOCOP_NATIVE_PATH" => File.join(ROOT, "crates/rustocop/target/debug/rustocop") }
      )
      rubocop = run_rubocop(*arguments, chdir: directory)

      expect(rustocop.status.exitstatus).to eq(rubocop.status.exitstatus)
      expect(rustocop.status).not_to be_success
      expect(rustocop.stderr).to include("source sequence is illegal/malformed utf-8")
      expect(rubocop.stderr).to include("source sequence is illegal/malformed utf-8")

      progress_arguments = ["--only", "Bundler/DuplicatedGem", path]
      rustocop = run_rustocop(
        *progress_arguments,
        chdir: directory,
        env: { "RUSTOCOP_NATIVE_PATH" => File.join(ROOT, "crates/rustocop/target/debug/rustocop") }
      )
      rubocop = run_rubocop(*progress_arguments, chdir: directory)

      expect(rustocop.status.exitstatus).to eq(rubocop.status.exitstatus)
      expect(rustocop.stdout).to eq(rubocop.stdout)
      expect(rustocop.stderr).to include("invalid byte sequence in UTF-8")
      expect(rubocop.stderr).to include("invalid byte sequence in UTF-8")

      [
        [["--format", "simple", "--format", "json"], "invalid byte sequence in UTF-8"],
        [["--format", "json", "--format", "progress"], "source sequence is illegal/malformed utf-8"]
      ].each do |formatter_arguments, expected_error|
        arguments = [*formatter_arguments, "--only", "Bundler/DuplicatedGem", path]
        rustocop = run_rustocop(
          *arguments,
          chdir: directory,
          env: {
            "RUSTOCOP_NATIVE_PATH" => File.join(ROOT, "crates/rustocop/target/debug/rustocop"),
            "RUSTOCOP_VERSION" => Gem.loaded_specs.fetch("rubocop").version.to_s
          }
        )
        rubocop = run_rubocop(*arguments, chdir: directory)

        expect(rustocop.status.exitstatus).to eq(rubocop.status.exitstatus)
        normalize_versions = lambda do |output|
          output.gsub(/"rubocop_version":"[^"]+"/, '"rubocop_version":"normalized"')
        end
        expect(normalize_versions.call(rustocop.stdout)).to eq(normalize_versions.call(rubocop.stdout))
        expect(rustocop.stderr).to include(expected_error)
        expect(rubocop.stderr).to include(expected_error)
      end
    end
  end

  it "matches DuplicatedGroup AST traversal and grouping edge cases" do
    Dir.mktmpdir("rustocop-duplicated-group-") do |directory|
      File.write(File.join(directory, ".rubocop.yml"), <<~YAML)
        AllCops:
          NewCops: enable
          TargetRubyVersion: 2.7

        Bundler/DuplicatedGroup:
          Enabled: true
          Include:
            - '**/*.rb'
      YAML
      paths = {
        "multiline.rb" => <<~RUBY,
          group(
            :development
          ) do
            gem 'one'
          end
          group(
            :development
          ) do
            gem 'two'
          end
        RUBY
        "explicit_receiver.rb" => <<~RUBY,
          self.group :development do
            gem 'one'
          end
          self.group :development do
            gem 'two'
          end
        RUBY
        "ordinary_block.rb" => <<~RUBY,
          2.times do
            group :development do
              gem 'one'
            end
          end
          group :development do
            gem 'two'
          end
        RUBY
        "nested_groups.rb" => <<~RUBY,
          group :first do
            group :shared do
              gem 'one'
            end
          end
          group :second do
            group :shared do
              gem 'two'
            end
          end
        RUBY
        "nearest_source.rb" => <<~RUBY,
          source 'one' do
            git 'same' do
              group :development do
                gem 'one'
              end
            end
          end
          source 'two' do
            git 'same' do
              group :development do
                gem 'two'
              end
            end
          end
        RUBY
        "first_source_argument.rb" => <<~RUBY,
          source 'same', foo: 1 do
            group :development do
              gem 'one'
            end
          end
          source 'same', foo: 2 do
            group :development do
              gem 'two'
            end
          end
        RUBY
        "composite_arguments.rb" => <<~RUBY,
          group [:development, :test], foo: [1, 2] do
            gem 'one'
          end
          group [:development, :test], foo: [1, 2] do
            gem 'two'
          end
        RUBY
        "numeric_values.rb" => <<~RUBY,
          group 1.50 do
            gem 'one'
          end
          group 1.5 do
            gem 'two'
          end
          group 1.50r do
            gem 'three'
          end
          group 1.5r do
            gem 'four'
          end
          group 2.00i do
            gem 'five'
          end
          group 2.0i do
            gem 'six'
          end
        RUBY
        "big_integer_values.rb" => <<~RUBY,
          group 0x100000000000000000000000000000000 do
            gem 'one'
          end
          group 340282366920938463463374607431768211456 do
            gem 'two'
          end
          group 340282366920938463463374607431768211457 do
            gem 'three'
          end
        RUBY
        "huge_rational_values.rb" => <<~RUBY,
          group 99999999999999999999999999999999999999.0r do
            gem 'one'
          end
          group 99999999999999999999999999999999999999r do
            gem 'two'
          end
        RUBY
        "huge_complex_rational_values.rb" => <<~RUBY,
          group 99999999999999999999999999999999999999.0ri do
            gem 'one'
          end
          group 99999999999999999999999999999999999999ri do
            gem 'two'
          end
        RUBY
        "descending_multiline_range.rb" => <<~RUBY,
              group(
                :development
          ) do
            gem 'one'
          end
              group(
                :development
          ) do
            gem 'two'
          end
        RUBY
        "dynamic_string_values.rb" => <<~'RUBY',
          group "foo#{bar}" do
            gem 'one'
          end
          group %Q(foo#{bar}) do
            gem 'two'
          end
        RUBY
        "non_finite_float_values.rb" => <<~RUBY,
          group 1e400 do
            gem 'one'
          end
          group 1e401 do
            gem 'two'
          end
        RUBY
        "complex_infinity_values.rb" => <<~RUBY,
          group "0+Infinity*i" do
            gem 'one'
          end
          group 1e400i do
            gem 'two'
          end
        RUBY
        "xstr_first_child_values.rb" => <<~'RUBY'
          group `foo#{bar}` do
            gem 'one'
          end
          group %x(foo#{baz}) do
            gem 'two'
          end
          group "" do
            gem 'three'
          end
          group `` do
            gem 'four'
          end
          group '(str "foo")' do
            gem 'five'
          end
          group `foo` do
            gem 'six'
          end
        RUBY
      }.map do |name, source|
        File.join(directory, name).tap { |path| File.write(path, source) }
      end
      arguments = ["--format", "json", "--only", "Bundler/DuplicatedGroup", *paths]

      rustocop = run_rustocop(
        *arguments,
        chdir: directory,
        env: { "RUSTOCOP_NATIVE_PATH" => File.join(ROOT, "crates/rustocop/target/debug/rustocop") }
      )
      rubocop = run_rubocop(*arguments, chdir: directory)

      expect(rustocop.stderr).to eq("")
      expect(rustocop.status.exitstatus).to eq(rubocop.status.exitstatus)
      expect(normalize_rubocop_report(parsed_json(rustocop)))
        .to eq(normalize_rubocop_report(parsed_json(rubocop)))

      binary_path = File.join(directory, "binary.rb")
      binary_source = "# encoding: ASCII-8BIT\n\ngroup \"".b + "\xFF".b + "\" do\n  gem 'one'\nend\n".b +
        "group \"".b + "\xFF".b + "\" do\n  gem 'two'\nend\n".b
      File.binwrite(binary_path, binary_source)
      binary_arguments = ["--format", "progress", "--only", "Bundler/DuplicatedGroup", binary_path]
      binary_rustocop = run_rustocop(
        *binary_arguments,
        chdir: directory,
        env: { "RUSTOCOP_NATIVE_PATH" => File.join(ROOT, "crates/rustocop/target/debug/rustocop") }
      )
      binary_rubocop = run_rubocop(*binary_arguments, chdir: directory)

      expect(binary_rustocop.status.exitstatus).to eq(binary_rubocop.status.exitstatus)
      expect(binary_rustocop.stdout.b).to eq(binary_rubocop.stdout.b)
      expect(binary_rustocop.stderr.b).to eq(binary_rubocop.stderr.b)
    end
  end

  it "prefers a compiled Rustocop configuration" do
    Dir.mktmpdir("rustocop-compiled-config") do |directory|
      File.write(File.join(directory, ".rubocop.yml"), <<~YAML)
        AllCops:
          DisabledByDefault: true

        Style/StringLiterals:
          Enabled: false
      YAML
      File.write(File.join(directory, ".rustocop.yml"), <<~YAML)
        Rustocop:
          SchemaVersion: 1
          BuiltInCops:
            - Style/StringLiterals
          NonNativeCops: []
        AllCops:
          DisabledByDefault: true
        Style/StringLiterals:
          Enabled: true
          EnforcedStyle: double_quotes
      YAML
      path = File.join(directory, "example.rb")
      File.write(path, "'example'\n")

      result = run_rustocop("--format", "json", path, chdir: directory)

      expect(result.stderr).to eq("")
      expect(result.status.exitstatus).to eq(1)
      expect(parsed_json(result).fetch("files").flat_map { |file| file.fetch("offenses") })
        .to contain_exactly(include("cop_name" => "Style/StringLiterals"))
    end
  end

  it "keeps resolved path exclusions relative to the discovered config" do
    Dir.mktmpdir("rustocop-resolved-paths") do |directory|
      File.write(File.join(directory, ".rubocop.yml"), <<~YAML)
        AllCops:
          DisabledByDefault: true
          NewCops: disable
          Exclude:
            - excluded.rb

        Layout/TrailingWhitespace:
          Enabled: true
      YAML
      File.write(File.join(directory, "included.rb"), "puts :included  \n")
      File.write(File.join(directory, "excluded.rb"), "puts :excluded  \n")

      rustocop = run_rustocop("--format", "json", directory, chdir: directory)
      rubocop = run_rubocop("--format", "json", directory, chdir: directory)

      expect(rustocop.stderr).to eq("")
      expect(rustocop.status.exitstatus).to eq(rubocop.status.exitstatus)
      expect(normalize_rubocop_report(parsed_json(rustocop)))
        .to eq(normalize_rubocop_report(parsed_json(rubocop)))
      expect(parsed_json(rustocop).fetch("files").map { |file| File.basename(file.fetch("path")) })
        .to eq(["included.rb"])
    end
  end

  it "warns about configured non-native cops and delegates them only when requested" do
    Dir.mktmpdir("rustocop-non-native-config") do |directory|
      custom_cop = File.join(ROOT, "benchmark/custom_cops/synthetic_file_header.rb")
      File.write(File.join(directory, ".rubocop.yml"), <<~YAML)
        require:
          - #{custom_cop.inspect}

        AllCops:
          DisabledByDefault: true
          NewCops: disable

        Custom/SyntheticFileHeader:
          Enabled: true
      YAML
      path = File.join(directory, "example.rb")
      File.write(path, "puts :ok\n")

      ignored = run_rustocop("--format", "json", path, chdir: directory)
      included = run_rustocop(
        "--included-non-native-cops", "--format", "json", path, chdir: directory
      )

      expect(ignored.stderr).to eq("#{Rustocop::RubocopConfiguration::WARNING}\n")
      expect(parsed_json(ignored).dig("summary", "offense_count")).to eq(0)
      expect(included.stderr).to eq("")
      expect(parsed_json(included).fetch("files").flat_map { |file| file.fetch("offenses") })
        .to contain_exactly(include("cop_name" => "Custom/SyntheticFileHeader"))
    end
  end

  it "rejects an invalid parallel worker count" do
    result = run_rustocop("--jobs", "0", "--stdin", "example.rb", stdin: "puts :ok\n")

    expect(result.stderr).to eq("rustocop: invalid worker count 0\n")
    expect(result.status.exitstatus).to eq(2)
  end

  it "inspects endless methods without panicking in the legacy lint pass" do
    result = run_rustocop(
      "--format", "json", "--only", "Lint/UnusedMethodArgument", "--stdin", "example.rb",
      stdin: "def example(unused) = :ok\n"
    )

    expect(result.status.exitstatus).to eq(1)
    expect(parsed_json(result).fetch("files").flat_map { |file| file.fetch("offenses") })
      .to contain_exactly(include("cop_name" => "Lint/UnusedMethodArgument"))
  end

  it "delegates required custom cops while keeping built-in cops native" do
    Dir.mktmpdir("rustocop-mixed-cops-") do |directory|
      files = 3.times.map do |index|
        File.join(directory, format("%02d.rb", index)).tap { |path| File.write(path, "value_#{index} = ()\n") }
      end
      custom_cop = File.join(ROOT, "benchmark/custom_cops/synthetic_file_header.rb")
      config = File.join(ROOT, "benchmark/custom-cop-rubocop.yml")
      arguments = [
        "--no-parallel", "--format", "json", "--config", config,
        "--require", custom_cop, "--only", "Lint/EmptyExpression,Custom/SyntheticFileHeader", *files
      ]

      mixed = run_rustocop(*arguments)
      rubocop = run_rubocop(*arguments.reject { |argument| argument == "--no-parallel" }, "--no-server")

      expect(mixed.stderr).to eq("")
      expect(mixed.status.exitstatus).to eq(1)
      expect(normalize_rubocop_report(parsed_json(mixed))).to eq(normalize_rubocop_report(parsed_json(rubocop)))
    end
  end

  it "rejects autocorrection when native and custom cops are mixed" do
    result = run_rustocop(
      "-A", "--require", File.join(ROOT, "benchmark/custom_cops/synthetic_file_header.rb"),
      "--only", "Lint/EmptyExpression,Custom/SyntheticFileHeader", __FILE__
    )

    expect(result.stderr).to eq("rustocop: mixed custom-cop runs do not yet support autocorrection\n")
    expect(result.status.exitstatus).to eq(2)
  end

  it "autocorrects distinct files safely in parallel" do
    Dir.mktmpdir("rustocop-parallel-correction") do |directory|
      files = 6.times.map do |index|
        File.join(directory, "example_#{index}.rb").tap { |path| File.write(path, "{key:1}\n") }
      end

      result = run_rustocop("-A", "--jobs", "4", "--only", "Layout/SpaceAfterColon", *files)

      expect(result.stderr).to eq("")
      expect(result.status.exitstatus).to eq(0)
      expect(files.map { |path| File.read(path) }).to all(eq("{key: 1}\n"))
    end
  end
end
