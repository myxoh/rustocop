# Minimized from rubygems/rubygems.org@3201f8831866f82eb9acd7f66287a978d0e59079
# app/controllers/concerns/avo_auditable.rb.

def perform_action_and_record_errors(&blk)
  in_audited_transaction(&blk)
end

expect { |block| perform_action_and_record_errors(&block) }.to yield_control

def perform_with_options(option: nil, &block)
  perform_action_and_record_errors(&block)
end
