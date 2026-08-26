# frozen_string_literal: true

require "open3"

module Rustocop
  module ProcessRunner
    Result = Data.define(:stdout, :stderr, :status, :seconds) do
      def exitstatus
        status.exitstatus
      end

      def success?
        status.success?
      end

      def accepted?(*statuses)
        statuses.flatten.include?(exitstatus)
      end

      def to_h
        {
          "stdout" => stdout,
          "stderr" => stderr,
          "exitstatus" => exitstatus,
          "seconds" => seconds
        }
      end
    end

    module_function

    def capture(*command, chdir: nil, env: {}, stdin_data: nil)
      options = { stdin_data: }
      options[:chdir] = chdir if chdir
      started = Process.clock_gettime(Process::CLOCK_MONOTONIC)
      stdout, stderr, status = Open3.capture3(env, *command, options)
      seconds = Process.clock_gettime(Process::CLOCK_MONOTONIC) - started
      Result.new(stdout:, stderr:, status:, seconds:)
    end
  end
end
