# frozen_string_literal: true

require "json"

module UpstreamCopCapture
  def expect_offense(...)
    offenses = rustocop_without_investigation_capture { super }
    capture_upstream_case(@processed_source.raw_source, offenses)
    offenses
  end

  def expect_no_offenses(source, file = nil)
    result = rustocop_without_investigation_capture { super }
    capture_upstream_case(source, [], file: file)
    result
  end

  def expect_correction(correction, loop: true, source: nil)
    result = rustocop_without_investigation_capture { super }

    if source
      capture_upstream_case(source, nil, correction: correction)
    else
      raise "captured correction has no preceding source" unless rustocop_capture_cases.last

      rustocop_capture_cases.last["correction"] = correction
    end

    result
  end

  def expect_no_corrections
    result = rustocop_without_investigation_capture { super }
    raise "captured no-correction assertion has no preceding source" unless rustocop_capture_cases.last

    rustocop_capture_cases.last["correction"] = nil
    rustocop_capture_cases.last["asserts_no_correction"] = true
    result
  end

  def rustocop_capture_cases
    @rustocop_capture_cases ||= []
  end

  def rustocop_flush_capture(example)
    return @rustocop_capture_cases = [] if example.exception

    if rustocop_capture_cases.empty? && respond_to?(:cop, true) && respond_to?(:offenses, true) &&
       respond_to?(:processed_source, true)
      capture_direct_investigation_case
    end
    return unless @rustocop_capture_cases

    output = ENV.fetch("RUSTOCOP_UPSTREAM_CAPTURE")
    File.open(output, "a") do |file|
      @rustocop_capture_cases.each_with_index do |test_case, index|
        test_case["example"] = {
          "id" => example.id,
          "description" => example.full_description,
          "file" => relative_upstream_path(example.metadata[:file_path]),
          "line" => example.metadata[:line_number],
          "expectation" => index + 1
        }
        file.puts(JSON.generate(capture_json_value(test_case)))
      end
    end
  ensure
    @rustocop_capture_cases = []
  end

  private

  def _investigate(cop_instance, processed_source)
    offenses = super
    unless @rustocop_capture_suppressed
      capture_upstream_case(
        processed_source.raw_source,
        offenses,
        file: processed_source.buffer.name
      )
    end
    offenses
  end

  def rustocop_without_investigation_capture
    previous = @rustocop_capture_suppressed
    @rustocop_capture_suppressed = true
    yield
  ensure
    @rustocop_capture_suppressed = previous
  end

  def capture_direct_investigation_case
    capture_upstream_case(processed_source.raw_source, offenses, file: processed_source.buffer.name)
  end

  def capture_upstream_case(source, offenses, file: nil, correction: :unspecified)
    test_case = {
      "cop" => cop.class.cop_name,
      "source" => source,
      "path" => capture_path(file),
      "ruby_version" => ruby_version.to_s,
      "parser_engine" => parser_engine.to_s,
      "cop_options" => defined?(cop_options) ? cop_options : {},
      "config" => configuration.to_h,
      "offenses" => offenses&.map { |offense| capture_offense(offense) }
    }
    test_case["correction"] = correction unless correction == :unspecified
    rustocop_capture_cases << test_case
  end

  def capture_offense(offense)
    location = offense.location
    {
      "message" => offense.message,
      "severity" => offense.severity.name.to_s,
      "correctable" => offense.correctable?,
      "line" => location.line,
      "column" => location.column + 1,
      "last_line" => location.last_line,
      "last_column" => location.last_column,
      "begin_pos" => location.begin_pos,
      "end_pos" => location.end_pos
    }
  end

  def capture_path(file)
    path = file.respond_to?(:path) ? file.path : file
    path ? path.to_s : "example.rb"
  end

  def relative_upstream_path(path)
    root = File.expand_path("../upstream/rubocop-1.87.0", __dir__)
    File.expand_path(path).delete_prefix("#{root}/")
  end

  def capture_json_value(value)
    case value
    when Hash
      value.to_h { |key, child| [key.to_s, capture_json_value(child)] }
    when Array
      value.map { |child| capture_json_value(child) }
    when String
      utf8 = value.dup.force_encoding(Encoding::UTF_8)
      return utf8 if utf8.valid_encoding?

      { "$hex" => value.b.unpack1("H*") }
    when Float
      value.finite? ? value : { "$float" => value.to_s }
    when Integer, TrueClass, FalseClass, NilClass
      value
    when Symbol
      value.to_s
    when Regexp
      { "$regexp" => value.source, "options" => value.options }
    else
      value.to_s
    end
  end
end
