# frozen_string_literal: true

require "rbconfig"

module Rustocop
  class Runner
    GEM_ROOT = File.expand_path("../..", __dir__)

    def self.call(argv = ARGV)
      new(argv).call
    end

    def initialize(argv)
      @argv = argv
    end

    def call
      executable = selected_executable
      warn "rustocop: using Ruby fallback at #{executable}" if fallback?(executable) && ENV["RUSTOCOP_WARN_FALLBACK"]

      Kernel.exec(runtime_environment, executable, *@argv)
    rescue SystemCallError => e
      warn "rustocop: failed to launch #{executable}: #{e.message}"
      2
    end

    private

    attr_reader :argv

    def selected_executable
      return fallback_executable if ENV["RUSTOCOP_DISABLE_NATIVE"]

      configured = ENV["RUSTOCOP_NATIVE_PATH"]
      return configured if configured && File.executable?(configured)

      return native_executable if File.executable?(native_executable)

      fallback_executable
    end

    def fallback?(executable)
      executable == fallback_executable
    end

    def native_executable
      File.join(GEM_ROOT, "libexec", "rustocop-native#{executable_extension}")
    end

    def fallback_executable
      File.join(GEM_ROOT, "libexec", "rustocop-ruby")
    end

    def executable_extension
      RbConfig::CONFIG.fetch("host_os").match?(/mswin|mingw|cygwin/) ? ".exe" : ""
    end

    def runtime_environment
      {
        "RUSTOCOP_VERSION" => Rustocop::VERSION,
        "RUSTOCOP_RUBY_ENGINE" => RUBY_ENGINE,
        "RUSTOCOP_RUBY_VERSION" => RUBY_VERSION,
        "RUSTOCOP_RUBY_PATCHLEVEL" => RUBY_PATCHLEVEL.to_s,
        "RUSTOCOP_RUBY_PLATFORM" => RUBY_PLATFORM
      }
    end
  end
end
