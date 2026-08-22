# frozen_string_literal: true

require 'auto_freeze'

## Freeze all gems, except for some which has issues at load time:
#
# Example:
# exclude_gems = %w[
#   arr-pm
#   email_reply_trimmer
#   method_source
#   seed-fu
#   unicode_utils
# ].freeze
# AutoFreeze.setup!(excluded_gems: exclude_gems)

# Skip installing the global RubyVM::InstructionSequence.load_iseq hook (via
