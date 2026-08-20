# frozen_string_literal: true

require "digest"
require "fileutils"
require "json"
require "open3"
require "pathname"
require "prism"
require "ripper"
require "thread"
require "tmpdir"
require "yaml"

module Rustocop
  module QualificationBatch
    RUBOCOP_VERSION = "1.87.0"
    RUBOCOP_COMMIT = "e5b788dba181ad94de30cfbad661c5d6aa08a4e5"
    METADATA_CONFIG_KEYS = %w[
      AutoCorrect Description Enabled Reference Safe SafeAutoCorrect StyleGuide
      VersionAdded VersionChanged
    ].freeze
    TOKEN_STOP_WORDS = %w[
      begin class def do else elsif end false if module nil return self then true unless until when while
      add_offense node parent source range expression selector arguments receiver child_nodes each first last
      bar baz c1 c2 foo it item items list object qux record records thread value values _1
    ].freeze

    PROJECTS = [
      {
        "name" => "chatwoot",
        "repository" => "chatwoot/chatwoot",
        "revision" => "8d93d69e8e356216e85c28de7c4240e66b8e83fa"
      },
      {
        "name" => "rubygems.org",
        "repository" => "rubygems/rubygems.org",
        "revision" => "3201f8831866f82eb9acd7f66287a978d0e59079"
      },
      {
        "name" => "gitlab-ce",
        "repository" => "gitlabhq/gitlabhq",
        "revision" => "67a526442c20d20b6e80ebf916bd766b54018c5e"
      }
    ].freeze

    module_function

    def slug(value)
      value.to_s.downcase.gsub(/[^a-z0-9]+/, "_").gsub(/\A_|_\z/, "")[0, 64]
    end

    def source_string(test_case)
      source = test_case.fetch("source")
      return [source.fetch("$hex")].pack("H*") if source.is_a?(Hash) && source.key?("$hex")

      source
    end

    def semantic_config(test_case)
      cop = test_case.fetch("cop")
      cop_config = test_case.fetch("config", {}).fetch(cop, {}).reject do |key, _value|
        METADATA_CONFIG_KEYS.include?(key)
      end
      all_cops = { "TargetRubyVersion" => test_case.fetch("ruby_version", 3.4) }
      config = { "AllCops" => all_cops }
      config[cop] = cop_config unless cop_config.empty?
      config
    end

    class Corpus
      def initialize(path)
        @path = path
      end

      def cases_for(cops)
        wanted = cops.to_h { |cop| [cop, []] }
        File.foreach(@path) do |line|
          test_case = JSON.parse(line)
          wanted[test_case["cop"]]&.push(test_case)
        end
        wanted
      end

      def select_edges(test_cases, limit: 4)
        candidates = test_cases.filter do |test_case|
          source = QualificationBatch.source_string(test_case)
          source.valid_encoding? && !source.include?("\0")
        end
        selected = []
        covered = []
        until selected.length == limit || candidates.empty?
          choice = candidates.max_by do |test_case|
            features = case_features(test_case)
            [(features - covered).length, features.length, -source_size(test_case)]
          end
          selected << choice
          covered |= case_features(choice)
          candidates.delete(choice)
        end
        selected.each_with_index.map { |test_case, index| edge_record(test_case, index) }
      end

      private

      def source_size(test_case)
        QualificationBatch.source_string(test_case).bytesize
      end

      def case_features(test_case)
        source = QualificationBatch.source_string(test_case)
        tokens = Ripper.lex(source).filter_map do |_position, type, token, _state|
          token if %i[on_ident on_const on_op on_kw].include?(type) && token.length > 1
        end
        [
          test_case.fetch("offenses", []).empty? ? "clean" : "offense",
          test_case.key?("correction") ? "correction" : "detection_only",
          source.lines.length > 1 ? "multiline" : "single_line",
          "ruby:#{test_case.fetch("ruby_version", "default")}",
          "path:#{File.extname(test_case.fetch("path", "example.rb"))}",
          "config:#{Digest::SHA256.hexdigest(JSON.generate(QualificationBatch.semantic_config(test_case)))[0, 12]}",
          *tokens.uniq.first(12).map { |token| "token:#{token}" }
        ].uniq
      rescue ArgumentError
        []
      end

      def edge_record(test_case, index)
        example = test_case.fetch("example", {})
        description = example.fetch("description", "Captured upstream example")
        {
          "id" => "upstream_#{index + 1}_#{QualificationBatch.slug(description)}",
          "description" => description,
          "path" => test_case.fetch("path", "example.rb"),
          "config" => QualificationBatch.semantic_config(test_case),
          "source" => QualificationBatch.source_string(test_case)
        }
      end
    end

    class SourceInventory
      attr_reader :root, :rubocop_root

      def initialize(root:, rubocop_root:)
        @root = root
        @rubocop_root = rubocop_root
      end

      def for(cop)
        department, name = cop.split("/", 2)
        ruby_relative = File.join("lib/rubocop/cop", underscore(department), "#{underscore(name)}.rb")
        rust_relative = rust_sources(cop)
        rust_internals = rust_relative.each_with_object(Hash.new { |hash, key| hash[key] = [] }) do |path, merged|
          internals(File.join(root, path), :rust).each do |key, values|
            merged[key] |= values
          end
        end
        {
          "rubocop" => ruby_relative,
          "rustocop" => rust_relative,
          "ruby_internals" => internals(File.join(rubocop_root, ruby_relative), :ruby),
          "rust_internals" => rust_internals
        }
      end

      def markers(cop, test_cases, inventory)
        sources = [File.join(rubocop_root, inventory.fetch("rubocop"))]
        text = sources.filter_map { |path| File.read(path) if File.file?(path) }.join("\n")
        restricted = text.scan(/RESTRICT_ON_SEND\s*=\s*%i\[([^\]]+)\]/m).flat_map do |match|
          match.first.scan(/[a-zA-Z_]\w*[!?=]?/)
        end
        captured = test_cases.select { |item| !item.fetch("offenses", []).empty? }.first(30).flat_map do |item|
          lexical_tokens(QualificationBatch.source_string(item))
        end
        frequencies = captured.tally
        ordered = restricted + frequencies.sort_by { |token, count| [-count, -token.length, token] }.map(&:first)
        ordered.select { |token| useful_marker?(token) }.uniq.first(10)
      end

      private

      def underscore(value)
        value.gsub(/([A-Z]+)([A-Z][a-z])/, '\\1_\\2')
             .gsub(/([a-z\d])([A-Z])/, '\\1_\\2')
             .tr("-", "_").downcase
      end

      def rust_sources(cop)
        root_path = File.join(root, "crates/rustocop/src")
        Dir.glob(File.join(root_path, "**/*.rs")).sort.filter_map do |path|
          relative = Pathname(path).relative_path_from(Pathname(root)).to_s
          relative if File.read(path).include?(%Q("#{cop}"))
        end
      end

      def internals(path, language)
        return {} unless File.file?(path)

        source = File.read(path)
        callbacks = source.scan(language == :ruby ? /def\s+(on_[a-zA-Z0-9_!?]+)/ : /fn\s+(on_[a-zA-Z0-9_]+)/).flatten
        helpers = source.scan(language == :ruby ? /def\s+([a-zA-Z_]\w*[!?]?)/ : /fn\s+([a-zA-Z_]\w*)/).flatten
        config = if language == :ruby
                   source.scan(/cop_config\s*\[\s*["']([^"']+)/).flatten
                 else
                   source.scan(/(?:config_[a-z_]+|enforced_style)\(\s*"([^"]+)/).flatten
                 end
        reports = if language == :ruby
                    source.scan(/\b(add_offense|add_global_offense)\b/).flatten
                  else
                    source.scan(/\b(report_[a-z_]+|add_offense|replace_[a-z_]+|remove_[a-z_]+)\b/).flatten
                  end
        {
          "callbacks" => callbacks.uniq,
          "helpers" => (helpers - callbacks).uniq,
          "configuration" => config.uniq,
          "offense_api" => reports.uniq
        }
      end

      def lexical_tokens(source)
        Ripper.lex(source).filter_map do |_position, type, token, _state|
          token if %i[on_ident on_const on_op].include?(type)
        end
      rescue ArgumentError
        []
      end

      def useful_marker?(token)
        token.length >= 2 && !TOKEN_STOP_WORDS.include?(token) && token.match?(/\A[[:alnum:]_!?=]+\z/)
      end
    end

    class SnippetExtractor
      MAX_LINES = 80

      def extract(source, line, required_markers: [])
        parsed = Prism.parse(source)
        return line_window(source, line) unless parsed.success?

        candidates = statement_nodes(parsed.value, line).select do |node|
          snippet = node.location.slice
          usable?(snippet) && (required_markers.empty? || lexical_marker?(snippet, required_markers))
        end
        candidate = candidates.min_by { |node| node.location.length }
        snippet = candidate&.location&.slice
        return snippet if usable?(snippet)

        line_window(source, line)
      rescue StandardError
        line_window(source, line)
      end

      private

      def statement_nodes(root, line)
        matches = []
        visit = lambda do |node, parent|
          return unless node.respond_to?(:location)

          location = node.location
          return unless location.start_line <= line && location.end_line >= line

          matches << node if parent&.class&.name&.end_with?("StatementsNode")
          node.compact_child_nodes.each { |child| visit.call(child, node) }
        end
        visit.call(root, nil)
        matches
      end

      def usable?(source)
        source && source.lines.length <= MAX_LINES && Prism.parse(source).success?
      end

      def line_window(source, line)
        lines = source.lines
        index = [[line.to_i - 1, 0].max, lines.length - 1].min
        0.upto(8) do |radius|
          first = [index - radius, 0].max
          last = [index + radius, lines.length - 1].min
          candidate = lines[first..last].join
          return candidate if Prism.parse(candidate).success?
        end
        lines[index].to_s
      end

      def lexical_marker?(source, markers)
        Ripper.lex(source).any? do |_position, type, token, _state|
          %i[on_ident on_const on_op on_kw].include?(type) && markers.include?(token)
        end
      rescue ArgumentError
        false
      end
    end

    class ProjectScanner
      EXCLUDED_COMPONENTS = %w[.git coverage ee enterprise log node_modules public tmp].freeze
      PROJECT_CANDIDATE_LIMIT = 6

      def initialize(root:, projects:, rubocop:, config_path:, cache_root: nil)
        @root = root
        @projects = projects
        @rubocop = rubocop
        @config_path = config_path
        @cache_root = cache_root
        @scan_key = Digest::SHA256.hexdigest(
          [RUBOCOP_VERSION, File.read(config_path)].join("\0")
        )[0, 16]
        @extractor = SnippetExtractor.new
      end

      def candidates(cops:, markers:)
        positives = Hash.new { |hash, key| hash[key] = [] }
        negatives = Hash.new { |hash, key| hash[key] = [] }
        @projects.each do |project|
          source_root = project.fetch("source_root")
          report = run_project(project, cops)
          project_positive_counts = Hash.new(0)
          project_positive_paths = Hash.new { |hash, cop| hash[cop] = {} }
          offenses_by_file = Hash.new { |hash, key| hash[key] = Hash.new { |inner, cop| inner[cop] = [] } }
          report.fetch("files", []).each do |file|
            path = File.expand_path(file.fetch("path"), @root)
            file.fetch("offenses", []).each do |offense|
              cop = offense.fetch("cop_name")
              next unless cops.include?(cop)

              line = offense.dig("location", "start_line")
              offenses_by_file[path][cop] << line
              next unless File.file?(path)
              next if project_positive_counts[cop] >= PROJECT_CANDIDATE_LIMIT
              next if project_positive_paths[cop][path]

              source = File.binread(path).encode("UTF-8", invalid: :replace, undef: :replace)
              snippet = @extractor.extract(source, line, required_markers: markers.fetch(cop, []))
              positives[cop] << real_record(project, source_root, path, line, snippet)
              project_positive_counts[cop] += 1
              project_positive_paths[cop][path] = true
            end
          end
          collect_negative_candidates(project, source_root, cops, markers, offenses_by_file, negatives)
        end
        {
          "positives" => positives.transform_values { |items| diverse(items, 8) },
          "negatives" => negatives.transform_values { |items| diverse(items, 16) }
        }
      end

      private

      def run_project(project, cops)
        source_root = project.fetch("source_root")
        cached = cops.to_h { |cop| [cop, read_cached_report(project, cop)] }
        missing = cached.select { |_cop, report| report.nil? }.keys
        unless missing.empty?
          fresh = scan_project(source_root, missing)
          missing.each do |cop|
            report = report_for_cop(fresh, cop)
            cached[cop] = report
            write_cached_report(project, cop, report)
          end
        end
        merge_reports(cached.values)
      end

      def scan_project(source_root, cops)
        command = [*@rubocop, "--cache", "false", "--no-server", "--format", "json",
                   "--only", cops.join(","), "--config", @config_path, source_root]
        stdout, stderr, status = Open3.capture3(*command)
        unless [0, 1].include?(status.exitstatus) && !stdout.empty?
          raise "RuboCop project scan failed (#{status.exitstatus}): #{stderr}"
        end
        JSON.parse(stdout)
      end

      def report_for_cop(report, cop)
        files = report.fetch("files", []).filter_map do |file|
          offenses = file.fetch("offenses", []).select { |offense| offense.fetch("cop_name") == cop }
          { "path" => file.fetch("path"), "offenses" => offenses } unless offenses.empty?
        end
        { "files" => files }
      end

      def merge_reports(reports)
        files = reports.compact.flat_map { |report| report.fetch("files", []) }
                       .group_by { |file| file.fetch("path") }
                       .map do |path, entries|
          { "path" => path, "offenses" => entries.flat_map { |entry| entry.fetch("offenses") } }
        end
        { "files" => files }
      end

      def cache_path(project, cop)
        return unless @cache_root

        directory = File.join(@cache_root, @scan_key, project.fetch("revision"))
        File.join(directory, "#{QualificationBatch.slug(cop)}.json")
      end

      def read_cached_report(project, cop)
        path = cache_path(project, cop)
        JSON.parse(File.read(path)) if path && File.file?(path)
      end

      def write_cached_report(project, cop, report)
        path = cache_path(project, cop)
        return unless path

        FileUtils.mkdir_p(File.dirname(path))
        File.write(path, JSON.generate(report))
      end

      def collect_negative_candidates(project, source_root, cops, markers, offenses, output)
        project_counts = Hash.new(0)
        files = Dir.glob(File.join(source_root, "**/*.rb"), File::FNM_DOTMATCH).sort.reject do |path|
          relative = Pathname(path).relative_path_from(Pathname(source_root)).to_s
          relative.split(File::SEPARATOR).any? do |component|
            EXCLUDED_COMPONENTS.include?(component) || component.start_with?(".")
          end
        end
        files.each do |path|
          break if cops.all? { |cop| project_counts[cop] >= PROJECT_CANDIDATE_LIMIT }

          source = File.binread(path).encode("UTF-8", invalid: :replace, undef: :replace)
          source.each_line.with_index(1) do |line_source, line|
            line_tokens = lexical_tokens(line_source)
            next if line_tokens.empty?

            cops.each do |cop|
              next if project_counts[cop] >= PROJECT_CANDIDATE_LIMIT
              next if offenses.fetch(path, {}).fetch(cop, []).include?(line)
              next if (markers.fetch(cop, []) & line_tokens).empty?

              snippet = @extractor.extract(source, line, required_markers: markers.fetch(cop, []))
              next if snippet.strip.empty?

              output[cop] << real_record(project, source_root, path, line, snippet)
              project_counts[cop] += 1
            end
          end
        end
      end

      def real_record(project, source_root, path, line, source)
        {
          "repository" => project.fetch("repository"),
          "revision" => project.fetch("revision"),
          "path" => Pathname(path).relative_path_from(Pathname(source_root)).to_s,
          "line" => line,
          "source" => source
        }
      end

      def lexical_tokens(line)
        Ripper.lex(line).filter_map do |_position, type, token, _state|
          token if %i[on_ident on_const on_op on_kw].include?(type)
        end.uniq
      rescue ArgumentError
        []
      end

      def diverse(items, limit)
        seen_repositories = Hash.new(0)
        seen_paths = {}
        items.uniq { |item| [item["repository"], item["path"], item["line"], item["source"]] }
             .sort_by { |item| [seen_repositories[item["repository"]], item["source"].bytesize] }
             .each_with_object([]) do |item, selected|
          next if seen_paths[[item["repository"], item["path"]]]

          selected << item
          seen_paths[[item["repository"], item["path"]]] = true
          seen_repositories[item["repository"]] += 1
          break selected if selected.length == limit
        end
      end
    end

    class DifferentialVerifier
      def initialize(rubocop:, rustocop:, cache_root:, jobs: 8)
        @rubocop = rubocop
        @rustocop = rustocop
        @cache_root = cache_root
        @jobs = jobs
        FileUtils.mkdir_p(cache_root)
        @engine_key = Digest::SHA256.hexdigest([
          RUBOCOP_VERSION,
          File.file?(rustocop) ? Digest::SHA256.file(rustocop).hexdigest : "missing"
        ].join(":"))
      end

      def filter(cop, candidates, positive:)
        queue = Queue.new
        candidates.each { |candidate| queue << candidate }
        accepted = []
        errors = []
        mutex = Mutex.new
        workers = Array.new(@jobs) do
          Thread.new do
            loop do
              break if mutex.synchronize { accepted.length >= 2 }

              candidate = queue.pop(true)
              result = verify(cop, candidate)
              expected_polarity = positive ? result.fetch("rubocop_offenses").positive? : result.fetch("rubocop_offenses").zero?
              mutex.synchronize { accepted << candidate if expected_polarity && result.fetch("matches") }
            rescue ThreadError
              break
            rescue StandardError => e
              mutex.synchronize { errors << e }
            end
          end
        end
        workers.each(&:join)
        raise errors.first unless errors.empty?

        accepted.sort_by { |item| [item.fetch("repository"), item.fetch("path"), item.fetch("line")] }.first(2)
      end

      private

      def verify(cop, candidate)
        payload = { "cop" => cop, "candidate" => candidate, "engine" => @engine_key }
        key = Digest::SHA256.hexdigest(JSON.generate(payload))
        cache_path = File.join(@cache_root, "#{key}.json")
        return JSON.parse(File.read(cache_path)) if File.file?(cache_path)

        Dir.mktmpdir("rustocop-qualification-candidate") do |directory|
          relative = candidate.fetch("path").sub(%r{\A/+}, "")
          relative = "example.rb" if relative.empty? || relative.start_with?("../")
          source_path = File.join(directory, relative)
          FileUtils.mkdir_p(File.dirname(source_path))
          config_path = File.join(directory, "rubocop.yml")
          File.write(config_path, YAML.dump(case_config(cop, candidate["config"])))
          source = candidate.fetch("source")
          ruby_diagnostics = diagnostics(@rubocop, cop, config_path, relative, source)
          rust_diagnostics = diagnostics([@rustocop], cop, config_path, relative, source)
          ruby_correction = correction(@rubocop, cop, config_path, source_path, source)
          rust_correction = correction([@rustocop], cop, config_path, source_path, source)
          result = {
            "rubocop_offenses" => ruby_diagnostics.length,
            "matches" => ruby_diagnostics == rust_diagnostics && ruby_correction == rust_correction
          }
          File.write(cache_path, JSON.generate(result))
          result
        end
      end

      def case_config(cop, values)
        values ||= {}
        base = { "AllCops" => { "NewCops" => "disable", "TargetRubyVersion" => 3.4 } }
        if values.keys.any? { |key| key == "AllCops" || key.include?("/") }
          config = base.merge(values)
          config["AllCops"] = base.fetch("AllCops").merge(values.fetch("AllCops", {}))
          config[cop] = { "Enabled" => true }.merge(config.fetch(cop, {}))
          config
        else
          base.merge(cop => { "Enabled" => true }.merge(values))
        end
      end

      def diagnostics(command, cop, config_path, path, source)
        compatibility_options = command.length > 1 ? ["--cache", "false", "--no-server"] : []
        stdout, stderr, status = Open3.capture3(
          *command, *compatibility_options, "--format", "json", "--only", cop,
          "--config", config_path, "--stdin", path, stdin_data: source
        )
        raise "diagnostic command failed: #{stderr}" unless [0, 1].include?(status.exitstatus) && !stdout.empty?

        JSON.parse(stdout).fetch("files").flat_map do |file|
          file.fetch("offenses").map do |offense|
            offense.slice("cop_name", "severity", "message", "correctable", "corrected", "location")
          end
        end.sort_by { |offense| JSON.generate(offense) }
      end

      def correction(command, cop, config_path, source_path, source)
        File.binwrite(source_path, source)
        compatibility_options = command.length > 1 ? ["--cache", "false", "--no-server"] : []
        stdout, stderr, status = Open3.capture3(
          *command, *compatibility_options, "-A", "--format", "json", "--only", cop,
          "--config", config_path, source_path
        )
        raise "correction command failed: #{stderr}" unless [0, 1].include?(status.exitstatus) && !stdout.empty?

        JSON.parse(stdout)
        File.binread(source_path)
      end
    end

    class ReviewPacket
      def render(document)
        rows = document.fetch("cops").map do |cop, record|
          upstream = record.fetch("upstream_tests")
          real = record.fetch("real_world")
          "| `#{cop}` | #{upstream.fetch("passed")}/#{upstream.fetch("total")} | " \
            "#{real.fetch("positives").length}/2 | #{real.fetch("negatives").length}/2 | " \
            "#{record.fetch("preparation").fetch("rust_source_state")} |"
        end
        details = document.fetch("cops").map do |cop, record|
          inventory = record.fetch("preparation").fetch("internals")
          ruby = inventory.fetch("ruby")
          rust = inventory.fetch("rust")
          <<~MARKDOWN
            ## #{cop}

            - RuboCop: `#{record.dig("sources", "rubocop")}`
            - Rustocop: #{Array(record.dig("sources", "rustocop")).map { |path| "`#{path}`" }.join(", ")}
            - Suggested action: **#{record.dig("preparation", "action")}**

            | Internal shape | Ruby | Rust |
            | --- | --- | --- |
            | Callbacks | #{code_list(ruby.fetch("callbacks", []))} | #{code_list(rust.fetch("callbacks", []))} |
            | Helpers | #{code_list(ruby.fetch("helpers", []))} | #{code_list(rust.fetch("helpers", []))} |
            | Configuration | #{code_list(ruby.fetch("configuration", []))} | #{code_list(rust.fetch("configuration", []))} |
            | Offense API | #{code_list(ruby.fetch("offense_api", []))} | #{code_list(rust.fetch("offense_api", []))} |

            Human review remains required. Add two concrete semantic comparison notes and change
            `manual_review.status` only after reviewing the source files above.
          MARKDOWN
        end
        <<~MARKDOWN
          # Qualification batch review

          Prepared against RuboCop #{document.fetch("rubocop_version")} at
          `#{document.fetch("rubocop_commit")}` and Rustocop at
          `#{document.fetch("rustocop_commit")}`.

          | Cop | Upstream | Positives | Negatives | Rust source |
          | --- | ---: | ---: | ---: | --- |
          #{rows.join("\n")}

          Generated candidates have passed differential validation when present, but this packet
          deliberately leaves semantic review pending.

          #{details.join("\n")}
        MARKDOWN
      end

      private

      def code_list(values)
        values.empty? ? "—" : values.map { |value| "`#{value}`" }.join(", ")
      end
    end
  end
end
