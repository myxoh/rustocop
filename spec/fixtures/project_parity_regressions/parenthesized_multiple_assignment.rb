return unless (api, list, arg = api_name_membership?(node))

if (arguments, receiver = reject_with_block?(node.parent))
  use_arguments(arguments, receiver)
end

return unless (receiver, object = negate_include_call?(node))
