# frozen_string_literal: true

require_relative "generated_section"

module Rustocop
  module BenchmarkDocumentation
    module_function

    def update_rubocop_prism(root, report)
      results = report.fetch("results")
      final = results.last
      speedup = final.fetch("speedup_vs_rubocop")
      rows = results.map do |result|
        "| #{result.fetch("files")} | #{milliseconds(result.dig("rustocop", "median_seconds"), 2)} ms | " \
          "#{milliseconds(result.dig("rubocop", "median_seconds"), 2)} ms | " \
          "#{format("%.2f", result.fetch("speedup_vs_rubocop"))}× |"
      end.join("\n")
      detailed_rows = results.map do |result|
        "| #{result.fetch("files")} | #{result.fetch("runs")} | " \
          "#{measurement(result.fetch("rustocop"))} | #{measurement(result.fetch("rubocop"))} | " \
          "#{format("%.2f", result.fetch("speedup_vs_rubocop"))}× |"
      end.join("\n")

      GeneratedSection.replace(File.join(root, "README.md"), "rubocop-prism", <<~MARKDOWN)
        On the pinned 500-file, 20-cop benchmark corpus, rustocop is currently
        about #{speedup.round} times faster than RuboCop with Prism. Both tools produced identical
        normalized JSON before measurement.

        | Files | rustocop | RuboCop (Prism) | Speedup |
        | ---: | ---: | ---: | ---: |
        #{rows}
      MARKDOWN
      GeneratedSection.replace(File.join(root, "docs/performance.md"), "rubocop-prism-results", <<~MARKDOWN)
        ## Results

        | Files | Runs | Rustocop median / p95 | RuboCop Prism median / p95 | Speedup |
        | ---: | ---: | ---: | ---: | ---: |
        #{detailed_rows}

        ## Interpretation

        The one-file result is dominated by process startup. At 500 files, rustocop is
        about #{speedup.round} times faster than RuboCop. This tiny corpus measures the complete CLI,
        configuration, file, traversal, and formatting paths together—not parsing alone.
      MARKDOWN
      GeneratedSection.replace(File.join(root, "docs/performance.md"), "rubocop-prism-throughput", <<~MARKDOWN)
        ```mermaid
        xychart-beta
            title "End-to-end speedup over RuboCop + Prism"
            x-axis "Ruby files" [#{results.map { |result| result.fetch("files") }.join(", ")}]
            y-axis "Speedup (times)" 0 --> #{(results.map { |result| result.fetch("speedup_vs_rubocop") }.max / 10.0).ceil * 10}
            bar [#{results.map { |result| format("%.2f", result.fetch("speedup_vs_rubocop")) }.join(", ")}]
        ```

        At 500 files, median throughput was approximately
        #{format_integer(final.dig("rustocop", "files_per_second").round)} files/second for Rustocop and
        #{format_integer(final.dig("rubocop", "files_per_second").round)} for RuboCop with Prism.
      MARKDOWN
    end

    def update_mixed(root, report)
      values = report.fetch("measurements")
      native = values.fetch("native_binary").fetch("median_seconds")
      mixed = values.fetch("mixed").fetch("median_seconds")
      rubocop_all = values.fetch("rubocop_all").fetch("median_seconds")
      readme_rows = [
        ["Pure native, 20 built-in cops", "native_binary"],
        ["Mixed, 20 native + 1 Ruby custom cop", "mixed"],
        ["Pure RuboCop, all 21 cops", "rubocop_all"]
      ].map { |label, key| "| #{label} | **#{milliseconds(values.dig(key, "median_seconds"), 2)} ms** |" }.join("\n")
      full_rows = values.map do |name, measurement|
        label = mixed_label(name)
        relative = measurement.fetch("median_seconds").fdiv(native)
        "| #{label} | #{milliseconds(measurement.fetch("median_seconds"), 2)} ms | " \
          "#{milliseconds(measurement.fetch("p95_seconds"), 2)} ms | #{format("%.1f", relative)}× |"
      end.join("\n")
      faster = (1 - mixed.fdiv(rubocop_all)) * 100
      slowdown = mixed.fdiv(native)

      GeneratedSection.replace(File.join(root, "README.md"), "mixed-custom", <<~MARKDOWN)
        Rustocop can keep recognized built-in cops native while delegating explicitly
        selected Ruby custom cops to RuboCop. On the same 500 files, 20 native cops plus
        one custom cop took #{milliseconds(mixed, 2)} ms, versus #{milliseconds(native, 2)} ms for pure native Rustocop and
        #{milliseconds(rubocop_all, 2)} ms for pure RuboCop.

        | 500-file mode | Median |
        | --- | ---: |
        #{readme_rows}
      MARKDOWN
      mixed_body = <<~MARKDOWN
        | Variant | Median | p95 | Relative to native binary |
        | --- | ---: | ---: | ---: |
        #{full_rows}

        The direct mixed run was #{format("%.1f", faster)}% faster than pure RuboCop and produced
        identical normalized JSON. One Ruby custom cop still made this tiny-corpus run
        #{format("%.1f", slowdown)} times slower than pure native Rustocop because RuboCop must
        start Ruby, load the custom cop, and build a second set of Prism trees.
      MARKDOWN
      GeneratedSection.replace(File.join(root, "benchmark/mixed-custom-cops.md"), "mixed-custom-results", mixed_body)
      GeneratedSection.replace(File.join(root, "adrs/enable-custom-ruby-cops.md"), "mixed-custom-results", mixed_body)
    end

    def update_memory(root, report)
      results = report.fetch("results")
      final = results.last
      rows = results.map do |result|
        "| #{result.fetch("files")} | #{memory_measurement(result.fetch("rustocop"))} | " \
          "#{memory_measurement(result.fetch("rustocop_parallel"))} | " \
          "#{memory_measurement(result.fetch("rubocop_prism"))} |"
      end.join("\n")
      sequential = final.dig("rustocop", "median_peak_rss_bytes")
      parallel = final.dig("rustocop_parallel", "median_peak_rss_bytes")
      rubocop = final.dig("rubocop_prism", "median_peak_rss_bytes")

      GeneratedSection.replace(File.join(root, "docs/performance.md"), "memory-results", <<~MARKDOWN)
        | Files | Rustocop sequential median / p95 | Rustocop parallel median / p95 | RuboCop + Prism median / p95 |
        | ---: | ---: | ---: | ---: |
        #{rows}

        The nearly flat curves show that fixed runtime and startup cost dominate this
        corpus. At 500 files, automatic parallel execution added about
        #{format("%.2f", bytes_to_mib(parallel - sequential))} MiB over sequential rustocop and RuboCop used
        #{format("%.1f", rubocop.fdiv(parallel))} times as much peak memory as parallel rustocop.
        This does not imply that arbitrary Ruby files cost only a few KiB each: the
        pinned corpus totals just 9,110 source bytes. Large files, large literals,
        and more complex syntax need a separate sustained-memory benchmark.
      MARKDOWN
    end

    def measurement(value)
      "#{milliseconds(value.fetch("median_seconds"), 3)} / #{milliseconds(value.fetch("p95_seconds"), 3)} ms"
    end

    def milliseconds(seconds, precision)
      format("%.#{precision}f", seconds * 1000)
    end

    def mixed_label(name)
      {
        "native_binary" => "Rustocop native binary, 20 built-in cops",
        "native_entrypoint" => "Rustocop Ruby entrypoint, 20 built-in cops",
        "mixed" => "Mixed native binary, 20 native + 1 custom cop",
        "mixed_entrypoint" => "Mixed Ruby entrypoint, 20 native + 1 custom cop",
        "rubocop_custom_only" => "RuboCop, custom cop only",
        "rubocop_all" => "RuboCop, all 20 built-ins + custom cop"
      }.fetch(name)
    end

    def memory_measurement(value)
      median = bytes_to_mib(value.fetch("median_peak_rss_bytes"))
      p95 = bytes_to_mib(value.fetch("p95_peak_rss_bytes"))
      "#{format("%.2f", median)} / #{format("%.2f", p95)} MiB"
    end

    def bytes_to_mib(bytes)
      bytes.fdiv(1024**2)
    end

    def format_integer(number)
      number.to_s.reverse.scan(/.{1,3}/).join(",").reverse
    end
    private_class_method :measurement, :milliseconds, :mixed_label, :memory_measurement, :bytes_to_mib, :format_integer
  end
end
